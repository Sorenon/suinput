use std::{sync::Arc, cell::RefCell};

use flume::Receiver;
use hashbrown::HashMap;
use suinput_types::{
    action::{ActionListener, ActionStateEnum},
    event::InputEvent,
    SuPath,
};
use thunderdome::{Arena, Index};

use crate::{
    action::Action, action_set::ActionSet, driver_interface::BatchInputUpdate, runtime::Runtime,
    user::User,
};

use super::{
    binding::{
        action_hierarchy::ParentActionState,
        working_user::{AttachedBindingLayout, WorkingUser},
    },
    device::{self, DeviceState},
    device_type::DeviceType,
    interaction_profile::InteractionProfileState,
    parallel_arena::ParallelArena,
};

pub enum Runtime2SessionEvent {
    RegisterDevice { idx: Index, ty: Arc<DeviceType> },
    DisconnectDevice { idx: Index },
    Input(InputEvent),
    BatchInput(BatchInputUpdate),
}

pub struct InnerSession {
    pub user: WorkingUser,

    pub device_states: ParallelArena<(DeviceState, Index)>,
    pub interaction_profile_states: Arena<InteractionProfileState>,

    pub desktop_profile_id: Index,
}

impl InnerSession {
    pub fn new(runtime: &Arc<Runtime>, action_sets: &Vec<Arc<ActionSet>>) -> Self {
        let mut interaction_profile_states = Arena::new();
        let desktop_profile_id = interaction_profile_states.insert(InteractionProfileState::new(
            runtime
                .interaction_profile_types
                .get(runtime.common_paths.desktop)
                .unwrap()
                .clone(),
        ));

        Self {
            user: WorkingUser::new(action_sets),
            device_states: ParallelArena::new(),
            interaction_profile_states,
            desktop_profile_id,
        }
    }

    pub fn sync(
        &mut self,
        runtime: Arc<Runtime>,
        events: &Receiver<Runtime2SessionEvent>,
        user: &Arc<User>,
        actions: &HashMap<u64, Arc<Action>>,
        callbacks: &mut Vec<Box<dyn ActionListener>>,
    ) {
        let working_user = &mut self.user;

        for (profile, binding_layout) in user.new_binding_layouts.lock().drain() {
            working_user
                .binding_layouts
                .insert(profile, RefCell::new(AttachedBindingLayout::new(binding_layout)));
        }

        let mut user_action_states = user.action_states.write();

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

        while let Ok(event) = events.try_recv() {
            match event {
                Runtime2SessionEvent::RegisterDevice { idx, ty } => {
                    self.register_device(&runtime, idx, ty);
                }
                Runtime2SessionEvent::DisconnectDevice { idx } => todo!(),
                Runtime2SessionEvent::Input(input) => {
                    self.input_event(actions, input, callbacks);
                }
                Runtime2SessionEvent::BatchInput(_) => todo!(),
            }
        }
    }

    fn register_device(
        &mut self,
        runtime: &Arc<Runtime>,
        device_idx: Index,
        device_type: Arc<DeviceType>,
    ) {
        let ty = device_type.id;

        let interaction_profile_id = if ty == runtime.common_paths.system_cursor
            || ty == runtime.common_paths.keyboard
            || ty == runtime.common_paths.mouse
        {
            self.desktop_profile_id
        } else if ty == runtime.controller_paths.device_dual_sense {
            let interaction_profile_type = runtime
                .interaction_profile_types
                .get(runtime.controller_paths.interaction_profile_dualsense)
                .unwrap();
            self.interaction_profile_states
                .insert(InteractionProfileState::new(
                    interaction_profile_type.clone(),
                ))
        } else {
            todo!()
        };

        self.device_states.insert_at(
            device_idx,
            (DeviceState::new(device_type), interaction_profile_id),
        );

        self.interaction_profile_states
            .get_mut(interaction_profile_id)
            .unwrap()
            .device_added(device_idx, ty);
    }

    fn input_event(
        &mut self,
        actions: &HashMap<u64, Arc<Action>>,
        event: InputEvent,
        callbacks: &mut [Box<dyn ActionListener>],
    ) {
        let device_idx = Index::from_bits(event.device).unwrap();

        let device = self.device_states.get_mut(device_idx).unwrap();

        if let Some(event) = device.0.process_input_event(event) {
            let (_, interaction_profile_id) = self.device_states.get(device_idx).unwrap();

            /*
                                            let session_window = session.window.lock();
                                            if let Some(session_window) = session_window.deref() {
                                                if *session_window != window {
                                                    return;
                                                }
                                            } else {
                                                return;
                                            }
                                        }

            */

            self.interaction_profile_states
                .get_mut(*interaction_profile_id)
                .unwrap()
                .update_component(
                    &event,
                    &self.device_states,
                    |profile_state, user_path, event, devices| {
                        // println!("{event:?}");

                        self.user.on_interaction_profile_event(
                            &profile_state,
                            user_path,
                            event,
                            actions,
                            callbacks,
                            devices,
                        );
                    },
                );
        }
    }
}
