use mint::{Vector2, Vector3};
use thiserror::Error;

use crate::{SuPath, Time, WindowHandle};

#[derive(Debug, Error)]
#[error("Path Format Error")]
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
    Trigger(f32),
    Joystick(Vector2<f32>),
    Move2D(Vector2<f64>),

    Cursor(Cursor),
    Gyro(Vector3<f32>),
    Accel(Vector3<f32>),
}

#[derive(Debug, Clone, Copy)]
pub struct Cursor {
    pub normalized_screen_coords: (f64, f64),
    pub window: Option<WindowHandle>,
}

impl InputComponentEvent {
    pub fn get_gyro(self) -> Result<Vector3<f32>, ()> {
        match self {
            InputComponentEvent::Gyro(state) => Ok(state),
            _ => Err(()),
        }
    }

    pub fn get_accel(self) -> Result<Vector3<f32>, ()> {
        match self {
            InputComponentEvent::Accel(state) => Ok(state),
            _ => Err(()),
        }
    }
}
