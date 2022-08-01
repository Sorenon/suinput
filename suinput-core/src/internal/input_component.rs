use std::time::Instant;

use nalgebra::Vector2;
use suinput_types::event::Cursor;

#[derive(Debug, Clone, Copy)]
pub struct InputComponentData {
    pub last_update_time: Instant,
    pub state: InputComponentState,
}

#[derive(Debug, Clone, Copy)]
pub enum InputComponentState {
    Button(bool),
    Cursor(Cursor),
    Trigger(f32),
    Joystick(Vector2<f32>),
    NonApplicable,
}

#[derive(Debug, Clone, Copy)]
pub enum InputComponentType {
    Button,
    Trigger,
    Move2D,
    Cursor,
    Joystick,
    Gyro(bool),
    Accel,
}
