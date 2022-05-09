use std::collections::HashMap;

use crate::{interaction_profile::InteractionProfile, Path, Time, Vec2D};

#[derive(Debug, Clone, Copy)]
pub(crate) enum InputEvent {
    Button { state: bool, changed: bool },
    Analog { state: f32 },
    Joystick { state: Vec2D },
    Cursor { state: Vec2D },
}

pub trait InputMapper {
    
}