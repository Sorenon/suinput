use suinput_types::{action::ActionStateEnum, SuPath};

use crate::internal::{paths::InteractionProfilePath, types::HashMap};

use super::working_user::WorkingActionState;

mod processed_binding;
pub mod processed_binding_layout;

pub struct WorkingUserInterface<'a> {
    pub(crate) path: InteractionProfilePath,
    pub(crate) layout_events: Vec<(u64, SuPath, ActionStateEnum)>,
    pub(crate) action_states: &'a HashMap<u64, ActionStateEnum>,
    pub(crate) action_info: &'a HashMap<u64, WorkingActionState>,
}

impl<'a> WorkingUserInterface<'a> {
    pub fn fire_action_event(&mut self, action: u64, new_binding_state: ActionStateEnum) {
        self.layout_events
            .push((action, self.path, new_binding_state))
    }

    pub fn is_action_active(&self, action_handle: u64) -> bool {
        todo!()
    }

    pub fn get_action_priority(&self, action_handle: u64) -> u32 {
        self.action_info.get(&action_handle).unwrap().priority
    }

    pub fn get_action_state(&self, action_handle: u64) -> Option<ActionStateEnum> {
        self.action_states.get(&action_handle).map(|s| s.clone())
    }
}
