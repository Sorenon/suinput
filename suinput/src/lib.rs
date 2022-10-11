use std::marker::PhantomData;
use std::sync::Arc;

use suinput_core::types::action_type::{ActionType, Pose};
use suinput_core::{action, action_set, user};

pub use suinput_core::driver_interface;
pub use suinput_core::driver_interface::RuntimeInterface;
pub use suinput_core::driver_interface::SuInputDriver;
pub use suinput_core::types::*;

pub use suinput_types::action::ActionEvent;
pub use suinput_types::action::ActionEventEnum;
pub use suinput_types::action::ActionListener;
pub use suinput_types::event::PathFormatError;

pub mod application_instance;
pub mod instance;
pub mod runtime;
pub mod session;

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

/// Attempts to load the external runtime, falling back on the embedded runtime if it is not available.
///
/// This should only be called once per process
pub fn load_runtime() -> runtime::SuInputRuntime {
    runtime::SuInputRuntime(Inner::Embedded(suinput_core::runtime::Runtime::new()))
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
