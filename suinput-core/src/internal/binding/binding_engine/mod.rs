use std::{cell::RefCell, sync::Arc};

use suinput_types::{
    action::{ActionListener, ActionStateEnum},
    SuPath,
};

use crate::{
    action::Action,
    internal::{paths::InteractionProfilePath, types::HashMap, interaction_profile_type::InteractionProfileType, compound_action::CompoundActionState},
};

use super::{
    action_hierarchy::ParentActionState,
    working_user::{AttachedBindingLayout, WorkingActionState, WorkingUser},
};

mod processed_binding;
pub mod processed_binding_layout;

pub struct WorkingUserInterface<'a> {
    pub(crate) binding_layouts: &'a HashMap<SuPath, RefCell<AttachedBindingLayout>>,
    pub(crate) binding_layout_action_states: &'a mut HashMap<u64, ActionStateEnum>,

    pub(crate) interaction_profile_id: InteractionProfilePath,

    pub(crate) action_states: &'a mut HashMap<u64, WorkingActionState>,
    pub(crate) compound_action_states: &'a mut HashMap<u64, Box<dyn CompoundActionState>>,
    pub(crate) callbacks: &'a mut [Box<dyn ActionListener>],
    pub(crate) actions: &'a HashMap<u64, Arc<Action>>,
}

impl<'a> WorkingUserInterface<'a> {
    pub fn fire_action_event(&mut self, action: u64, new_binding_state: ActionStateEnum) {
        self.binding_layout_action_states
            .insert(action, new_binding_state);

        WorkingUser::handle_binding_event(
            self.action_states,
            self.binding_layouts,
            self.compound_action_states,
            self.callbacks,
            self.actions,
            action,
            self.interaction_profile_id,
            new_binding_state,
        );
    }

    pub fn is_action_active(&self, action_handle: u64) -> bool {
        todo!()
    }

    pub fn get_action_priority(&self, action_handle: u64) -> u32 {
        self.action_states.get(&action_handle).unwrap().priority
    }

    pub fn get_action_state(&self, action_handle: u64) -> Option<ActionStateEnum> {
        self.binding_layout_action_states
            .get(&action_handle)
            .map(|s| s.clone())
    }
}
