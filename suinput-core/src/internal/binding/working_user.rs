use std::{borrow::BorrowMut, cell::RefCell, sync::Arc, time::Instant, vec::IntoIter, ops::DerefMut};

use nalgebra::Vector2;
use suinput_types::{
    action::{ActionEvent, ActionEventEnum, ActionListener, ActionStateEnum, ChildActionType},
    event::InputEvent,
    SuPath,
};
use thunderdome::Index;

use crate::{
    action::{Action, ActionTypeEnum},
    action_set::ActionSet,
    internal::{parallel_arena::ParallelArena, types::HashMap},
    types::action_type::{Axis2d, Value},
};
use crate::{
    action::{ActionHierarchyType, ParentActionType},
    internal::{
        device::DeviceState,
        input_events::{InputEventSources, InputEventType},
        interaction_profile::InteractionProfileState,
        paths::InteractionProfilePath,
    },
};

use super::{
    action_hierarchy::{
        handle_axis1d_event, handle_axis2d_event, handle_sticky_bool_event, ParentActionState,
    },
    binding_engine::{processed_binding_layout::ProcessedBindingLayout, WorkingUserInterface},
};

pub struct WorkingUser {
    pub binding_layouts: HashMap<SuPath, RefCell<AttachedBindingLayout>>,

    pub action_states: HashMap<u64, WorkingActionState>,
    pub parent_action_states: HashMap<u64, ParentActionState>,
}

pub struct WorkingActionState {
    pub state: ActionStateEnum,
    pub action_set: u64,
    pub priority: u32,
}

impl WorkingUser {
    //TODO improve how child actions are handled
    pub fn new(action_sets: &Vec<Arc<ActionSet>>) -> Self {
        let action_states = action_sets
            .iter()
            .flat_map(|action_set| {
                action_set
                    .baked_actions
                    .get()
                    .expect("Session created with unbaked action set")
                    .iter()
                    .map(|action| {
                        let default_state = match action.data_type {
                            ActionTypeEnum::Boolean => ActionStateEnum::Boolean(false),
                            ActionTypeEnum::Delta2d => {
                                ActionStateEnum::Delta2d(mint::Vector2 { x: 0., y: 0. })
                            }
                            ActionTypeEnum::Cursor => {
                                ActionStateEnum::Cursor(mint::Vector2 { x: 0., y: 0. })
                            }
                            ActionTypeEnum::Value => ActionStateEnum::Value(0.),
                            ActionTypeEnum::Axis1d => ActionStateEnum::Axis1d(0.),
                            ActionTypeEnum::Axis2d => {
                                ActionStateEnum::Axis2d(mint::Vector2 { x: 0., y: 0. })
                            }
                        };

                        (
                            action.handle,
                            WorkingActionState {
                                state: default_state,
                                action_set: action_set.handle,
                                priority: action_set.default_priority,
                            },
                        )
                    })
            })
            .collect::<HashMap<_, _>>();

        let parent_action_states = action_sets
            .iter()
            .flat_map(|action_set| {
                action_set
                    .baked_actions
                    .get()
                    .expect("Session created with unbaked action set")
                    .iter()
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
            })
            .collect();

        Self {
            binding_layouts: HashMap::new(),
            action_states,
            parent_action_states,
        }
    }

    pub(crate) fn on_event(
        &mut self,
        interaction_profile: &InteractionProfileState,
        user_path: SuPath,
        event: &InputEvent,
        actions: &HashMap<u64, Arc<Action>>,
        callbacks: &mut [Box<dyn ActionListener>],
        devices: &ParallelArena<(DeviceState, Index)>,
    ) {
        if let Some(binding_layout_cell) = self.binding_layouts.get(&interaction_profile.ty.id) {
            let mut attached_binding_layout_ref = binding_layout_cell.borrow_mut();
            let attached_binding_layout = attached_binding_layout_ref.deref_mut();

            let mut wui = WorkingUserInterface {
                path: interaction_profile.ty.id,
                binding_layout_action_states: &mut attached_binding_layout.action_states,
                binding_layouts: &self.binding_layouts,
                action_states: &mut self.action_states,
                parent_action_states: &mut self.parent_action_states,
                callbacks,
                actions,
                interaction_profile_id: interaction_profile.ty.id,
            };

            attached_binding_layout.binding_layout.on_event(user_path, event, interaction_profile, devices, &mut wui);
        }
    }

