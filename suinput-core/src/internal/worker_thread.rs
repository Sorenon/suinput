use std::{
    collections::HashMap,
    ops::Deref,
    sync::{Arc, Weak},
    thread::JoinHandle,
};

use flume::Receiver;

use parking_lot::Mutex;

use suinput_types::SuPath;
use thunderdome::{Arena, Index};

use crate::{
    internal::interaction_profile::InteractionProfileState,
    runtime::{Driver2RuntimeEvent, Driver2RuntimeEventResponse, Runtime},
};

use super::{
    binding::working_user::WorkingUser,
    device::DeviceState,
    interaction_profile_type::{self, InteractionProfileType},
    interaction_profile_types::InteractionProfileTypes,
};

#[derive(Debug)]
pub enum WorkerThreadEvent {
    Poll {
        instance: u64,
    },
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
        let mut working_user = WorkingUser {
            binding_layouts: HashMap::new(),
            action_states: HashMap::new(),
        };

        let mut device_states = Arena::<(SuPath, DeviceState, Index)>::new();
        let mut interaction_profile_states = Arena::<InteractionProfileState>::new();

        let desktop_profile_id = interaction_profile_states.insert(InteractionProfileState::new(
            runtime
                .interaction_profile_types
                .get(runtime.common_paths.desktop)
                .unwrap()
                .clone(),
        ));

        while let Ok(event) = driver2runtime_receiver.recv() {
            match event {
                WorkerThreadEvent::Poll { instance } => {
                    let instances = runtime.instances.read();
                    let instance = instances.get(instance as usize - 1).unwrap();
                    let mut user = instance.user.write();

                    for (profile, binding_layout) in user.new_binding_layouts.drain() {
                        working_user.binding_layouts.insert(profile, binding_layout);
                    }
                }
                WorkerThreadEvent::DriverEvent { id, event } => {
                    match event {
                        Driver2RuntimeEvent::RegisterDevice(ty) => {
                            //TODO: Device ID persistence
                            let device_id = device_states.insert((
                                ty,
                                DeviceState::default(),
                                desktop_profile_id,
                            ));

                            runtime
                                .driver_response_senders
                                .lock()
                                .get(id)
                                .expect("Could not access driver response channel")
                                .send(Driver2RuntimeEventResponse::DeviceId(device_id.to_bits()))
                                .expect("Driver response channel closed unexpectedly");

                            interaction_profile_states
                                .get_mut(desktop_profile_id)
                                .unwrap()
                                .device_added(device_id, ty);
                        }
                        Driver2RuntimeEvent::Input(event) => {
                            let instances = runtime.instances.read();
                            let device_idx = Index::from_bits(event.device).unwrap();
                            let (_, _, interaction_profile_id) =
                                device_states.get(device_idx).unwrap();

                            interaction_profile_states
                                .get_mut(*interaction_profile_id)
                                .unwrap()
                                .update_component(
                                    &event,
                                    &device_states,
                                    &instances.first().unwrap(),
                                    &mut working_user,
                                );

                            let device = device_states.get_mut(device_idx).unwrap();
                            let device_state = &mut device.1;

                            device_state.update_input(event);
                        }
                        Driver2RuntimeEvent::DisconnectDevice(id) => {
                            let device_idx = Index::from_bits(id).unwrap();
                            let (_, _, interaction_profile_id) =
                                device_states.get(device_idx).unwrap();

                            interaction_profile_states
                                .get_mut(*interaction_profile_id)
                                .unwrap()
                                .device_removed(device_idx, &device_states);
                            device_states.remove(device_idx);
                        }
                    }
                }
            }
        }
    })
}
