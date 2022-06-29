use std::{
    ops::Deref,
    sync::{Arc, Weak},
    thread::JoinHandle,
    time::Instant,
};

use flume::Receiver;

use parking_lot::Mutex;

use suinput_types::{
    action::{ActionEvent, ActionStateEnum},
    event::{Cursor, InputComponentEvent, InputEvent},
    Time,
};
use thunderdome::{Arena, Index};

use crate::internal::types::HashMap;
use crate::{
    internal::interaction_profile::InteractionProfileState,
    runtime::{Driver2RuntimeEvent, Driver2RuntimeEventResponse, Runtime},
    session::Session,
};

use super::{
    binding::{
        action_hierarchy::{handle_sticky_bool_event, ParentActionState},
        working_user::{AttachedBindingLayout, WorkingUser},
    },
    device::DeviceState,
    paths::DevicePath,
};

#[derive(Debug)]
pub enum WorkerThreadEvent {
    Poll {
        session: u64,
    },
    Output(OutputEvent),
    DriverEvent {
        id: usize,
        event: Driver2RuntimeEvent,
    },
}

#[derive(Debug)]
pub enum OutputEvent {
    ReleaseStickyBool { session: u64, action: u64 },
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
                WorkerThreadEvent::Poll { session } => {
                    worker_thread.poll(session);
                }
                WorkerThreadEvent::DriverEvent { id, event } => match event {
                    Driver2RuntimeEvent::RegisterDevice(ty) => {
                        worker_thread.register_new_device(id, ty);
                    }
                    Driver2RuntimeEvent::Input(event) => {
                        worker_thread.on_input_event(event);
                    }
                    Driver2RuntimeEvent::BatchInput(batch_update) => {
                        let (device, _) = worker_thread
                            .device_states
                            .get_mut(Index::from_bits(batch_update.device).unwrap())
                            .unwrap();

                        device
                            .handle_batch(batch_update.time, &batch_update.inner)
                            .unwrap();

                        for (path, data) in batch_update.inner {
                            worker_thread.on_input_event(InputEvent {
                                device: batch_update.device,
                                time: Time(0),
                                path,
                                data,
                            })
                        }
                    }
                    Driver2RuntimeEvent::DisconnectDevice(id) => {
                        let device_idx = Index::from_bits(id).unwrap();
                        let (_, interaction_profile_id) =
                            worker_thread.device_states.get(device_idx).unwrap();

                        worker_thread
                            .interaction_profile_states
                            .get_mut(*interaction_profile_id)
                            .unwrap()
                            .device_removed(device_idx, &worker_thread.device_states);
                        worker_thread.device_states.remove(device_idx);
                    }
                },
                WorkerThreadEvent::Output(data) => match data {
                    OutputEvent::ReleaseStickyBool { session, action } => {
                        let (session, working_user) =
                            worker_thread.sessions.get_mut(&session).unwrap();

                        if let Some(ParentActionState::StickyBool { stuck, .. }) =
                            working_user.parent_action_states.get_mut(&action)
                        {
                            *stuck = false;
                        }

                        if let Some(event) = handle_sticky_bool_event(
                            action,
                            &mut working_user.parent_action_states,
                            &working_user.action_states,
                        ) {
                            let event = ActionEvent {
                                action_handle: action,
                                time: Instant::now(),
                                data: event,
                            };
                            for listener in session.listeners.write().iter_mut() {
                                listener.handle_event(event, 0);
                            }
                        }
                    }
                },
            }
        }
    })
}

struct WorkerThread {
    runtime: Arc<Runtime>,

    //For now we just assume one user per session
    sessions: HashMap<u64, (Arc<Session>, WorkingUser)>,

    device_states: Arena<(DeviceState, Index)>,
    interaction_profile_states: Arena<InteractionProfileState>,

    desktop_profile_id: Index,
}

impl WorkerThread {
    pub fn new(runtime: Weak<Runtime>) -> Self {
        let runtime = runtime.upgrade().unwrap();

        let mut interaction_profile_states = Arena::<InteractionProfileState>::new();

        let desktop_profile_id = interaction_profile_states.insert(InteractionProfileState::new(
            runtime
                .interaction_profile_types
                .get(runtime.common_paths.desktop)
                .unwrap()
                .clone(),
        ));

        Self {
            runtime,
            sessions: HashMap::new(),
            device_states: Arena::new(),
            interaction_profile_states,
            desktop_profile_id,
        }
    }

