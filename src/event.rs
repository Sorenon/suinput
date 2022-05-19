use crate::{SuPath, Time};

#[derive(Debug)]
pub struct PathFormatError;

#[derive(Debug, Clone, Copy)]
pub struct InputEvent {
    pub device: u64,
    pub path: SuPath,
    pub time: Time,
    pub data: InputComponentEvent,
}

#[derive(Debug, Clone, Copy)]
pub enum InputComponentEvent {
    Button(bool),
    Move2D(Move2D),
    Cursor(Cursor),
}

#[derive(Debug, Clone, Copy)]
pub struct Move2D {
    pub value: (f64, f64),
}

#[derive(Debug, Clone, Copy)]
pub struct Cursor {
    pub normalized_screen_coords: (f64, f64),
}
