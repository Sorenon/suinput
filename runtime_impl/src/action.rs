use std::sync::Weak;

use crate::action_set::{ActionSet, ActionType};

pub struct Action {
    pub handle: u64,
    pub action_set: Weak<ActionSet>,

    pub name: String,
    pub action_type: ActionType,
}
