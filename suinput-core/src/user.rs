use parking_lot::{Mutex, RwLock};
use suinput_types::action::ActionStateEnum;
use suinput_types::{SuPath, Time};

use crate::action::ActionTypeEnum;
use crate::internal::binding::binding_engine::processed_binding_layout::ProcessedBindingLayout;
use crate::internal::types::HashMap;
use crate::types::action_type::{
    Axis1dActionState, Axis2dActionState, BooleanActionState, CursorActionState,
    Delta2dActionState, ValueActionState,
};

#[derive(Default)]
pub struct User {
    pub action_states: RwLock<HashMap<u64, OutActionStateEnum>>,
    //should there also be a way to remove binding layouts?
    pub new_binding_layouts: Mutex<HashMap<SuPath, ProcessedBindingLayout>>,
}

pub enum OutActionStateEnum {
    Boolean(BooleanActionState),
    Delta2d(Delta2dActionState),
    Cursor(CursorActionState),
    Value(ValueActionState),
    Axis1d(Axis1dActionState),
    Axis2d(Axis2dActionState),
}

impl OutActionStateEnum {
    pub fn from_type(action_state: ActionTypeEnum) -> Self {
        match action_state {
            ActionTypeEnum::Boolean => Self::Boolean(Default::default()),
            ActionTypeEnum::Delta2d => Self::Delta2d(Default::default()),
            ActionTypeEnum::Cursor => Self::Cursor(Default::default()),
            ActionTypeEnum::Value => Self::Value(Default::default()),
            ActionTypeEnum::Axis1d => Self::Axis1d(Default::default()),
            ActionTypeEnum::Axis2d => Self::Axis2d(Default::default()),
        }
    }

    pub fn update(&mut self, action_state: &ActionStateEnum, last_changed_time: Time) {
        match (self, action_state) {
            (OutActionStateEnum::Boolean(prev_state), ActionStateEnum::Boolean(new_value)) => {
                let old_value = prev_state.current_state;
                prev_state.is_active = true;
                prev_state.current_state = *new_value;
                prev_state.changed_since_last_sync = *new_value != old_value;
                prev_state.last_changed_time = last_changed_time;
            }
            (OutActionStateEnum::Delta2d(prev_state), ActionStateEnum::Delta2d(new_value)) => {
                prev_state.is_active = true;
                prev_state.accumulated_delta = *new_value;
                prev_state.last_changed_time = last_changed_time;
            }
            (OutActionStateEnum::Cursor(prev_state), ActionStateEnum::Cursor(new_value)) => {
                let old_value = prev_state.current_state;
                prev_state.is_active = true;
                prev_state.current_state = *new_value;
                prev_state.changed_since_last_sync = *new_value != old_value;
                prev_state.last_changed_time = last_changed_time;
            }
            (OutActionStateEnum::Value(prev_state), ActionStateEnum::Value(new_value)) => {
                let old_value = prev_state.current_state;
                prev_state.is_active = true;
                prev_state.current_state = *new_value;
                prev_state.changed_since_last_sync = *new_value != old_value;
                prev_state.last_changed_time = last_changed_time;
            }
            (OutActionStateEnum::Axis1d(prev_state), ActionStateEnum::Axis1d(new_value)) => {
                let old_value = prev_state.current_state;
                prev_state.is_active = true;
                prev_state.current_state = *new_value;
                prev_state.changed_since_last_sync = *new_value != old_value;
                prev_state.last_changed_time = last_changed_time;
            }
            (OutActionStateEnum::Axis2d(prev_state), ActionStateEnum::Axis2d(new_value)) => {
                let old_value = prev_state.current_state;
                prev_state.is_active = true;
                prev_state.current_state = *new_value;
                prev_state.changed_since_last_sync = *new_value != old_value;
                prev_state.last_changed_time = last_changed_time;
            }
            _ => panic!("Action Type Mismatch"),
        }
    }

    pub fn mark_inactive(&mut self) {
        match self {
            OutActionStateEnum::Boolean(prev_state) => *prev_state = Default::default(),
            OutActionStateEnum::Delta2d(prev_state) => *prev_state = Default::default(),
            OutActionStateEnum::Cursor(prev_state) => *prev_state = Default::default(),
            OutActionStateEnum::Value(prev_state) => *prev_state = Default::default(),
            OutActionStateEnum::Axis1d(prev_state) => *prev_state = Default::default(),
            OutActionStateEnum::Axis2d(prev_state) => *prev_state = Default::default(),
        }
    }
}
