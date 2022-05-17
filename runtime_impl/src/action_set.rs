use std::sync::Weak;

use super::instance::Instance;

pub struct ActionSet {
    instance: Weak<Instance>,

    name: String,
    default_priority: u32,
}

pub enum ActionType {
    Boolean { sticky: bool },
    Value,
    Axis1D,
    Axis2D { shape: () },
    Delta1D,
    Delta2D,
    Cursor,
}

impl ActionSet {
    pub fn create_action(&self, name: String, action_type: ActionType) {
        todo!()
    }

    pub fn create_action_layer(&self, name: String, default_priority: u32) {
        todo!()
    }
}
