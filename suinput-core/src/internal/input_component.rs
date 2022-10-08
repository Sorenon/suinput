use std::time::Instant;

use nalgebra::Vector2;

#[derive(Debug, Clone, Copy)]
pub struct InputComponentData {
    pub last_update_time: Instant,
    pub state: InputComponentState,
}

#[derive(Debug, Clone, Copy)]
pub enum InputComponentState {
    Button(bool),
    Trigger(f32),
    Joystick(Vector2<f32>),
    NonApplicable,
}

#[derive(Debug, Clone, Copy)]
pub enum InputComponentType {
    Button,
    Trigger,
    Move2D,
    Joystick,
    Gyro(bool),
    Accel,
}

#[derive(Debug, Clone, Copy)]
pub enum InternalActionState {
    Boolean(bool),
    Value(f32),
    Axis1d(f32),
    Axis2d(Vector2<f32>),
    NonApplicable,
}
