use std::sync::Weak;

use super::instance::Instance;

pub struct ActionSet {
    instance: Weak<Instance>,

    name: String,
    default_priority: u32,
}

impl ActionSet {
    pub fn create_action(&self, name: String, action_type: ()) {
        todo!()
    }

    pub fn create_action_layer(&self, name: String, default_priority: u32) {
        todo!()
    }
}
