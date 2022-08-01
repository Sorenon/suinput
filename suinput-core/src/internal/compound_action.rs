use nalgebra::Vector2;
use suinput_types::action::{ActionEventEnum, ActionStateEnum, ChildActionType};

pub trait CompoundActionState: Send {
    fn on_action_event(
        &mut self,
        event: &ActionEventEnum,
        child_type: ChildActionType,
    ) -> Option<ActionEventEnum>;

    fn get_state(&self) -> ActionStateEnum;

    fn handle_event(&mut self) -> Option<ActionEventEnum> {
        None
    }

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

    fn get_state(&self) -> ActionStateEnum {
        ActionStateEnum::Boolean(self.combined_state)
    }

    fn handle_event(&mut self) -> Option<ActionEventEnum> {
        if self.stuck {
            self.stuck = self.sticky_press_state;
            if self.combined_state&& !self.parent_state && !self.stuck {
                Some(ActionEventEnum::Boolean { state: false, changed: true })
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Default)]
pub struct CompoundAxis1dState {
    state: f32,

    parent: f32,
    positive: f32,
    negative: f32,
}

impl CompoundActionState for CompoundAxis1dState {
    fn on_action_event(
        &mut self,
        event: &ActionEventEnum,
        child_type: ChildActionType,
    ) -> Option<ActionEventEnum> {
        match child_type {
            ChildActionType::Parent => {
                self.parent = match *event {
                    ActionEventEnum::Axis1d { state } => state,
                    _ => panic!(),
                };
            }
            ChildActionType::Positive => {
                self.positive = match *event {
                    ActionEventEnum::Value { state } => state,
                    _ => panic!(),
                };
            }
            ChildActionType::Negative => {
                self.negative = match *event {
                    ActionEventEnum::Value { state } => state,
                    _ => panic!(),
                };
            }
            _ => panic!(),
        }

        let new_state = (self.positive - self.negative + self.parent).clamp(-1., 1.);
        if new_state.abs() != self.state.abs() {
            self.state = new_state;
            Some(ActionEventEnum::Axis1d { state: new_state })
        } else {
            None
        }
    }

    fn get_state(&self) -> ActionStateEnum {
        ActionStateEnum::Axis1d(self.state)
    }
}

#[derive(Default)]
pub struct CompoundAxis2dState {
    state: Vector2<f32>,

    parent: Vector2<f32>,
    horizontal: f32,
    vertical: f32,
    up: f32,
    down: f32,
    left: f32,
    right: f32,
}

impl CompoundActionState for CompoundAxis2dState {
    fn on_action_event(
        &mut self,
        event: &ActionEventEnum,
        child_type: ChildActionType,
    ) -> Option<ActionEventEnum> {
        match child_type {
            ChildActionType::Parent => {
                self.parent = match *event {
                    ActionEventEnum::Axis2d { state } => state.into(),
                    _ => panic!(),
                };
            }
            ChildActionType::Horizontal => {
                self.horizontal = match *event {
                    ActionEventEnum::Axis1d { state } => state,
                    _ => panic!(),
                };
            }
            ChildActionType::Vertical => {
                self.vertical = match *event {
                    ActionEventEnum::Axis1d { state } => state,
                    _ => panic!(),
                };
            }
            ChildActionType::Up => {
                self.up = match *event {
                    ActionEventEnum::Value { state } => state,
                    _ => panic!(),
                };
            }
            ChildActionType::Down => {
                self.down = match *event {
                    ActionEventEnum::Value { state } => state,
                    _ => panic!(),
                };
            }
            ChildActionType::Left => {
                self.left = match *event {
                    ActionEventEnum::Value { state } => state,
                    _ => panic!(),
                };
            }
            ChildActionType::Right => {
                self.right = match *event {
                    ActionEventEnum::Value { state } => state,
                    _ => panic!(),
                };
            }
            _ => panic!(),
        }

        let new_state = Vector2::new(
            (self.right - self.left + self.horizontal + self.parent.x).clamp(-1., 1.),
            (self.up - self.down + self.vertical + self.parent.y).clamp(-1., 1.),
        );

        if new_state != self.state {
            self.state = new_state;

            Some(ActionEventEnum::Axis2d {
                state: new_state.into(),
            })
        } else {
            None
        }
    }

    fn get_state(&self) -> ActionStateEnum {
        ActionStateEnum::Axis2d(self.state.into())
    }
}
