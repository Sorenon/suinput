use std::time::Instant;

use mint::Vector2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionCreateInfo {
    Boolean {
        sticky: bool,
    },
    Delta2D,
    Cursor,
    Value,
    Axis1d {
        positive: Option<String>,
        negative: Option<String>,
    },
    Axis2d {
        up: Option<String>,
        down: Option<String>,
        left: Option<String>,
        right: Option<String>,
        vertical: Option<String>,
        horizontal: Option<String>,
    },
}

pub trait ActionListener: Send + Sync {
    fn handle_event(&mut self, event: ActionEvent, user: u64);
}

#[derive(Debug, Clone, Copy)]
pub struct ActionEvent {
    pub action_handle: u64,
    pub time: Instant,
    pub data: ActionEventEnum,
}

#[derive(Debug, Clone, Copy)]
pub enum ActionEventEnum {
    Boolean {
        state: bool,
        changed: bool,
    },
    Delta2d {
        delta: Vector2<f64>,
    },
    Cursor {
        normalized_window_coords: Vector2<f64>,
    },
    Value {
        state: f32,
    },
    Axis1d {
        state: f32,
    },
    Axis2d {
        state: Vector2<f32>,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum ActionStateEnum {
    Boolean(bool),
    Delta2d(Vector2<f64>),
    Cursor(Vector2<f64>),
    Value(f32),
    Axis1d(f32),
    Axis2d(Vector2<f32>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChildActionType {
    StickyPress,
    StickyToggle,
    StickyRelease,
    Positive,
    Negative,
    Up,
    Right,
    Down,
    Left,
    Vertical,
    Horizontal,
    Move,
}
