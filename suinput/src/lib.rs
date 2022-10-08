use std::marker::PhantomData;
use std::path::Path;
use std::sync::Arc;

use raw_window_handle::RawWindowHandle;
pub use suinput_core::driver_interface::RuntimeInterface;
pub use suinput_core::driver_interface::SuInputDriver;
use suinput_core::types::action_type::{ActionType, Pose};
pub use suinput_core::types::*;
pub use suinput_types::event::PathFormatError;

pub use suinput_core::driver_interface;
use suinput_core::{action, action_set, session, user};

pub mod runtime;
pub mod application_instance;
pub mod instance;

#[allow(dead_code)]
pub(crate) enum Inner<E> {
    Embedded(Arc<E>),
    FFI(/*TODO*/),
}

impl<E> Inner<E> {
    pub fn get(&self) -> Option<&Arc<E>> {
        match self {
            Inner::Embedded(e) => Some(e),
            Inner::FFI() => None,
        }
    }
}

impl<E> Clone for Inner<E> {
    fn clone(&self) -> Self {
        match self {
            Self::Embedded(arg0) => Self::Embedded(arg0.clone()),
            Self::FFI() => Self::FFI(),
        }
    }
}

/// Attempts to load the external runtime, falling back on the embedded runtime if it is not available
/// This should only be called once per process
///
/// fallback_path specifies a file to allow the embedded runtime to store persistent state
pub fn load_runtime(fallback_path: Option<&Path>) -> runtime::SuInputRuntime {
    runtime::SuInputRuntime(Inner::Embedded(suinput_core::runtime::Runtime::new(fallback_path)))
}

pub use suinput_types::action::ActionEvent;
pub use suinput_types::action::ActionEventEnum;
pub use suinput_types::action::ActionListener;

#[derive(Clone)]
pub struct SuSession(Inner<session::Session>);

impl SuSession {
    pub fn register_event_listener(&self, listener: Box<dyn ActionListener>) -> u64 {
        match &self.0 {
            Inner::Embedded(inner) => inner.register_event_listener(listener),
            Inner::FFI() => todo!(),
        }
    }

    pub fn get_main_user(&self) -> SuUser {
        SuUser(match &self.0 {
            Inner::Embedded(inner) => Inner::Embedded(inner.user.clone()),
            Inner::FFI() => todo!(),
        })
    }

    pub fn sync(&self, action_sets: &[&SuActionSet]) {
        match &self.0 {
            Inner::Embedded(inner) => inner.sync(action_sets.iter().map(|set| match &set.0 {
                Inner::Embedded(action_set) => action_set,
                Inner::FFI() => todo!(),
            })),
            Inner::FFI() => todo!(),
        }
    }

    pub fn get_action_state<T: ActionType>(&self, action: &SuAction<T>) -> Result<T::State, ()> {
        match (&self.0, &action.0) {
            (Inner::Embedded(inner), Inner::Embedded(action)) => {
                inner.get_action_state::<T>(action)
            }
            (Inner::FFI(), Inner::FFI()) => todo!(),
            _ => panic!(),
        }
    }

    pub fn unstick_bool_action(&self, action: &SuAction<bool>) {
        match (&self.0, &action.0) {
            (Inner::Embedded(inner), Inner::Embedded(action)) => inner.unstick_bool_action(action),
            (Inner::FFI(), Inner::FFI()) => todo!(),
            _ => panic!(),
        }
    }

    // pub fn create_action_space(&self, action: &SuAction<Pose>, pose_in_space: Pose) {
    //     todo!()
    // }
}

#[derive(Clone)]
pub struct SuUser(Inner<user::User>);

#[derive(Clone)]
pub struct SuBindingLayout(Inner<suinput_core::instance::BindingLayout>);

#[derive(Clone)]
pub struct SuActionSet(Inner<action_set::ActionSet>);

pub use suinput_types::action::ActionCreateInfo;

impl SuActionSet {
    pub fn create_action<T: ActionType>(
        &self,
        name: &str,
        create_info: T::CreateInfo,
    ) -> SuAction<T> {
        SuAction(
            match &self.0 {
                Inner::Embedded(inner) => {
                    Inner::Embedded(inner.create_action::<T>(name, create_info))
                }
                Inner::FFI() => todo!(),
            },
            PhantomData,
        )
    }

    pub fn create_action_layer(&self, _name: &str, _default_priority: u32) {
        todo!()
    }
}

#[derive(Clone)]
pub struct SuAction<T: ActionType>(Inner<action::Action>, PhantomData<T>);

pub use suinput_types::action::ChildActionType;

impl<T: ActionType> SuAction<T> {
    pub fn handle(&self) -> u64 {
        match &self.0 {
            Inner::Embedded(inner) => inner.handle,
            Inner::FFI() => todo!(),
        }
    }

    pub fn get_child_action(&self, ty: ChildActionType) -> u64 {
        match &self.0 {
            Inner::Embedded(inner) => inner.get_child_action(ty),
            Inner::FFI() => todo!(),
        }
    }
}
