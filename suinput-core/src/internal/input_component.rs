use std::time::Instant;

use suinput_types::event::Cursor;

#[derive(Debug)]
pub struct InputComponentData {
    pub last_update_time: Instant,
    pub state: InputComponentState,
}

#[derive(Debug)]
pub enum InputComponentState {
    Button(bool),
    Cursor(Cursor),
    NonApplicable,
}

#[derive(Debug, Clone, Copy)]
pub enum InputComponentType {
    Button,
    // Trigger,
    Move2D,
    Cursor,
}
