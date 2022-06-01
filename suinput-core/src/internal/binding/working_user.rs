use std::{collections::HashMap, time::Instant};

use log::warn;
use suinput_types::{
    action::{ActionEvent, ActionEventEnum, ActionStateEnum},
    event::InputEvent,
    SuPath,
};

use crate::{internal::interaction_profile_type::InteractionProfileType, session::Session};

use super::binding_engine::ProcessedBindingLayout;

pub struct WorkingUser {
    pub binding_layouts: HashMap<SuPath, ProcessedBindingLayout>,
    pub action_states: HashMap<u64, ActionStateEnum>,
}

impl WorkingUser {
    pub(crate) fn on_event(
        &mut self,
        interaction_profile: &InteractionProfileType,
        user_path: SuPath,
        event: &InputEvent,
        session: &Session,
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
                                        self.action_states.insert(action_handle, ActionStateEnum::Boolean(false));
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
                        ActionStateEnum::Cursor(normalized_window_coords) => {
                            self.action_states.insert(action_handle, binding_state);
                            ActionEventEnum::Cursor { normalized_window_coords }
                        },
                    };

                    let event = ActionEvent {
                        action_handle,
                        time: Instant::now(),
                        data: event,
                    };
                    for listener in session.listeners.read().iter() {
                        listener.handle_event(event, 0);
                    }
                },
            );
        }
    }
}
