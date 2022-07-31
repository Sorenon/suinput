use std::sync::{Arc, Weak};

use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use suinput_types::action::ChildActionType;

use crate::{
    action::{Action, ActionCompoundType, ActionTypeEnum, ParentActionType},
    types::action_type::ActionType,
};

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
    pub fn create_action<T: ActionType>(
        self: &Arc<Self>,
        name: &str,
        create_info: T::CreateInfo,
    ) -> Arc<Action> {
        let instance = self
            .instance
            .upgrade()
            .expect("Instance dropped unexpectedly");

        let mut instance_actions = instance.actions.write();
        let mut set_actions = self.actions.write();

        use crate::types::action_type::private::InternalActionType;

        let action = Arc::new_cyclic(|action| {
            let action_type = T::Internal::action_type();

            let parent_action_type = match action_type {
                ActionTypeEnum::Boolean | ActionTypeEnum::Axis1d | ActionTypeEnum::Axis2d => {
                    T::Internal::create_child_actions(
                        T::appease_the_type_checker(create_info),
                        action,
                        |parent: Weak<Action>,
                         name: String,
                         action_type: ActionTypeEnum,
                         child_action_type: ChildActionType|
                         -> Arc<Action> {
                            self.create_child_action(
                                &mut instance_actions,
                                &mut set_actions,
                                parent,
                                name,
                                action_type,
                                child_action_type,
                            )
                        },
                    )
                }
                _ => ParentActionType::None,
            };

            Action {
                handle: instance_actions.len() as u64 + 1,
                action_set: Arc::downgrade(self),
                name: name.into(),
                data_type: action_type,
                compound: ActionCompoundType::Parent {
                    ty: parent_action_type,
                },
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
        action_type: ActionTypeEnum,
        child_action_type: ChildActionType,
    ) -> Arc<Action> {
        let action = Arc::new(Action {
            handle: (instance_actions.len() as u64) + 1,
            action_set: Arc::downgrade(self),
            name: name,
            data_type: action_type,
            compound: ActionCompoundType::Child {
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
