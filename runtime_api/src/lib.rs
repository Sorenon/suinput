use std::sync::Arc;

use raw_window_handle::RawWindowHandle;
use suinput::{
    driver_interface::{DriverInterface, RuntimeInterface},
};

use runtime_impl::*;

#[derive(Clone)]
pub struct SuInputRuntime(Inner<runtime::Runtime>);

#[allow(dead_code)]
pub(crate) enum Inner<E> {
    Embedded(Arc<E>),
    FFI(/*TODO*/),
}

impl<E> Clone for Inner<E> {
    fn clone(&self) -> Self {
        match self {
            Self::Embedded(arg0) => Self::Embedded(arg0.clone()),
            Self::FFI() => Self::FFI(),
        }
    }
}

impl SuInputRuntime {
    pub fn new_tmp() -> Self {
        SuInputRuntime(Inner::Embedded(Arc::new(runtime::Runtime::new())))
    }

    pub fn add_driver<F, T, E>(&self, f: F) -> Result<usize, E>
    where
        F: FnOnce(RuntimeInterface) -> Result<T, E>,
        T: DriverInterface + 'static,
    {
        match &self.0 {
            Inner::Embedded(inner) => inner.add_driver(f),
            Inner::FFI() => todo!(),
        }
    }

    pub fn set_windows(&self, windows: &[usize]) {
        match &self.0 {
            Inner::Embedded(inner) => inner.set_windows(windows),
            Inner::FFI() => todo!(),
        }
    }

    pub fn set_windows_rwh(&self, raw_window_handles: &[RawWindowHandle]) {
        self.set_windows(
            &raw_window_handles
                .iter()
                .filter_map(|raw_window_handle| match raw_window_handle {
                    RawWindowHandle::Win32(f) => Some(f.hwnd as usize),
                    _ => None,
                })
                .collect::<Vec<usize>>(),
        );
    }

    pub fn destroy(&self) {
        match &self.0 {
            Inner::Embedded(inner) => inner.destroy(),
            Inner::FFI() => todo!(),
        }
    }

    pub fn create_instance(&self, name: String) -> SuInstance {
        SuInstance(match &self.0 {
            Inner::Embedded(inner) => Inner::Embedded(inner.create_instance(name)),
            Inner::FFI() => todo!(),
        })
    }
}

#[derive(Clone)]
pub struct SuInstance(Inner<instance::Instance>);

impl SuInstance {
    pub fn create_action_set(&self, name: String, default_priority: u32) -> SuActionSet {
        SuActionSet(match &self.0 {
            Inner::Embedded(inner) => {
                Inner::Embedded(inner.create_action_set(name, default_priority))
            }
            Inner::FFI() => todo!(),
        })
    }
}

#[derive(Clone)]
pub struct SuActionSet(Inner<action_set::ActionSet>);

pub use action_set::ActionType;

impl SuActionSet {
    pub fn create_action(&self, name: String, action_type: ActionType) -> SuAction {
        SuAction(match &self.0 {
            Inner::Embedded(inner) => Inner::Embedded(inner.create_action(name, action_type)),
            Inner::FFI() => todo!(),
        })
    }

    pub fn create_action_layer(&self, name: String, default_priority: u32) {
        todo!()
    }
}

#[derive(Clone)]
pub struct SuAction(Inner<action::Action>);

impl SuAction {
    pub fn handle(&self) -> u64 {
        match &self.0 {
            Inner::Embedded(inner) => inner.handle,
            Inner::FFI() => todo!(),
        }
    }
}