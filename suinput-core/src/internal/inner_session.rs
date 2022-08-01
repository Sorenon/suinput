use std::{cell::RefCell, sync::Arc, time::Instant};

use flume::Receiver;
use hashbrown::{HashMap, HashSet};
use suinput_types::{
    action::{ActionEvent, ActionListener, ActionStateEnum},
    event::InputEvent,
};
use thunderdome::Index;

use crate::{
    action::Action, action_set::ActionSet, driver_interface::BatchInputUpdate, runtime::Runtime,
    user::User,
};

use super::{
    binding::working_user::{AttachedBindingLayout, WorkingUser},
    device::DeviceState,
    device_type::DeviceType,
    interaction_profile::InteractionProfileState,
    parallel_arena::ParallelArena,
    paths::InteractionProfilePath,
};

pub enum Runtime2SessionEvent {
    RegisterDevice { idx: Index, ty: Arc<DeviceType> },
    DisconnectDevice { idx: Index },
    Input(InputEvent),
    BatchInput(BatchInputUpdate),
}

pub enum SessionActionEvent {
    Unstick { action: u64 },
}

pub struct InnerSession {
    pub user: WorkingUser,

    //TODO move to working user
    pub active_action_sets: HashSet<u64>,
    pub old_active_action_sets: HashSet<u64>,

    pub default_interaction_profiles: HashMap<InteractionProfilePath, InteractionProfileState>,
    pub device_states: ParallelArena<(DeviceState, InteractionProfilePath)>,
}

impl InnerSession {
    pub fn new(runtime: &Arc<Runtime>, action_sets: &HashMap<u64, Arc<ActionSet>>) -> Self {
        let mut default_interaction_profiles = HashMap::new();
        default_interaction_profiles.insert(
            runtime.common_paths.desktop,
            InteractionProfileState::new(
                runtime
                    .interaction_profile_types
                    .get(runtime.common_paths.desktop)
                    .unwrap()
                    .clone(),
            ),
        );
        default_interaction_profiles.insert(
            runtime.controller_paths.interaction_profile_dualsense,
            InteractionProfileState::new(
                runtime
                    .interaction_profile_types
                    .get(runtime.controller_paths.interaction_profile_dualsense)
                    .unwrap()
                    .clone(),
            ),
        );

        Self {
            user: WorkingUser::new(action_sets),
            device_states: ParallelArena::new(),
            default_interaction_profiles,
            active_action_sets: HashSet::new(),
            old_active_action_sets: HashSet::new(),
        }
    }

    pub fn sync<'a>(
        &mut self,
        runtime: Arc<Runtime>,
        new_active_action_sets: impl Iterator<Item = &'a Arc<ActionSet>>,
        action_sets: &HashMap<u64, Arc<ActionSet>>,
        action_events: &Receiver<SessionActionEvent>,
        events: &Receiver<Runtime2SessionEvent>,
        user: &Arc<User>,
        actions: &HashMap<u64, Arc<Action>>,
        callbacks: &mut Vec<Box<dyn ActionListener>>,
    ) {
        std::mem::swap(
            &mut self.active_action_sets,
            &mut self.old_active_action_sets,
        );

        self.active_action_sets.clear();
        self.active_action_sets
            .extend(new_active_action_sets.map(|set| (set.handle)));

        let working_user = &mut self.user;

        if self.active_action_sets != self.old_active_action_sets {
            let mut disabling = self
                .old_active_action_sets
                .difference(&self.active_action_sets)
                .map(|handle| action_sets.get(handle).unwrap())
                .collect::<Vec<_>>();

            let mut enabling = self
                .active_action_sets
                .difference(&self.old_active_action_sets)
                .map(|handle| action_sets.get(handle).unwrap())
                .collect::<Vec<_>>();

            disabling.sort_by(|left, right| left.handle.cmp(&right.handle));
            enabling.sort_by(|left, right| left.handle.cmp(&right.handle).reverse());

            working_user.change_enabled_action_sets(
                callbacks,
                actions,
                &self.default_interaction_profiles,
                &disabling,
                &enabling,
                &self.active_action_sets,
            );
        }

        while let Ok(event) = action_events.try_recv() {
            match event {
                SessionActionEvent::Unstick { action } => {
                    if let Some(event) = working_user
                        .compound_action_states
                        .get_mut(&action)
                        .unwrap()
                        .handle_event()
                    {
                        let event = ActionEvent {
                            action_handle: action,
                            time: Instant::now(),
                            data: event,
                        };
                        for listener in callbacks.iter_mut() {
                            listener.handle_event(event, 0);
                        }
                    }
                }
            }
        }

        for (profile, binding_layout) in user.new_binding_layouts.lock().drain() {
            working_user.binding_layouts.insert(
                profile,
                RefCell::new(AttachedBindingLayout::new(binding_layout)),
            );
        }

        let mut user_action_states = user.action_states.write();

        for (path, working_action_state) in working_user.action_states.iter_mut() {
            let action_state = &mut working_action_state.state;
            if let Some(compound_state) = working_user.compound_action_states.get(path) {
                user_action_states.insert(*path, compound_state.get_state());
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
                Runtime2SessionEvent::DisconnectDevice { idx: _ } => todo!(),
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
            runtime.common_paths.desktop
        } else if ty == runtime.controller_paths.device_dual_sense {
            runtime.controller_paths.interaction_profile_dualsense
        } else {
            todo!()
        };

        self.device_states.insert_at(
            device_idx,
            (DeviceState::new(device_type), interaction_profile_id),
        );

        self.default_interaction_profiles
            .get_mut(&interaction_profile_id)
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

            self.default_interaction_profiles
                .get_mut(interaction_profile_id)
                .unwrap()
                .update_component(
                    &event,
                    &self.device_states,
                    |profile_state, user_path, event, devices| {
                        // println!("{event:?}");

                        self.user.on_interaction_profile_event(
                            *interaction_profile_id,
                            profile_state,
                            &self.active_action_sets,
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
