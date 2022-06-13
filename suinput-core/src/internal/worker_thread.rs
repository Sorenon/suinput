use std::{
    collections::HashMap,
    ops::Deref,
    sync::{Arc, Weak},
    thread::JoinHandle,
    time::Instant,
};

use flume::Receiver;

use nalgebra::Vector2;
use parking_lot::Mutex;

use suinput_types::{
    action::ActionEvent,
    event::{Cursor, InputComponentEvent},
    SuPath,
};
use thunderdome::{Arena, Index};

use crate::{
    action::{ActionHierarchyType, ParentActionType},
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

        let runtime = runtime.upgrade().unwrap();

        //For now we just assume one user per session
        let mut sessions = HashMap::<u64, (Arc<Session>, WorkingUser)>::new();

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
                WorkerThreadEvent::Poll { session } => {
                    if !sessions.contains_key(&session) {
                        let runtime_sessions = runtime.sessions.read();
                        let session = runtime_sessions.get(session as usize - 1).unwrap();

                        let parent_action_states = session
                            .actions
                            .values()
                            .filter_map(|action| match &action.hierarchy_type {
                                ActionHierarchyType::Parent {
                                    ty:
                                        ParentActionType::StickyBool {
                                            sticky_press,
                                            sticky_release,
                                            sticky_toggle,
                                        },
                                } => Some((
                                    action.handle,
                                    ParentActionState::StickyBool {
                                        combined_state: false,
                                        stuck: false,
                                        press: sticky_press.handle,
                                        release: sticky_release.handle,
                                        toggle: sticky_toggle.handle,
                                    },
                                )),
                                ActionHierarchyType::Parent {
                                    ty: ParentActionType::Axis1d { positive, negative },
                                } => Some((
                                    action.handle,
                                    ParentActionState::Axis1d {
                                        combined_state: 0.,
                                        positive: positive.handle,
                                        negative: negative.handle,
                                    },
                                )),
                                ActionHierarchyType::Parent {
                                    ty:
                                        ParentActionType::Axis2d {
                                            up,
                                            down,
                                            left,
                                            right,
                                            vertical,
                                            horizontal,
                                        },
                                } => Some((
                                    action.handle,
                                    ParentActionState::Axis2d {
                                        combined_state: Vector2::new(0., 0.),
                                        up: up.handle,
                                        down: down.handle,
                                        left: left.handle,
                                        right: right.handle,
                                        vertical: vertical.handle,
                                        horizontal: horizontal.handle,
                                    },
                                )),
                                _ => None,
                            })
                            .collect();

                        sessions.insert(
                            session.runtime_handle,
                            (session.clone(), WorkingUser::new(parent_action_states)),
                        );
                    }

                    let (session, working_user) = sessions.get_mut(&session).unwrap();

                    let user = &session.user;

                    for (profile, binding_layout) in user.new_binding_layouts.lock().drain() {
                        working_user
                            .binding_layouts
                            .insert(profile, AttachedBindingLayout::new(binding_layout));
                    }
                }
                WorkerThreadEvent::DriverEvent { id, event } => {
                    match event {
                        Driver2RuntimeEvent::RegisterDevice(ty) => {
                            let interaction_profile_id = if ty == runtime.common_paths.system_cursor
                                || ty == runtime.common_paths.keyboard
                                || ty == runtime.common_paths.mouse
                            {
                                desktop_profile_id
                            } else if ty == runtime.controller_paths.device_dual_sense {
                                let interaction_profile_type = runtime
                                    .interaction_profile_types
                                    .get(runtime.controller_paths.interaction_profile_dualsense)
                                    .unwrap();
                                interaction_profile_states.insert(InteractionProfileState::new(
                                    interaction_profile_type.clone(),
                                ))
                            } else {
                                todo!()
                            };

                            //TODO: Device ID persistence
                            let device_id = device_states.insert((
                                ty,
                                DeviceState::default(),
                                interaction_profile_id,
                            ));

                            runtime
                                .driver_response_senders
                                .lock()
                                .get(id)
                                .expect("Could not access driver response channel")
                                .send(Driver2RuntimeEventResponse::DeviceId(device_id.to_bits()))
                                .expect("Driver response channel closed unexpectedly");

                            interaction_profile_states
                                .get_mut(interaction_profile_id)
                                .unwrap()
                                .device_added(device_id, ty);
                        }
                        Driver2RuntimeEvent::Input(event) => {
                            let device_idx = Index::from_bits(event.device).unwrap();
                            let (_, _, interaction_profile_id) =
                                device_states.get(device_idx).unwrap();

                            interaction_profile_states
                                .get_mut(*interaction_profile_id)
                                .unwrap()
                                .update_component(
                                    &event,
                                    &device_states,
                                    |profile_state, user_path, event| {
                                        // println!("{event:?}");

                                        for (session, working_user) in sessions.values_mut() {
                                            if let InputComponentEvent::Cursor(Cursor {
                                                window: Some(window),
                                                ..
                                            }) = event.data
                                            {
                                                let session_window = session.window.lock();
                                                if let Some(session_window) = session_window.deref()
                                                {
                                                    if *session_window != window {
                                                        return;
                                                    }
                                                } else {
                                                    return;
                                                }
                                            }

                                            working_user.on_event(
                                                &profile_state.profile,
                                                user_path,
                                                event,
                                                session,
                                            );
                                        }
                                    },
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
                WorkerThreadEvent::Output(data) => match data {
                    OutputEvent::ReleaseStickyBool { session, action } => {
                        let (session, working_user) = sessions.get_mut(&session).unwrap();

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
                            for listener in session.listeners.read().iter() {
                                listener.handle_event(event, 0);
                            }
                        }
                    }
                },
            }
        }
    })
}
