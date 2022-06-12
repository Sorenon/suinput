use std::sync::{Arc, Weak};

use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use suinput_types::action::{ActionCreateInfo, ChildActionType};

use crate::action::{Action, ActionHierarchyType, ActionType, ParentActionType};

use super::instance::Instance;

pub struct ActionSet {
    // A user-facing variable that can be used to uniquely identify an action set
    pub handle: u64,
    pub instance: Weak<Instance>,

    pub name: String,
    pub default_priority: u32,
    pub actions: RwLock<Vec<Arc<Action>>>,
    pub baked_actions: OnceCell<Vec<Arc<Action>>>,
}

impl ActionSet {
    pub fn create_action(
        self: &Arc<Self>,
        name: &str,
        create_info: ActionCreateInfo,
    ) -> Arc<Action> {
        let instance = self
            .instance
            .upgrade()
            .expect("Instance dropped unexpectedly");

        let mut instance_actions = instance.actions.write();
        let mut set_actions = self.actions.write();

        let action = Arc::new_cyclic(|action| {
            let (action_type, hierarchy_type) = match create_info {
                ActionCreateInfo::Boolean {
                    sticky: has_sticky_child,
                } => {
                    if has_sticky_child {
                        (
                            ActionType::Boolean,
                            ParentActionType::StickyBool {
                                sticky_press: self.create_child_action(
                                    &mut instance_actions,
                                    &mut set_actions,
                                    action.clone(),
                                    "sticky_press".into(),
                                    ActionType::Boolean,
                                    ChildActionType::StickyPress,
                                ),
                                sticky_release: self.create_child_action(
                                    &mut instance_actions,
                                    &mut set_actions,
                                    action.clone(),
                                    "sticky_release".into(),
                                    ActionType::Boolean,
                                    ChildActionType::StickyRelease,
                                ),
                                sticky_toggle: self.create_child_action(
                                    &mut instance_actions,
                                    &mut set_actions,
                                    action.clone(),
                                    "sticky_toggle".into(),
                                    ActionType::Boolean,
                                    ChildActionType::StickyToggle,
                                ),
                            },
                        )
                    } else {
                        (ActionType::Boolean, ParentActionType::None)
                    }
                }
                ActionCreateInfo::Delta2D => (ActionType::Delta2D, ParentActionType::None),
                ActionCreateInfo::Cursor => (ActionType::Cursor, ParentActionType::None),
                ActionCreateInfo::Value => (ActionType::Value, ParentActionType::None),
                ActionCreateInfo::Axis1D { positive, negative } => (
                    ActionType::Axis1D,
                    ParentActionType::Axis1D {
                        positive: self.create_child_action(
                            &mut instance_actions,
                            &mut set_actions,
                            action.clone(),
                            positive.unwrap_or("positive".into()),
                            ActionType::Value,
                            ChildActionType::Positive,
                        ),
                        negative: self.create_child_action(
                            &mut instance_actions,
                            &mut set_actions,
                            action.clone(),
                            negative.unwrap_or("negative".into()),
                            ActionType::Value,
                            ChildActionType::Negative,
                        ),
                    },
                ),
                ActionCreateInfo::Axis2D {
                    up,
                    down,
                    left,
                    right,
                    vertical,
                    horizontal,
                } => (
                    ActionType::Axis2D,
                    ParentActionType::Axis2D {
                        up: self.create_child_action(
                            &mut instance_actions,
                            &mut set_actions,
                            action.clone(),
                            up.unwrap_or("up".into()),
                            ActionType::Value,
                            ChildActionType::Up,
                        ),
                        down: self.create_child_action(
                            &mut instance_actions,
                            &mut set_actions,
                            action.clone(),
                            down.unwrap_or("down".into()),
                            ActionType::Value,
                            ChildActionType::Down,
                        ),
                        left: self.create_child_action(
                            &mut instance_actions,
                            &mut set_actions,
                            action.clone(),
                            left.unwrap_or("left".into()),
                            ActionType::Value,
                            ChildActionType::Left,
                        ),
                        right: self.create_child_action(
                            &mut instance_actions,
                            &mut set_actions,
                            action.clone(),
                            right.unwrap_or("right".into()),
                            ActionType::Value,
                            ChildActionType::Right,
                        ),
                        vertical: self.create_child_action(
                            &mut instance_actions,
                            &mut set_actions,
                            action.clone(),
                            vertical.unwrap_or("vertical".into()),
                            ActionType::Axis1D,
                            ChildActionType::Vertical,
                        ),
                        horizontal: self.create_child_action(
                            &mut instance_actions,
                            &mut set_actions,
                            action.clone(),
                            horizontal.unwrap_or("horizontal".into()),
                            ActionType::Axis1D,
                            ChildActionType::Horizontal,
                        ),
                    },
                )
            };

            Action {
                handle: instance_actions.len() as u64 + 1,
                action_set: Arc::downgrade(self),
                name: name.into(),
                data_type: action_type,
                hierarchy_type: ActionHierarchyType::Parent { ty: hierarchy_type },
            }
        });

        instance_actions.push(action.clone());
        set_actions.push(action.clone());

        action
    }

    fn create_child_action(
        self: &Arc<Self>,
        instance_actions: &mut Vec<Arc<Action>>,
        set_actions: &mut Vec<Arc<Action>>,
        parent: Weak<Action>,
        name: String,
        action_type: ActionType,
        child_action_type: ChildActionType,
    ) -> Arc<Action> {
        let action = Arc::new(Action {
            handle: (instance_actions.len() as u64) + 1,
            action_set: Arc::downgrade(self),
            name: name,
            data_type: action_type,
            hierarchy_type: ActionHierarchyType::Child {
                parent,
                ty: child_action_type,
            },
        });

        instance_actions.push(action.clone());
        set_actions.push(action.clone());
        action
    }

    pub fn create_action_layer(&self, name: String, default_priority: u32) {
        todo!()
    }
}
