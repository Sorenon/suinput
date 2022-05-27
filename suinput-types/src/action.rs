use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    Boolean,
    Delta2D,
    Cursor,
}

pub trait ActionListener: Send + Sync {
    fn handle_event(&self, event: ActionEvent);
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
    Delta2D {
        delta: (f64, f64),
    },
    Cursor {
        normalized_screen_coords: (f64, f64),
    },
}

#[derive(Debug, Clone, Copy)]
pub enum ActionStateEnum {
    Boolean(bool),
    Delta2D((f64, f64)),
    Cursor((f64, f64)),
}
