use std::sync::Weak;

use suinput_types::action::ActionType;

use crate::action_set::ActionSet;

pub struct Action {
    pub handle: u64,
    pub action_set: Weak<ActionSet>,

    pub name: String,
    pub action_type: ActionType,
}
