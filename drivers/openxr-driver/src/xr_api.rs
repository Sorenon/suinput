use std::os::raw::c_char;

use crate::{cvt, place_cstr};
use openxr::{
    sys::{
        Action, ActionCreateInfo, ActionSet, ActionSetCreateInfo, MAX_ACTION_NAME_SIZE,
        MAX_ACTION_SET_NAME_SIZE, MAX_LOCALIZED_ACTION_NAME_SIZE,
        MAX_LOCALIZED_ACTION_SET_NAME_SIZE,
    },
    ActionType, Instance,
};

pub(crate) fn create_action_set(
    instance: &Instance,
    name: &str,
    localized_name: &str,
) -> ActionSet {
    let mut action_set = ActionSet::NULL;
    cvt(unsafe {
        let mut action_set_name: [c_char; MAX_ACTION_SET_NAME_SIZE] = std::mem::zeroed();
        let mut localized_action_set_name: [c_char; MAX_LOCALIZED_ACTION_SET_NAME_SIZE] =
            std::mem::zeroed();
        place_cstr(&mut action_set_name, name);
        place_cstr(&mut localized_action_set_name, localized_name);

        (instance.fp().create_action_set)(
            instance.as_raw(),
            &ActionSetCreateInfo {
                ty: ActionSetCreateInfo::TYPE,
                next: std::ptr::null(),
                action_set_name,
                localized_action_set_name,
                priority: 0,
            },
            &mut action_set,
        )
    })
    .unwrap();
    action_set
}

pub(crate) fn create_action(
    instance: &Instance,
    action_set: ActionSet,
    name: &str,
    localized_name: &str,
    action_type: ActionType,
) -> Action {
    let mut action = Action::NULL;
    cvt(unsafe {
        let mut action_name: [c_char; MAX_ACTION_NAME_SIZE] = std::mem::zeroed();
        let mut localized_action_name: [c_char; MAX_LOCALIZED_ACTION_NAME_SIZE] =
            std::mem::zeroed();
        place_cstr(&mut action_name, name);
        place_cstr(&mut localized_action_name, localized_name);

        (instance.fp().create_action)(
            action_set,
            &ActionCreateInfo {
                ty: ActionCreateInfo::TYPE,
                next: std::ptr::null(),
                action_name,
                action_type,
                count_subaction_paths: 0,
                subaction_paths: std::ptr::null(),
                localized_action_name,
            },
            &mut action,
        )
    })
    .unwrap();
    action
}
