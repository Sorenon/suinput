use std::{
    borrow::BorrowMut, cell::RefCell, ops::DerefMut, sync::Arc, time::Instant, vec::IntoIter,
};

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
    internal::{
        compound_action::{CompoundActionState, CompoundAxis1dState, StickyBoolState, CompoundAxis2dState},
        parallel_arena::ParallelArena,
        types::HashMap,
    },
    types::action_type::{Axis2d, Value},
};
use crate::{
    action::{ActionCompoundType, ParentActionType},
    internal::{
        device::DeviceState,
        input_events::{InputEventSources, InputEventType},
        interaction_profile::InteractionProfileState,
        paths::InteractionProfilePath,
    },
};

use super::binding_engine::{
    processed_binding_layout::ProcessedBindingLayout, WorkingUserInterface,
};

pub struct WorkingUser {
    pub binding_layouts: HashMap<SuPath, RefCell<AttachedBindingLayout>>,

    pub action_states: HashMap<u64, WorkingActionState>,
    pub compound_action_states: HashMap<u64, Box<dyn CompoundActionState>>,
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

        let compound_action_states = action_sets
            .iter()
            .flat_map(|action_set| {
                action_set
                    .baked_actions
                    .get()
                    .expect("Session created with unbaked action set")
                    .iter()
                    .filter_map(|action| match &action.compound {
                        ActionCompoundType::Parent {
                            ty: ParentActionType::StickyBool { .. },
                        } => Some((
                            action.handle,
                            Box::new(StickyBoolState::default()) as Box<dyn CompoundActionState>,
                        )),
                        ActionCompoundType::Parent {
                            ty: ParentActionType::Axis1d { .. },
                        } => Some((
                            action.handle,
                            Box::new(CompoundAxis1dState::default())
                                as Box<dyn CompoundActionState>,
                        )),
                        ActionCompoundType::Parent {
                            ty: ParentActionType::Axis2d { .. },
                        } => Some((
                            action.handle,
                            Box::new(CompoundAxis2dState::default())
                                as Box<dyn CompoundActionState>,
                        )),
                        _ => None,
                    })
            })
            .collect();

        Self {
            binding_layouts: HashMap::new(),
            action_states,
            compound_action_states,
        }
    }

    pub(crate) fn on_interaction_profile_event(
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
                binding_layout_action_states: &mut attached_binding_layout.action_states,
                binding_layouts: &self.binding_layouts,
                action_states: &mut self.action_states,
                compound_action_states: &mut self.compound_action_states,
                callbacks,
                actions,
                interaction_profile_id: interaction_profile.ty.id,
            };

            attached_binding_layout
                .binding_layout
                .handle_component_event(user_path, event, interaction_profile, devices, &mut wui);
        }
    }

    pub fn handle_binding_event(
        action_states: &mut HashMap<u64, WorkingActionState>,
        binding_layouts: &HashMap<SuPath, RefCell<AttachedBindingLayout>>,
        compound_action_states: &mut HashMap<u64, Box<dyn CompoundActionState>>,
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

            let mut out_action = action_handle;

            //Child action processing
            //TODO rewrite this to be event driven and lower amount of HashMap indirections
            let event = match &action.compound {
                ActionCompoundType::Parent { .. } => {
                    let compound_state = compound_action_states.get_mut(&action_handle).unwrap();
                    compound_state.on_action_event(&event, ChildActionType::Parent)
                }
                ActionCompoundType::Child { parent, ty } => {
                    let parent_handle = parent.upgrade().unwrap().handle;
                    out_action = parent_handle;

                    let compound_state = compound_action_states.get_mut(&parent_handle).unwrap();
                    compound_state.on_action_event(&event, *ty)
                }
                ActionCompoundType::None => Some(event),
            };

            if let Some(event) = event {
                let event = ActionEvent {
                    action_handle: out_action,
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
