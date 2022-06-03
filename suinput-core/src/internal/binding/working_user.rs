use std::{
    collections::{hash_map::Entry, HashMap},
    time::Instant,
};

use log::warn;
use suinput_types::{
    action::{ActionEvent, ActionEventEnum, ActionStateEnum, ChildActionType},
    event::InputEvent,
    SuPath,
};

use crate::{
    action::{ActionHierarchyType, ParentActionType},
    internal::{interaction_profile_type::InteractionProfileType, paths::InteractionProfilePath},
    session::Session,
};

use super::binding_engine::ProcessedBindingLayout;

pub struct WorkingUser {
    pub binding_layouts: HashMap<SuPath, ProcessedBindingLayout>,

    pub action_states: HashMap<u64, ActionStateEnum>,
    pub parent_action_states: HashMap<u64, ParentActionState>,
}

impl WorkingUser {
    pub(crate) fn on_event(
        &mut self,
        interaction_profile: &InteractionProfileType,
        user_path: SuPath,
        event: &InputEvent,
        session: &Session,
    ) {
        let mut binding_events = Vec::new();

        if let Some(binding_layout) = self.binding_layouts.get_mut(&interaction_profile.id) {
            binding_layout.on_event(
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
        let event = match binding_event {
            ActionStateEnum::Boolean(new_binding_state) => self.handle_boolean_binding_event(
                action_handle,
                binding_index,
                interaction_profile_id,
                new_binding_state,
            ),
            ActionStateEnum::Delta2D(delta) => {
                //Update accumulated delta
                match self.action_states.entry(action_handle) {
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

                Some(ActionEventEnum::Delta2D { delta })
            }
            ActionStateEnum::Cursor(normalized_window_coords) => {
                self.action_states.insert(action_handle, binding_event);
                Some(ActionEventEnum::Cursor {
                    normalized_window_coords,
                })
            }
        };

        if let Some(event) = event {
            let action = session.actions.get(&action_handle).unwrap();

            let mut action_handle = action_handle;

            let event = match &action.hierarchy_type {
                ActionHierarchyType::Parent { ty } => match ty {
                    ParentActionType::StickyBool { .. } => {
                        self.handle_sticky_bool_event(action_handle)
                    }
                    ParentActionType::Axis1D { .. } => todo!(),
                    ParentActionType::None => Some(event),
                },
                ActionHierarchyType::Child { parent, ty } => {
                    let parent_handle = parent.upgrade().unwrap().handle;
                    action_handle = parent_handle;

                    match ty {
                        ChildActionType::StickyPress => {
                            self.handle_sticky_bool_event(parent_handle)
                        }
                        ChildActionType::StickyRelease => {
                            self.handle_sticky_bool_event(parent_handle)
                        }
                        ChildActionType::StickyToggle => {
                            self.handle_sticky_bool_event(parent_handle)
                        }
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
                for listener in session.listeners.read().iter() {
                    listener.handle_event(event, 0);
                }
            }
        }
    }

    pub fn handle_sticky_bool_event(&mut self, parent: u64) -> Option<ActionEventEnum> {
        if let Some(ParentActionState::StickyBool {
            combined_state,
            stuck,
            press,
            release,
            toggle,
        }) = self.parent_action_states.get_mut(&parent)
        {
            match (
                self.action_states
                    .get(&parent)
                    .unwrap_or(&ActionStateEnum::Boolean(false)),
                self.action_states
                    .get(press)
                    .unwrap_or(&ActionStateEnum::Boolean(false)),
                self.action_states
                    .get(release)
                    .unwrap_or(&ActionStateEnum::Boolean(false)),
                self.action_states
                    .get(toggle)
                    .unwrap_or(&ActionStateEnum::Boolean(false)),
            ) {
                (
                    ActionStateEnum::Boolean(parent),
                    ActionStateEnum::Boolean(press),
                    ActionStateEnum::Boolean(release),
                    ActionStateEnum::Boolean(toggle),
                ) => {
                    if *toggle {
                        *stuck = !*stuck;
                    }

                    if *stuck && *release || *press {
                        *stuck = *press;
                    }

                    let last_state = *combined_state;

                    *combined_state = *parent || *stuck;

                    if last_state != *combined_state {
                        Some(ActionEventEnum::Boolean {
                            state: *combined_state,
                            changed: true,
                        })
                    } else {
                        None
                    }
                }
                _ => panic!(),
            }
        } else {
            panic!()
        }
    }

    fn handle_boolean_binding_event(
        &mut self,
        action_handle: u64,
        binding_index: usize,
        interaction_profile_id: InteractionProfilePath,
        new_binding_state: bool,
    ) -> Option<ActionEventEnum> {
        let old_action_state = match self.action_states.get(&action_handle) {
            Some(ActionStateEnum::Boolean(action_state)) => *action_state,
            _ => false,
        };

        if new_binding_state {
            self.action_states
                .insert(action_handle, ActionStateEnum::Boolean(true));
            Some(ActionEventEnum::Boolean {
                state: true,
                changed: new_binding_state != old_action_state,
            })
        } else {
            if old_action_state {
                let binding_layout = self.binding_layouts.get(&interaction_profile_id).unwrap();

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
                    self.action_states
                        .insert(action_handle, ActionStateEnum::Boolean(false));
                    Some(ActionEventEnum::Boolean {
                        state: false,
                        changed: true,
                    })
                } else {
                    return None;
                }
            } else {
                warn!("Somehow fired a false event on an action which is already false");
                return None;
            }
        }
    }
}

pub enum ParentActionState {
    StickyBool {
        combined_state: bool,
        stuck: bool,

        press: u64,
        release: u64,
        toggle: u64,
    },
}
