use std::{
    ops::Deref,
    sync::{Arc, Weak},
    thread::JoinHandle,
};

use flume::Receiver;

use log::warn;
use parking_lot::Mutex;

use suinput_types::{
    action::ActionStateEnum,
    event::{Cursor, InputComponentEvent, InputEvent},
    Time,
};
use thunderdome::{Arena, Index};

use crate::{
    internal::interaction_profile::InteractionProfileState,
    runtime::{Driver2RuntimeEvent, Driver2RuntimeEventResponse, Runtime},
    session::Session,
};
use crate::{internal::types::HashMap, session};

use super::{
    binding::{
        action_hierarchy::ParentActionState,
        working_user::{AttachedBindingLayout, WorkingUser},
    },
    device::DeviceState,
    inner_session::Runtime2SessionEvent,
    parallel_arena::ParallelArena,
    paths::DevicePath,
};

#[derive(Debug)]
pub enum WorkerThreadEvent {
    Driver {
        id: usize,
        event: Driver2RuntimeEvent,
    },
    CreateSession {
        handle: Index,
    },
}

pub fn spawn_thread(
    driver2runtime_receiver: Receiver<WorkerThreadEvent>,
    runtime: Weak<Runtime>,
    ready: Arc<Mutex<()>>,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        std::mem::drop(ready.lock());
        std::mem::drop(ready);

        let mut worker_thread = WorkerThread::new(runtime);

        while let Ok(event) = driver2runtime_receiver.recv() {
            match event {
                WorkerThreadEvent::Driver { id, event } => match event {
                    Driver2RuntimeEvent::RegisterDevice(ty) => {
                        worker_thread.register_new_device(id, ty);
                    }
                    Driver2RuntimeEvent::Input(event) => {
                        worker_thread.on_input_event(event);
                    }
                    Driver2RuntimeEvent::BatchInput(batch_update) => {
                        todo!()
                    }
                    Driver2RuntimeEvent::DisconnectDevice(id) => {
                        let device_idx = Index::from_bits(id).unwrap();

                        worker_thread.device_states.remove(device_idx);
                    }
                },
                WorkerThreadEvent::CreateSession { handle } => {
                    let session = worker_thread.runtime.sessions.read().get(handle).cloned();
                    if let Some(session) = session {
                        for (device_index, device_state) in worker_thread.device_states.iter() {
                            session
                                .driver_events_send
                                .send(Runtime2SessionEvent::RegisterDevice {
                                    idx: device_index,
                                    ty: device_state.ty.clone(),
                                })
                                .unwrap();
                        }

                        worker_thread.sessions.insert_at(handle, session);
                    } else {
                        warn!(
                            "Session {:?} deleted before worker thread initialization",
                            handle
                        )
                    }
                }
            }
        }
    })
}

struct WorkerThread {
    runtime: Arc<Runtime>,
    sessions: ParallelArena<Arc<Session>>,
    device_states: Arena<DeviceState>,
}

impl WorkerThread {
    pub fn new(runtime: Weak<Runtime>) -> Self {
        Self {
            runtime: runtime.upgrade().unwrap(),
            sessions: ParallelArena::new(),
            device_states: Arena::new(),
        }
    }

    fn register_new_device(&mut self, driver_id: usize, ty: DevicePath) {
        let device_type = self.runtime.device_types.get(ty).unwrap();

        //TODO: Device ID persistence
        let device_id = self
            .device_states
            .insert(DeviceState::new(device_type.clone()));

        self.runtime
            .driver_response_senders
            .lock()
            .get(driver_id)
            .expect("Could not access driver response channel")
            .send(Driver2RuntimeEventResponse::DeviceId(device_id.to_bits()))
            .expect("Driver response channel closed unexpectedly");

        for session in self.sessions.iter() {
            session
                .driver_events_send
                .send(Runtime2SessionEvent::RegisterDevice {
                    idx: device_id,
                    ty: device_type.clone(),
                })
                .unwrap();
        }
    }

    fn on_input_event(&mut self, event: InputEvent) {
        let device_idx = Index::from_bits(event.device).unwrap();
        self.device_states.get(device_idx).unwrap();

        for session in self.sessions.iter() {
            session
                .driver_events_send
                .send(Runtime2SessionEvent::Input(event))
                .unwrap();
        }
    }
}
