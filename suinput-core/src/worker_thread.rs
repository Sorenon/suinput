use std::{
    collections::HashMap,
    sync::{Arc, Weak},
    thread::JoinHandle,
    time::Instant,
};

use flume::Receiver;
use log::warn;
use parking_lot::Mutex;

use suinput_types::{
    action::{ActionEvent, ActionEventEnum},
    event::InputEvent,
    SuPath,
};
use thunderdome::{Arena, Index};

use crate::{
    binding_engine::{ActionStateEnum, ProcessedBindingLayout},
    instance::Instance,
    interaction_profile::{DeviceState, InteractionProfileState, InteractionProfileType},
    runtime::{Driver2RuntimeEvent, Driver2RuntimeEventResponse, Runtime},
};

#[derive(Debug)]
pub enum WorkerThreadEvent {
    Poll,
    DriverEvent {
        id: usize,
        event: Driver2RuntimeEvent,
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

        let runtime = runtime.upgrade().unwrap();

        //For now we just assume one instance and one user
        let mut user = WorkerThreadUser {
            binding_layouts: HashMap::new(),
            action_states: HashMap::new(),
        };

        let mut devices = Arena::<(SuPath, DeviceState)>::new();

        let mut desktop_profile =
            InteractionProfileState::new(InteractionProfileType::new_desktop_profile(|str| {
                runtime.paths.get_path(str).unwrap()
            }));

        while let Ok(event) = driver2runtime_receiver.recv() {
            match event {
                WorkerThreadEvent::Poll => {
                    let instances = runtime.instances.read();
                    let instance = instances.first().unwrap();
                    let mut instance_user = instance.user.write();

                    for (profile, binding_layout) in instance_user.new_binding_layouts.drain() {
                        user.binding_layouts.insert(profile, binding_layout);
                    }
                }
                WorkerThreadEvent::DriverEvent { id, event } => {
                    match event {
                        Driver2RuntimeEvent::RegisterDevice(ty) => {
                            //TODO: Device ID persistence
                            let device_id = devices.insert((ty, DeviceState::default()));

                            runtime
                                .driver_response_senders
                                .lock()
                                .get(id)
                                .expect("Could not access driver response channel")
                                .send(Driver2RuntimeEventResponse::DeviceId(device_id.to_bits()))
                                .expect("Driver response channel closed unexpectedly");

                            desktop_profile.device_added(device_id, ty);
                        }
                        Driver2RuntimeEvent::Input(event) => {
                            let instances = runtime.instances.read();
                            desktop_profile.update_component(
                                &event,
                                &devices,
                                &instances.first().unwrap(),
                                &mut user,
                            );

                            let device = devices
                                .get_mut(Index::from_bits(event.device).unwrap())
                                .unwrap();
                            let device_state = &mut device.1;

                            device_state.update_input(event);
                        }
                        Driver2RuntimeEvent::DisconnectDevice(id) => {
                            let index = Index::from_bits(id).unwrap();

                            desktop_profile.device_removed(index, &devices);
                            devices.remove(index);
                        }
                    }
                }
            }
        }
    })
}

pub(crate) struct WorkerThreadUser {
    binding_layouts: HashMap<SuPath, ProcessedBindingLayout>,
    action_states: HashMap<u64, ActionStateEnum>,
}

impl WorkerThreadUser {
    pub(crate) fn on_event(
        &mut self,
        interaction_profile: &InteractionProfileType,
        user_path: SuPath,
        event: &InputEvent,
        instance: &Instance,
    ) {
        if let Some(binding_layout) = self.binding_layouts.get_mut(&interaction_profile.id) {
            binding_layout.on_event(
                user_path,
                event,

                //How can we handle passing iterators between user and layout without copying possibly large amounts of data to the heap?
                |action_handle, binding_index, &binding_state, binding_layout | {
                    let event = match binding_state {
                        ActionStateEnum::Boolean(new_binding_state) => {
                            let old_action_state = match self.action_states.get(&action_handle) {
                                Some(ActionStateEnum::Boolean(action_state)) => *action_state,
                                _ => false,
                            };

                            if new_binding_state {
                                self.action_states.insert(action_handle, ActionStateEnum::Boolean(true));
                                ActionEventEnum::Boolean {
                                    state: true,
                                    changed: new_binding_state != old_action_state,
                                }
                            } else {
                                if old_action_state {
                                    let none_other_true = binding_layout
                                    .bindings_for_action
                                    .get(&action_handle)
                                    .unwrap()
                                    .iter()
                                    .filter(|idx| **idx != binding_index)
                                    .find(|idx| {
                                        let (_, state, _) = &binding_layout.bindings_index[**idx];
                                        match state {
                                            ActionStateEnum::Boolean(state) => *state,
                                            _ => unreachable!(),
                                        }
                                    })
                                    .is_none();

                                    if none_other_true {
                                        ActionEventEnum::Boolean {
                                            state: false,
                                            changed: true,
                                        }
                                    }
                                    else {
                                        return;
                                    }
                                } else {
                                    warn!("Somehow fired a false event on an action which is already false");
                                    return;
                                }
                            }
                        }
                        ActionStateEnum::Delta2D(delta) => ActionEventEnum::Delta2D { delta },
                        ActionStateEnum::Cursor(_) => todo!(),
                    };

                    let event = ActionEvent {
                        action_handle,
                        time: Instant::now(),
                        data: event,
                    };
                    for listener in instance.listeners.read().iter() {
                        listener.handle_event(event);
                    }
                },
            );
        }
    }
}
