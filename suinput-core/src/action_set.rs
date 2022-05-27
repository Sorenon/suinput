use std::sync::{Arc, Weak};

use parking_lot::RwLock;
use suinput_types::action::ActionType;

use crate::action::Action;

use super::instance::Instance;

pub struct ActionSet {
    // A user-facing variable that can be used to uniquely identify an action set
    pub handle: u64,
    pub instance: Weak<Instance>,

    pub name: String,
    pub default_priority: u32,
    pub actions: RwLock<Vec<Arc<Action>>>,
}

impl ActionSet {
    pub fn create_action(self: &Arc<Self>, name: String, action_type: ActionType) -> Arc<Action> {
        let instance = self
            .instance
            .upgrade()
            .expect("Instance dropped unexpectedly");

        let mut instance_actions = instance.actions.write();

        let action = Arc::new(Action {
            handle: (instance_actions.len() as u64) + 1,
            action_set: Arc::downgrade(self),
            name,
            action_type,
        });

        instance_actions.push(action.clone());
        self.actions.write().push(action.clone());
        action
    }

    pub fn create_action_layer(&self, name: String, default_priority: u32) {
        todo!()
    }
}
