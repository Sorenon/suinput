use std::sync::{Arc, Weak};

use mint::Vector2;

use suinput_types::action::{ActionStateEnum, ChildActionType};

use crate::action::{Action, ActionTypeEnum, ParentActionType};

use self::private::InternalActionType;

pub trait ActionType: Copy + private::Sealed {
    type Value;
    type CreateInfo;

    fn from_ase(ase: &ActionStateEnum) -> Option<Self::Value>;

    #[doc(hidden)]
    type Internal: private::InternalActionType;

    #[doc(hidden)]
    fn appease_the_type_checker(
        create_info: Self::CreateInfo,
    ) -> <<Self as ActionType>::Internal as ActionType>::CreateInfo;
}

//I hate this hack so much
pub(crate) mod private {
    use std::sync::{Arc, Weak};

    use crate::action::{Action, ActionTypeEnum, ParentActionType};

    use super::*;

    //Used for sealing ActionType
    pub trait Sealed {}
    impl Sealed for bool {}
    impl Sealed for Value {}
    impl Sealed for Delta2d {}
    impl Sealed for Cursor {}
    impl Sealed for Axis1d {}
    impl Sealed for Axis2d {}

    //Used for putting private methods on ActionType
    pub trait InternalActionType: ActionType {
        fn action_type() -> ActionTypeEnum;

