use std::sync::{Arc, Weak};

use mint::Vector2;

use suinput_types::{
    action::{ActionStateEnum, ChildActionType},
    Time,
};

use crate::{
    action::{Action, ActionTypeEnum, ParentActionType},
    user::OutActionStateEnum,
};
use crate::types::action_type::private::Sealed;

use self::private::InternalActionType;

pub trait ActionType: Copy + private::Sealed {
    type Value;
    type State: Copy;
    type CreateInfo;

    fn from_ase(ase: &ActionStateEnum) -> Option<Self::Value>;

    #[doc(hidden)]
    fn pick_state(state: &OutActionStateEnum) -> Option<&Self::State>;

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
    impl Sealed for Axis1d {}
    impl Sealed for Axis2d {}

    //Used for putting private methods on ActionType
    pub trait InternalActionType: ActionType {
        fn action_type() -> ActionTypeEnum;

        fn create_child_actions<F>(
            _create_info: Self::CreateInfo,
            _action: &Weak<Action>,
            _create_child_action: F,
        ) -> Option<ParentActionType>
        where
            F: FnMut(Weak<Action>, String, ActionTypeEnum, ChildActionType) -> Arc<Action>,
        {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BooleanActionCreateInfo {
    pub sticky: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BooleanActionState {
    pub current_state: bool,
    pub changed_since_last_sync: bool,
    pub last_changed_time: Time,
    pub is_active: bool,
}

impl ActionType for bool {
    type Value = Self;
    type State = BooleanActionState;
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

    fn pick_state(state: &OutActionStateEnum) -> Option<&Self::State> {
        match state {
            OutActionStateEnum::Boolean(state) => Some(state),
            _ => None,
        }
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
    ) -> Option<ParentActionType>
    where
        F: FnMut(Weak<Action>, String, ActionTypeEnum, ChildActionType) -> Arc<Action>,
    {
        if create_info.sticky {
            Some(ParentActionType::StickyBool {
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
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Value;

#[derive(Debug, Clone, Copy, Default)]
pub struct ValueActionState {
    pub current_state: f32,
    pub changed_since_last_sync: bool,
    pub last_changed_time: Time,
    pub is_active: bool,
}

impl ActionType for Value {
    type Value = f32;
    type State = ValueActionState;
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

    fn pick_state(state: &OutActionStateEnum) -> Option<&Self::State> {
        match state {
            OutActionStateEnum::Value(state) => Some(state),
            _ => None,
        }
    }
}

impl InternalActionType for Value {
    fn action_type() -> ActionTypeEnum {
        ActionTypeEnum::Value
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Delta2d;

#[derive(Debug, Clone, Copy)]
pub struct Delta2dActionState {
    pub accumulated_delta: Vector2<f64>,
    pub last_changed_time: Time,
    pub is_active: bool,
}

impl Default for Delta2dActionState {
    fn default() -> Self {
        Self {
            accumulated_delta: Vector2 { x: 0., y: 0. },
            last_changed_time: Default::default(),
            is_active: Default::default(),
        }
    }
}

impl ActionType for Delta2d {
    type Value = Vector2<f64>;
    type State = Delta2dActionState;
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

    fn pick_state(state: &OutActionStateEnum) -> Option<&Self::State> {
        match state {
            OutActionStateEnum::Delta2d(state) => Some(state),
            _ => None,
        }
    }
}

impl InternalActionType for Delta2d {
    fn action_type() -> ActionTypeEnum {
        ActionTypeEnum::Delta2d
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Axis1d;

#[derive(Debug, Default, Clone)]
pub struct Axis1dActionCreateInfo {
    pub positive: Option<String>,
    pub negative: Option<String>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Axis1dActionState {
    pub current_state: f32,
    pub changed_since_last_sync: bool,
    pub last_changed_time: Time,
    pub is_active: bool,
}

impl ActionType for Axis1d {
    type Value = f32;
    type State = Axis1dActionState;
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

    fn pick_state(state: &OutActionStateEnum) -> Option<&Self::State> {
        match state {
            OutActionStateEnum::Axis1d(state) => Some(state),
            _ => None,
        }
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
    ) -> Option<ParentActionType>
    where
        F: FnMut(Weak<Action>, String, ActionTypeEnum, ChildActionType) -> Arc<Action>,
    {
        Some(ParentActionType::Axis1d {
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
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Axis2d;

#[derive(Debug, Default, Clone)]
pub struct Axis2dActionCreateInfo {
    pub up: Option<String>,
    pub down: Option<String>,
    pub left: Option<String>,
    pub right: Option<String>,
    pub vertical: Option<String>,
    pub horizontal: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct Axis2dActionState {
    pub current_state: Vector2<f32>,
    pub changed_since_last_sync: bool,
    pub last_changed_time: Time,
    pub is_active: bool,
}

impl Default for Axis2dActionState {
    fn default() -> Self {
        Self {
            current_state: Vector2 { x: 0., y: 0. },
            changed_since_last_sync: Default::default(),
            last_changed_time: Default::default(),
            is_active: Default::default(),
        }
    }
}

impl ActionType for Axis2d {
    type Value = Vector2<f32>;
    type State = Axis2dActionState;
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

    fn pick_state(state: &OutActionStateEnum) -> Option<&Self::State> {
        match state {
            OutActionStateEnum::Axis2d(state) => Some(state),
            _ => None,
        }
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
    ) -> Option<ParentActionType>
    where
        F: FnMut(Weak<Action>, String, ActionTypeEnum, ChildActionType) -> Arc<Action>,
    {
        Some(ParentActionType::Axis2d {
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
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Pose {
    transform: mint::Vector3<f32>,
    oritentation: mint::Quaternion<f32>,
}

#[derive(Debug, Clone, Copy)]
pub struct PoseActionState {
    pub is_active: bool,
}