    fn poll(&mut self, session: u64) {
        if !self.sessions.contains_key(&session) {
            let runtime_sessions = self.runtime.sessions.read();
            let session = runtime_sessions.get(session as usize - 1).unwrap();

            self.sessions.insert(
                session.runtime_handle,
                (session.clone(), WorkingUser::new(&session.action_sets)),
            );
        }

        let (session, working_user) = self.sessions.get_mut(&session).unwrap();

        let user = &session.user;

        for (profile, binding_layout) in user.new_binding_layouts.lock().drain() {
            working_user
                .binding_layouts
                .insert(profile, AttachedBindingLayout::new(binding_layout));
        }

        let mut user_action_states = session.user.action_states.write();

        for (path, working_action_state) in working_user.action_states.iter_mut() {
            let action_state = &mut working_action_state.state;
            if let Some(parent_action_state) = working_user.parent_action_states.get(path) {
                user_action_states.insert(
                    *path,
                    match parent_action_state {
                        ParentActionState::StickyBool { combined_state, .. } => {
                            ActionStateEnum::Boolean(*combined_state)
                        }
                        ParentActionState::Axis1d { combined_state, .. } => {
                            ActionStateEnum::Axis1d(*combined_state)
                        }
                        ParentActionState::Axis2d { combined_state, .. } => {
                            ActionStateEnum::Axis2d((*combined_state).into())
                        }
                    },
                );
            } else {
                user_action_states.insert(*path, *action_state);
            }

            match action_state {
                ActionStateEnum::Delta2d(delta) => {
                    *delta = mint::Vector2 { x: 0., y: 0. };
                }
                _ => (),
            }
        }

        session
            .done
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    fn register_new_device(&mut self, driver_id: usize, ty: DevicePath) {
        let interaction_profile_id = if ty == self.runtime.common_paths.system_cursor
            || ty == self.runtime.common_paths.keyboard
            || ty == self.runtime.common_paths.mouse
        {
            self.desktop_profile_id
        } else if ty == self.runtime.controller_paths.device_dual_sense {
            let interaction_profile_type = self
                .runtime
                .interaction_profile_types
                .get(self.runtime.controller_paths.interaction_profile_dualsense)
                .unwrap();
            self.interaction_profile_states
                .insert(InteractionProfileState::new(
                    interaction_profile_type.clone(),
                ))
        } else {
            todo!()
        };

        //TODO: Device ID persistence
        let device_id = self.device_states.insert((
            DeviceState::new(self.runtime.device_types.get(ty).unwrap().clone()),
            interaction_profile_id,
        ));

        self.runtime
            .driver_response_senders
            .lock()
            .get(driver_id)
            .expect("Could not access driver response channel")
            .send(Driver2RuntimeEventResponse::DeviceId(device_id.to_bits()))
            .expect("Driver response channel closed unexpectedly");

        self.interaction_profile_states
            .get_mut(interaction_profile_id)
            .unwrap()
            .device_added(device_id, ty);
    }

    fn on_input_event(&mut self, event: InputEvent) {
        let device_idx = Index::from_bits(event.device).unwrap();

        let device = self.device_states.get_mut(device_idx).unwrap();
        if let Some(event) = device.0.process_input_event(event) {
            let (_, interaction_profile_id) = self.device_states.get(device_idx).unwrap();
            self.interaction_profile_states
                .get_mut(*interaction_profile_id)
                .unwrap()
                .update_component(
                    &event,
                    &self.device_states,
                    |profile_state, user_path, event, devices| {
                        // println!("{event:?}");

                        for (session, working_user) in self.sessions.values_mut() {
                            if let InputComponentEvent::Cursor(Cursor {
                                window: Some(window),
                                ..
                            }) = event.data
                            {
                                let session_window = session.window.lock();
                                if let Some(session_window) = session_window.deref() {
                                    if *session_window != window {
                                        return;
                                    }
                                } else {
                                    return;
                                }
                            }

                            working_user.on_event(
                                &profile_state,
                                user_path,
                                event,
                                session,
                                devices,
                            );
                        }
                    },
                );
        }
    }
}
