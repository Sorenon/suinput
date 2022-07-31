use suinput_types::action::{ActionEventEnum, ChildActionType};

pub trait CompoundActionState: Send {
    fn on_action_event(
        &mut self,
        event: &ActionEventEnum,
        child_type: ChildActionType,
    ) -> Option<ActionEventEnum>;

    //TODO method for updating by polling state instead of event
}

#[derive(Default)]
pub struct StickyBoolState {
    combined_state: bool,

    parent_state: bool,

    stuck: bool,

    sticky_press_state: bool,
    release_state: bool,
}

impl CompoundActionState for StickyBoolState {
    fn on_action_event(
        &mut self,
        event: &ActionEventEnum,
        child_type: ChildActionType,
    ) -> Option<ActionEventEnum> {
        let (new_child_state, changed) = match *event {
            ActionEventEnum::Boolean { state, changed } => (state, changed),
            _ => panic!(),
        };

        let new_state = match child_type {
            ChildActionType::Parent => {
                if new_child_state {
                    self.parent_state = true;
                    Some(true)
                } else if self.parent_state && !self.sticky_press_state && !self.stuck {
                    self.parent_state = false;
                    Some(false)
                } else {
                    None
                }
            }
            ChildActionType::StickyPress => {
                if new_child_state {
                    self.sticky_press_state = true;
                    self.stuck = !self.release_state;
                    Some(true)
                } else if self.sticky_press_state && !self.parent_state && !self.stuck {
                    self.sticky_press_state = false;
                    Some(false)
                } else {
                    None
                }
            }
            ChildActionType::StickyToggle => {
                if new_child_state && changed {
                    self.stuck = !self.stuck;

                    if self.stuck {
                        Some(true)
                    } else if !self.parent_state && !self.sticky_press_state && self.combined_state
                    {
                        Some(false)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            ChildActionType::StickyRelease => {
                if new_child_state && changed {
                    self.stuck = false;

                    if !self.parent_state && !self.sticky_press_state && self.combined_state {
                        Some(false)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => panic!(),
        };

        if let Some(new_state) = new_state {
            let out = ActionEventEnum::Boolean {
                state: new_state,
                changed: new_state != self.combined_state,
            };
            self.combined_state = new_state;
            Some(out)
        } else {
            None
        }
    }
}
