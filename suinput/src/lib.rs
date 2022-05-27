use std::sync::Arc;

use raw_window_handle::RawWindowHandle;
use suinput_types::{
    driver_interface::{DriverInterface, RuntimeInterface},
    event::PathFormatError,
    SuPath,
};

use suinput_core::*;

pub fn load_runtime() -> SuInputRuntime {
    SuInputRuntime(Inner::Embedded(runtime::Runtime::new()))
}

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

    pub fn create_instance(&self, name: &str) -> SuInstance {
        SuInstance(match &self.0 {
            Inner::Embedded(inner) => Inner::Embedded(inner.create_instance(name.into())),
            Inner::FFI() => todo!(),
        })
    }
}

pub use instance::ActionEvent;
pub use instance::ActionEventEnum;
pub use instance::ActionListener;

#[derive(Clone)]
pub struct SuInstance(Inner<instance::Instance>);

pub use instance::SimpleBinding;

impl SuInstance {
    pub fn get_path(&self, path_string: &str) -> Result<SuPath, PathFormatError> {
        match &self.0 {
            Inner::Embedded(inner) => inner.get_path(path_string),
            Inner::FFI() => todo!(),
        }
    }

    pub fn create_action_set(&self, name: &str, default_priority: u32) -> SuActionSet {
        SuActionSet(match &self.0 {
            Inner::Embedded(inner) => {
                Inner::Embedded(inner.create_action_set(name.into(), default_priority))
            }
            Inner::FFI() => todo!(),
        })
    }

    pub fn register_event_listener(&self, listener: Box<dyn ActionListener>) -> u64 {
        match &self.0 {
            Inner::Embedded(inner) => inner.register_event_listener(listener),
            Inner::FFI() => todo!(),
        }
    }

    pub fn create_binding_layout(
        &self,
        name: &str,
        interaction_profile: SuPath,
        bindings: &[SimpleBinding],
    ) -> SuBindingLayout {
        SuBindingLayout(match &self.0 {
            Inner::Embedded(inner) => Inner::Embedded(inner.create_binding_layout(
                name.into(),
                interaction_profile,
                bindings,
            )),
            Inner::FFI() => todo!(),
        })
    }

    pub fn set_default_binding_layout(
        &self,
        interaction_profile: SuPath,
        binding_layout: &SuBindingLayout,
    ) {
        match (&self.0, &binding_layout.0) {
            (Inner::Embedded(inner), Inner::Embedded(binding_layout)) => {
                inner.set_default_binding_layout(interaction_profile, binding_layout)
            }
            (Inner::FFI(), Inner::FFI()) => todo!(),
            _ => panic!(),
        }
    }
}

#[derive(Clone)]
pub struct SuBindingLayout(Inner<instance::BindingLayout>);

#[derive(Clone)]
pub struct SuActionSet(Inner<action_set::ActionSet>);

pub use suinput_types::action::ActionType;

impl SuActionSet {
    pub fn create_action(&self, name: &str, action_type: ActionType) -> SuAction {
        SuAction(match &self.0 {
            Inner::Embedded(inner) => {
                Inner::Embedded(inner.create_action(name.into(), action_type))
            }
            Inner::FFI() => todo!(),
        })
    }

    pub fn create_action_layer(&self, name: &str, default_priority: u32) {
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