    pub fn handle_binding_event(
        action_states: &mut HashMap<u64, WorkingActionState>,
        binding_layouts: &HashMap<SuPath, RefCell<AttachedBindingLayout>>,
        parent_action_states: &mut HashMap<u64, ParentActionState>,
        callbacks: &mut [Box<dyn ActionListener>],
        actions: &HashMap<u64, Arc<Action>>,
        action_handle: u64,
        interaction_profile_id: InteractionProfilePath,
        binding_event: ActionStateEnum,
    ) {
        //Store updated binding endpoint action state and decide if an event should be thrown after aggregating against other bindings and then other binding layouts
        let event = match binding_event {
            ActionStateEnum::Boolean(new_binding_state) => UserActions {
                attached_binding_layouts: &binding_layouts,
                action_states: &action_states,
            }
            .aggregate::<bool>(action_handle, new_binding_state, interaction_profile_id)
            .map(|(state, changed)| {
                if changed {
                    action_states.get_mut(&action_handle).unwrap().state =
                        ActionStateEnum::Boolean(state);
                }

                ActionEventEnum::Boolean { state, changed }
            }),
            ActionStateEnum::Delta2d(delta) => {
                //Update accumulated delta
                let working_state = action_states.get_mut(&action_handle).unwrap();
                if let ActionStateEnum::Delta2d(acc_delta) = &mut working_state.state {
                    acc_delta.x += delta.x;
                    acc_delta.y += delta.y;
                }

                Some(ActionEventEnum::Delta2d { delta })
            }
            ActionStateEnum::Cursor(normalized_window_coords) => {
                action_states.get_mut(&action_handle).unwrap().state =
                    ActionStateEnum::Cursor(normalized_window_coords);
                Some(ActionEventEnum::Cursor {
                    normalized_window_coords,
                })
            }
            ActionStateEnum::Value(value) => UserActions {
                attached_binding_layouts: &binding_layouts,
                action_states: &action_states,
            }
            .aggregate::<Value>(action_handle, value, interaction_profile_id)
            .map(|value| {
                action_states.get_mut(&action_handle).unwrap().state =
                    ActionStateEnum::Value(value);
                ActionEventEnum::Value { state: value }
            }),
            //TODO support Axis1d binding endpoints
            ActionStateEnum::Axis1d(_) => todo!(),
            ActionStateEnum::Axis2d(state) => UserActions {
                attached_binding_layouts: &binding_layouts,
                action_states: &action_states,
            }
            .aggregate::<Axis2d>(action_handle, state.into(), interaction_profile_id)
            .map(|state| {
                action_states.get_mut(&action_handle).unwrap().state =
                    ActionStateEnum::Axis2d(state.into());
                ActionEventEnum::Axis2d {
                    state: state.into(),
                }
            }),
        };

        if let Some(event) = event {
            let action = actions.get(&action_handle).unwrap();

            let mut action_handle = action_handle;

            //Child action processing
            //TODO rewrite this to be event driven and lower amount of HashMap indirections
            let event = match &action.hierarchy_type {
                ActionHierarchyType::Parent { ty } => match ty {
                    ParentActionType::StickyBool { .. } => handle_sticky_bool_event(
                        action_handle,
                        parent_action_states,
                        &action_states,
                    ),
                    ParentActionType::Axis1d { .. } => handle_axis1d_event(
                        action_handle,
                        parent_action_states,
                        &action_states,
                    ),
                    ParentActionType::Axis2d { .. } => handle_axis2d_event(
                        action_handle,
                        parent_action_states,
                        &action_states,
                    ),
                    ParentActionType::None => Some(event),
                },
                ActionHierarchyType::Child { parent, ty } => {
                    let parent_handle = parent.upgrade().unwrap().handle;
                    action_handle = parent_handle;

                    match ty {
                        ChildActionType::StickyPress => handle_sticky_bool_event(
                            parent_handle,
                            parent_action_states,
                            &action_states,
                        ),
                        ChildActionType::StickyRelease => handle_sticky_bool_event(
                            parent_handle,
                            parent_action_states,
                            &action_states,
                        ),
                        ChildActionType::StickyToggle => handle_sticky_bool_event(
                            parent_handle,
                            parent_action_states,
                            &action_states,
                        ),
                        ChildActionType::Positive => handle_axis1d_event(
                            parent_handle,
                            parent_action_states,
                            &action_states,
                        ),
                        ChildActionType::Negative => handle_axis1d_event(
                            parent_handle,
                            parent_action_states,
                            &action_states,
                        ),
                        ChildActionType::Up => handle_axis2d_event(
                            parent_handle,
                            parent_action_states,
                            &action_states,
                        ),
                        ChildActionType::Down => handle_axis2d_event(
                            parent_handle,
                            parent_action_states,
                            &action_states,
                        ),
                        ChildActionType::Left => handle_axis2d_event(
                            parent_handle,
                            parent_action_states,
                            &action_states,
                        ),
                        ChildActionType::Right => handle_axis2d_event(
                            parent_handle,
                            parent_action_states,
                            &action_states,
                        ),
                        ChildActionType::Horizontal => handle_axis2d_event(
                            parent_handle,
                            parent_action_states,
                            &action_states,
                        ),
                        ChildActionType::Vertical => handle_axis2d_event(
                            parent_handle,
                            parent_action_states,
                            &action_states,
                        ),
                        _ => todo!(),
                    }
                }
            };

            if let Some(event) = event {
                let event = ActionEvent {
                    action_handle,
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

//TODO investigate moving aggregation into the binding layout so we don't need to keep track of bindings from outside of it
pub struct AttachedBindingLayout {
    binding_layout: ProcessedBindingLayout,
    action_states: HashMap<u64, ActionStateEnum>,
}

impl AttachedBindingLayout {
    pub fn new(binding_layout: ProcessedBindingLayout) -> Self {
        Self {
            action_states: HashMap::new(),
            binding_layout,
        }
    }
}

struct UserActions<'a> {
    attached_binding_layouts: &'a HashMap<InteractionProfilePath, RefCell<AttachedBindingLayout>>,
    action_states: &'a HashMap<u64, WorkingActionState>,
}

impl<'a> InputEventSources for UserActions<'a> {
    type Index = u64;

    type SourceIndex = InteractionProfilePath;

    type Sources = IntoIter<InteractionProfilePath>;

    fn get_state<I: InputEventType>(&self, idx: Self::Index) -> Option<I::Value> {
        self.action_states
            .get(&idx)
            .map(|was| I::from_ase(&was.state))
    }

    fn get_source_state<I: InputEventType>(
        &self,
        idx: Self::Index,
        source_idx: Self::SourceIndex,
    ) -> Option<I::Value> {
        self.attached_binding_layouts
            .get(&source_idx)
            .unwrap()
            .borrow()
            .action_states
            .get(&idx)
            .map(|state| I::from_ase(state))
    }

    fn get_sources<I: InputEventType>(&self, _: Self::Index) -> Self::Sources {
        self.attached_binding_layouts
            .keys()
            .map(|p| *p)
            .collect::<Vec<_>>()
            .into_iter()
    }
}
