use std::sync::{Arc, Weak};

use suinput_types::action::ChildActionType;

use crate::action_set::ActionSet;

pub struct Action {
    pub handle: u64,
    pub action_set: Weak<ActionSet>,

    pub name: String,

    pub data_type: ActionType,
    pub hierarchy_type: ActionHierarchyType,
}

pub enum ActionHierarchyType {
    Child {
        parent: Weak<Action>,
        ty: ChildActionType,
    },
    Parent {
        ty: ParentActionType,
    },
}

pub enum ParentActionType {
    StickyBool {
        sticky_press: Arc<Action>,
        sticky_release: Arc<Action>,
        sticky_toggle: Arc<Action>,
    },
    Axis1d {
        positive: Arc<Action>,
        negative: Arc<Action>,
    },
    Axis2d {
        up: Arc<Action>,
        down: Arc<Action>,
        left: Arc<Action>,
        right: Arc<Action>,
        vertical: Arc<Action>,
        horizontal: Arc<Action>,
    },
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    Boolean,
    Delta2D,
    Cursor,
    Value,
    Axis1d,
    Axis2d,
}

impl Action {
    pub fn get_child_action(&self, ty: ChildActionType) -> u64 {
        if let ActionHierarchyType::Parent { ty: parent_type } = &self.hierarchy_type {
            match &parent_type {
                ParentActionType::StickyBool {
                    sticky_press,
                    sticky_release,
                    sticky_toggle,
                } => match ty {
                    ChildActionType::StickyPress => sticky_press.handle,
                    ChildActionType::StickyRelease => sticky_release.handle,
                    ChildActionType::StickyToggle => sticky_toggle.handle,
                    _ => todo!(),
                },
                ParentActionType::Axis1d { positive, negative } => match ty {
                    ChildActionType::Positive => positive.handle,
                    ChildActionType::Negative => negative.handle,
                    _ => todo!(),
                },
                ParentActionType::Axis2d {
                    up,
                    down,
                    left,
                    right,
                    vertical,
                    horizontal,
                } => match ty {
                    ChildActionType::Up => up.handle,
                    ChildActionType::Down => down.handle,
                    ChildActionType::Left => left.handle,
                    ChildActionType::Right => right.handle,
                    ChildActionType::Vertical => vertical.handle,
                    ChildActionType::Horizontal => horizontal.handle,
                    _ => todo!(),
                },
                _ => todo!(),
            }
        } else {
            todo!()
        }
    }
}
