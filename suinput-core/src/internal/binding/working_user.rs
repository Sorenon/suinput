use std::{
    collections::{hash_map::Entry, HashMap},
    time::Instant,
    vec::IntoIter,
};

use suinput_types::{
    action::{ActionEvent, ActionEventEnum, ActionStateEnum, ChildActionType},
    event::InputEvent,
    SuPath,
};

use crate::{
    action::{ActionHierarchyType, ParentActionType},
    internal::{
        input_events::{Axis2d, InputEventSources, InputEventType, Value},
        interaction_profile_type::InteractionProfileType,
        paths::InteractionProfilePath,
    },
    session::Session,
};

use super::{
    action_hierarchy::{
        handle_axis1d_event, handle_axis2d_event, handle_sticky_bool_event, ParentActionState,
    },
    binding_engine::ProcessedBindingLayout,
};

pub struct WorkingUser {
    pub binding_layouts: HashMap<SuPath, AttachedBindingLayout>,

    pub action_states: ActionStates,
    pub parent_action_states: HashMap<u64, ParentActionState>,
}

//TODO combine action states across binding layouts
impl WorkingUser {
    pub fn new(parent_action_states: HashMap<u64, ParentActionState>) -> Self {
        Self {
            binding_layouts: HashMap::new(),
            action_states: ActionStates {
                states: HashMap::new(),
            },
            parent_action_states,
        }
    }

    pub(crate) fn on_event(
        &mut self,
        interaction_profile: &InteractionProfileType,
        user_path: SuPath,
        event: &InputEvent,
        session: &Session,
    ) {
        let mut binding_events = Vec::new();

        if let Some(binding_layout) = self.binding_layouts.get_mut(&interaction_profile.id) {
            binding_layout.binding_layout.on_event(
                user_path,
                event,
                |action_handle, binding_index, &binding_event| {
                    binding_events.push((
                        action_handle,
                        binding_index,
                        interaction_profile.id,
                        binding_event,
                    ))
                },
            );
        }

        for (action_handle, binding_index, interaction_profile_id, binding_event) in binding_events
        {
            self.handle_binding_event(
                session,
                action_handle,
                binding_index,
                interaction_profile_id,
                binding_event,
            );
        }
    }