        fn create_child_actions<F>(
            _create_info: Self::CreateInfo,
            _action: &Weak<Action>,
            _create_child_action: F,
        ) -> ParentActionType
        where
            F: FnMut(Weak<Action>, String, ActionTypeEnum, ChildActionType) -> Arc<Action>,
        {
            ParentActionType::None
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BooleanActionCreateInfo {
    pub sticky: bool,
}

impl ActionType for bool {
    type Value = Self;
    type CreateInfo = BooleanActionCreateInfo;

    fn from_ase(ase: &ActionStateEnum) -> Option<Self::Value> {
        match ase {
            ActionStateEnum::Boolean(state) => Some(*state),
            _ => None,
        }
    }

    type Internal = Self;

    fn appease_the_type_checker(
        create_info: Self::CreateInfo,
    ) -> <<Self as ActionType>::Internal as ActionType>::CreateInfo {
        create_info
    }
}

impl InternalActionType for bool {
    fn action_type() -> ActionTypeEnum {
        ActionTypeEnum::Boolean
    }

    fn create_child_actions<F>(
        create_info: Self::CreateInfo,
        action: &Weak<Action>,
        mut create_child_action: F,
    ) -> ParentActionType
    where
        F: FnMut(Weak<Action>, String, ActionTypeEnum, ChildActionType) -> Arc<Action>,
    {
        if create_info.sticky {
            ParentActionType::StickyBool {
                sticky_press: create_child_action(
                    action.clone(),
                    "sticky_press".into(),
                    ActionTypeEnum::Boolean,
                    ChildActionType::StickyPress,
                ),
                sticky_release: create_child_action(
                    action.clone(),
                    "sticky_release".into(),
                    ActionTypeEnum::Boolean,
                    ChildActionType::StickyRelease,
                ),
                sticky_toggle: create_child_action(
                    action.clone(),
                    "sticky_toggle".into(),
                    ActionTypeEnum::Boolean,
                    ChildActionType::StickyToggle,
                ),
            }
        } else {
            ParentActionType::None
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Value;

impl ActionType for Value {
    type Value = f32;
    type CreateInfo = ();

    fn from_ase(ase: &ActionStateEnum) -> Option<Self::Value> {
        match ase {
            ActionStateEnum::Value(state) => Some(*state),
            _ => None,
        }
    }

    type Internal = Self;

    fn appease_the_type_checker(
        create_info: Self::CreateInfo,
    ) -> <<Self as ActionType>::Internal as ActionType>::CreateInfo {
        create_info
    }
}

impl InternalActionType for Value {
    fn action_type() -> ActionTypeEnum {
        ActionTypeEnum::Value
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Delta2d;

impl ActionType for Delta2d {
    type Value = Vector2<f64>;
    type CreateInfo = ();

    fn from_ase(ase: &ActionStateEnum) -> Option<Self::Value> {
        match ase {
            ActionStateEnum::Delta2d(state) => Some(*state),
            _ => None,
        }
    }

    type Internal = Self;

    fn appease_the_type_checker(
        create_info: Self::CreateInfo,
    ) -> <<Self as ActionType>::Internal as ActionType>::CreateInfo {
        create_info
    }
}

impl InternalActionType for Delta2d {
    fn action_type() -> ActionTypeEnum {
        ActionTypeEnum::Delta2d
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Cursor;

impl ActionType for Cursor {
    type Value = Vector2<f64>;
    type CreateInfo = ();

    fn from_ase(ase: &ActionStateEnum) -> Option<Self::Value> {
        match ase {
            ActionStateEnum::Cursor(state) => Some(*state),
            _ => None,
        }
    }

    type Internal = Self;

    fn appease_the_type_checker(
        create_info: Self::CreateInfo,
    ) -> <<Self as ActionType>::Internal as ActionType>::CreateInfo {
        create_info
    }
}

impl InternalActionType for Cursor {
    fn action_type() -> ActionTypeEnum {
        ActionTypeEnum::Cursor
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Axis1d;

#[derive(Debug, Clone)]
pub struct Axis1dActionCreateInfo {
    pub positive: Option<String>,
    pub negative: Option<String>,
}

impl ActionType for Axis1d {
    type Value = f32;
    type CreateInfo = Axis1dActionCreateInfo;

    fn from_ase(ase: &ActionStateEnum) -> Option<Self::Value> {
        match ase {
            ActionStateEnum::Axis1d(state) => Some(*state),
            _ => None,
        }
    }

    type Internal = Self;

    fn appease_the_type_checker(
        create_info: Self::CreateInfo,
    ) -> <<Self as ActionType>::Internal as ActionType>::CreateInfo {
        create_info
    }
}

impl InternalActionType for Axis1d {
    fn action_type() -> ActionTypeEnum {
        ActionTypeEnum::Axis1d
    }

    fn create_child_actions<F>(
        create_info: Self::CreateInfo,
        action: &Weak<Action>,
        mut create_child_action: F,
    ) -> ParentActionType
    where
        F: FnMut(Weak<Action>, String, ActionTypeEnum, ChildActionType) -> Arc<Action>,
    {
        ParentActionType::Axis1d {
            positive: create_child_action(
                action.clone(),
                create_info.positive.unwrap_or_else(|| "positive".into()),
                ActionTypeEnum::Value,
                ChildActionType::Positive,
            ),
            negative: create_child_action(
                action.clone(),
                create_info.negative.unwrap_or_else(|| "negative".into()),
                ActionTypeEnum::Value,
                ChildActionType::Negative,
            ),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Axis2d;

#[derive(Debug, Clone)]
pub struct Axis2dActionCreateInfo {
    pub up: Option<String>,
    pub down: Option<String>,
    pub left: Option<String>,
    pub right: Option<String>,
    pub vertical: Option<String>,
    pub horizontal: Option<String>,
}

impl ActionType for Axis2d {
    type Value = Vector2<f32>;
    type CreateInfo = Axis2dActionCreateInfo;

    fn from_ase(ase: &ActionStateEnum) -> Option<Self::Value> {
        match ase {
            ActionStateEnum::Axis2d(state) => Some(*state),
            _ => None,
        }
    }

    type Internal = Self;

    fn appease_the_type_checker(
        create_info: Self::CreateInfo,
    ) -> <<Self as ActionType>::Internal as ActionType>::CreateInfo {
        create_info
    }
}

impl InternalActionType for Axis2d {
    fn action_type() -> ActionTypeEnum {
        ActionTypeEnum::Axis2d
    }

    fn create_child_actions<F>(
        create_info: Self::CreateInfo,
        action: &Weak<Action>,
        mut create_child_action: F,
    ) -> ParentActionType
    where
        F: FnMut(Weak<Action>, String, ActionTypeEnum, ChildActionType) -> Arc<Action>,
    {
        ParentActionType::Axis2d {
            up: create_child_action(
                action.clone(),
                create_info.up.unwrap_or_else(|| "up".into()),
                ActionTypeEnum::Value,
                ChildActionType::Up,
            ),
            down: create_child_action(
                action.clone(),
                create_info.down.unwrap_or_else(|| "down".into()),
                ActionTypeEnum::Value,
                ChildActionType::Down,
            ),
            left: create_child_action(
                action.clone(),
                create_info.left.unwrap_or_else(|| "left".into()),
                ActionTypeEnum::Value,
                ChildActionType::Left,
            ),
            right: create_child_action(
                action.clone(),
                create_info.right.unwrap_or_else(|| "right".into()),
                ActionTypeEnum::Value,
                ChildActionType::Right,
            ),
            vertical: create_child_action(
                action.clone(),
                create_info.vertical.unwrap_or_else(|| "vertical".into()),
                ActionTypeEnum::Axis1d,
                ChildActionType::Vertical,
            ),
            horizontal: create_child_action(
                action.clone(),
                create_info
                    .horizontal
                    .unwrap_or_else(|| "horizontal".into()),
                ActionTypeEnum::Axis1d,
                ChildActionType::Horizontal,
            ),
        }
    }
}
