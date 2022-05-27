use std::{
    collections::HashMap,
    sync::{Arc, Weak},
    thread::JoinHandle,
    time::Instant,
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
    binding::working_user::WorkingUser, device::DeviceState,
    interaction_profile_type::InteractionProfileType,
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
        let mut user = WorkingUser {
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