    fn handle_binding_event(
        &mut self,
        session: &Session,
        action_handle: u64,
        binding_index: usize,
        interaction_profile_id: InteractionProfilePath,
        binding_event: ActionStateEnum,
    ) {
        let binding_layout = self
            .binding_layouts
            .get_mut(&interaction_profile_id)
            .unwrap();

        let event = match binding_event {
            ActionStateEnum::Boolean(new_binding_state) => {
                match binding_layout.aggregate::<bool>(
                    action_handle,
                    new_binding_state,
                    binding_index,
                ) {
                    Some((state, changed)) => {
                        if changed {
                            binding_layout
                                .action_states
                                .insert(action_handle, ActionStateEnum::Boolean(state));
                        }

                        UserActions {
                            attached_binding_layouts: &self.binding_layouts,
                            action_states: &self.action_states,
                        }
                        .aggregate::<bool>(action_handle, new_binding_state, interaction_profile_id)
                        .map(|(state, changed)| {
                            if changed {
                                self.action_states
                                    .insert(action_handle, ActionStateEnum::Boolean(state));
                            }

                            ActionEventEnum::Boolean { state, changed }
                        })
                    }
                    None => None,
                }
            }
            ActionStateEnum::Delta2D(delta) => {
                //Update accumulated delta
                self.action_states.append_delta2d(action_handle, delta);
                Some(ActionEventEnum::Delta2D { delta })
            }
            ActionStateEnum::Cursor(normalized_window_coords) => {
                self.action_states.insert(action_handle, binding_event);
                Some(ActionEventEnum::Cursor {
                    normalized_window_coords,
                })
            }
            ActionStateEnum::Value(value) => {
                match binding_layout.aggregate::<Value>(action_handle, value, binding_index) {
                    Some(value) => {
                        binding_layout
                            .action_states
                            .insert(action_handle, ActionStateEnum::Value(value));

                        UserActions {
                            attached_binding_layouts: &self.binding_layouts,
                            action_states: &self.action_states,
                        }
                        .aggregate::<Value>(action_handle, value, interaction_profile_id)
                        .map(|value| {
                            self.action_states
                                .insert(action_handle, ActionStateEnum::Value(value));
                            ActionEventEnum::Value { state: value }
                        })
                    }
                    None => None,
                }
            }
            ActionStateEnum::Axis1d(_) => todo!(),
            ActionStateEnum::Axis2d(state) => {
                match binding_layout.aggregate::<Axis2d>(action_handle, state.into(), binding_index)
                {
                    Some(state) => {
                        binding_layout
                            .action_states
                            .insert(action_handle, ActionStateEnum::Axis2d(state.into()));

                        UserActions {
                            attached_binding_layouts: &self.binding_layouts,
                            action_states: &self.action_states,
                        }
                        .aggregate::<Axis2d>(action_handle, state.into(), interaction_profile_id)
                        .map(|state| {
                            self.action_states
                                .insert(action_handle, ActionStateEnum::Axis2d(state.into()));
                            ActionEventEnum::Axis2d {
                                state: state.into(),
                            }
                        })
                    }
                    None => None,
                }
            }
        };

        if let Some(event) = event {
            let action = session.actions.get(&action_handle).unwrap();

            let mut action_handle = action_handle;

            let event = match &action.hierarchy_type {
                ActionHierarchyType::Parent { ty } => match ty {
                    ParentActionType::StickyBool { .. } => handle_sticky_bool_event(
                        action_handle,
                        &mut self.parent_action_states,
                        &self.action_states,
                    ),
                    ParentActionType::Axis1d { .. } => handle_axis1d_event(
                        action_handle,
                        &mut self.parent_action_states,
                        &self.action_states,
                    ),
                    ParentActionType::Axis2d { .. } => handle_axis2d_event(
                        action_handle,
                        &mut self.parent_action_states,
                        &self.action_states,
                    ),
                    ParentActionType::None => Some(event),
                },
                ActionHierarchyType::Child { parent, ty } => {
                    let parent_handle = parent.upgrade().unwrap().handle;
                    action_handle = parent_handle;

                    match ty {
                        ChildActionType::StickyPress => handle_sticky_bool_event(
                            parent_handle,
                            &mut self.parent_action_states,
                            &self.action_states,
                        ),
                        ChildActionType::StickyRelease => handle_sticky_bool_event(
                            parent_handle,
                            &mut self.parent_action_states,
                            &self.action_states,
                        ),
                        ChildActionType::StickyToggle => handle_sticky_bool_event(
                            parent_handle,
                            &mut self.parent_action_states,
                            &self.action_states,
                        ),
                        ChildActionType::Positive => handle_axis1d_event(
                            parent_handle,
                            &mut self.parent_action_states,
                            &self.action_states,
                        ),
                        ChildActionType::Negative => handle_axis1d_event(
                            parent_handle,
                            &mut self.parent_action_states,
                            &self.action_states,
                        ),
                        ChildActionType::Up => handle_axis2d_event(
                            parent_handle,
                            &mut self.parent_action_states,
                            &self.action_states,
                        ),
                        ChildActionType::Down => handle_axis2d_event(
                            parent_handle,
                            &mut self.parent_action_states,
                            &self.action_states,
                        ),
                        ChildActionType::Left => handle_axis2d_event(
                            parent_handle,
                            &mut self.parent_action_states,
                            &self.action_states,
                        ),
                        ChildActionType::Right => handle_axis2d_event(
                            parent_handle,
                            &mut self.parent_action_states,
                            &self.action_states,
                        ),
                        ChildActionType::Horizontal => handle_axis2d_event(
                            parent_handle,
                            &mut self.parent_action_states,
                            &self.action_states,
                        ),
                        ChildActionType::Vertical => handle_axis2d_event(
                            parent_handle,
                            &mut self.parent_action_states,
                            &self.action_states,
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
                for listener in session.listeners.write().iter_mut() {
                    listener.handle_event(event, 0);
                }
            }
        }
    }
}

pub struct AttachedBindingLayout {
    binding_layout: ProcessedBindingLayout,
    binding_states: Vec<(ActionStateEnum, u64)>,
    bindings_for_action: HashMap<u64, Vec<usize>>,
    action_states: HashMap<u64, ActionStateEnum>,
}

impl AttachedBindingLayout {
    pub fn new(binding_layout: ProcessedBindingLayout) -> Self {
        Self {
            binding_states: binding_layout
                .bindings_index
                .iter()
                .map(|(_, b, c)| (*b, *c))
                .collect(),
            bindings_for_action: binding_layout.bindings_for_action.clone(),
            action_states: HashMap::new(),
            binding_layout,
        }
    }
}

pub struct ActionStates {
    pub states: HashMap<u64, ActionStateEnum>,
}

impl ActionStates {
    pub fn get_bool(&self, handle: u64) -> Option<bool> {
        match self.states.get(&handle) {
            Some(ActionStateEnum::Boolean(state)) => Some(*state),
            None => None,
            _ => panic!(), //TODO should this panic
        }
    }

    pub fn append_delta2d(&mut self, handle: u64, delta: (f64, f64)) {
        match self.states.entry(handle) {
            Entry::Occupied(mut entry) => match entry.get_mut() {
                ActionStateEnum::Delta2D(acc) => {
                    acc.0 += delta.0;
                    acc.1 += delta.1;
                }
                _ => panic!(),
            },
            Entry::Vacant(entry) => {
                entry.insert(ActionStateEnum::Delta2D(delta));
            }
        }
    }

    pub fn insert(&mut self, handle: u64, val: ActionStateEnum) {
        self.states.insert(handle, val);
    }
}

impl InputEventSources for AttachedBindingLayout {
    type Index = u64;

    type SourceIndex = usize;

    type Sources = IntoIter<Self::SourceIndex>;

    fn get_state<I: InputEventType>(&self, idx: Self::Index) -> Option<I::Value> {
        self.action_states.get(&idx).map(|state| I::from_ase(state))
    }

    fn get_source_state<I: InputEventType>(
        &self,
        _: Self::Index,
        source_idx: Self::SourceIndex,
    ) -> Option<I::Value> {
        Some(I::from_ase(&self.binding_states.get(source_idx).unwrap().0))
    }

    fn get_sources<I: InputEventType>(&self, idx: Self::Index) -> Self::Sources {
        self.bindings_for_action
            .get(&idx)
            .unwrap()
            .clone()
            .into_iter()
    }
}

struct UserActions<'a> {
    attached_binding_layouts: &'a HashMap<InteractionProfilePath, AttachedBindingLayout>,
    action_states: &'a ActionStates,
}

impl<'a> InputEventSources for UserActions<'a> {
    type Index = u64;

    type SourceIndex = InteractionProfilePath;

    type Sources = IntoIter<InteractionProfilePath>;

    fn get_state<I: InputEventType>(&self, idx: Self::Index) -> Option<I::Value> {
        self.action_states
            .states
            .get(&idx)
            .map(|state| I::from_ase(state))
    }

    fn get_source_state<I: InputEventType>(
        &self,
        idx: Self::Index,
        source_idx: Self::SourceIndex,
    ) -> Option<I::Value> {
        self.attached_binding_layouts
            .get(&source_idx)
            .unwrap()
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
