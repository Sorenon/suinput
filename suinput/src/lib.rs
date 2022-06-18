use std::sync::Arc;

use raw_window_handle::RawWindowHandle;
use suinput_core::driver_interface::RuntimeInterface;
use suinput_core::driver_interface::SuInputDriver;
use suinput_types::event::PathFormatError;

use suinput_core::*;

pub use suinput_core::driver_interface;

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

pub use suinput_types::SuPath;

impl SuInputRuntime {
    pub fn set_window_driver<F, T, E>(&self, f: F) -> Result<usize, E>
    where
        F: FnOnce(RuntimeInterface) -> Result<T, E>,
        T: SuInputDriver + 'static,
    {
        match &self.0 {
            Inner::Embedded(inner) => inner.add_driver(f),
            Inner::FFI() => todo!(),
        }
    }

    pub fn add_generic_driver<F, T, E>(&self, f: F) -> Result<usize, E>
    where
        F: FnOnce(RuntimeInterface) -> Result<T, E>,
        T: SuInputDriver + 'static,
    {
        match &self.0 {
            Inner::Embedded(inner) => inner.add_driver(f),
            Inner::FFI() => todo!(),
        }
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

pub use suinput_types::action::ActionEvent;
pub use suinput_types::action::ActionEventEnum;
pub use suinput_types::action::ActionListener;

#[derive(Clone)]
pub struct SuInstance(Inner<instance::Instance>);

pub use suinput_types::binding::SimpleBinding;
pub use suinput_types::CreateBindingLayoutError;

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

    pub fn create_binding_layout(
        &self,
        name: &str,
        interaction_profile: SuPath,
        bindings: &[SimpleBinding],
    ) -> Result<SuBindingLayout, CreateBindingLayoutError> {
        Ok(SuBindingLayout(match &self.0 {
            Inner::Embedded(inner) => Inner::Embedded(inner.create_binding_layout(
                name.into(),
                interaction_profile,
                bindings,
            )?),
            Inner::FFI() => todo!(),
        }))
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

    pub fn create_session(&self, action_sets: &[&SuActionSet]) -> SuSession {
        SuSession(match &self.0 {
            Inner::Embedded(inner) => Inner::Embedded(
                inner.create_session(
                    &action_sets
                        .iter()
                        .map(|action_set| match &action_set.0 {
                            Inner::Embedded(thing) => thing,
                            _ => panic!(),
                        })
                        .collect::<Vec<_>>(),
                ),
            ),
            Inner::FFI() => todo!(),
        })
    }
}

#[derive(Clone)]
pub struct SuSession(Inner<session::Session>);

impl SuSession {
    pub fn set_window(&self, window: usize) {
        match &self.0 {
            Inner::Embedded(inner) => inner.set_window(window),
            Inner::FFI() => todo!(),
        }
    }

    pub fn set_window_rwh(&self, window: RawWindowHandle) {
        self.set_window(match window {
            RawWindowHandle::Win32(f) => f.hwnd as usize,
            _ => todo!(),
        })
    }

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

    pub fn poll(&self) {
        match &self.0 {
            Inner::Embedded(inner) => inner.poll(),
            Inner::FFI() => todo!(),
        }
    }

    pub fn unstick_bool_action(&self, action: &SuAction) {
        match (&self.0, &action.0) {
            (Inner::Embedded(inner), Inner::Embedded(action)) => inner.unstick_bool_action(action),
            (Inner::FFI(), Inner::FFI()) => todo!(),
            _ => panic!(),
        }
    }
}

#[derive(Clone)]
pub struct SuUser(Inner<user::User>);

#[derive(Clone)]
pub struct SuBindingLayout(Inner<instance::BindingLayout>);

#[derive(Clone)]
pub struct SuActionSet(Inner<action_set::ActionSet>);

pub use suinput_types::action::ActionCreateInfo;

impl SuActionSet {
    pub fn create_action(&self, name: &str, action_type: ActionCreateInfo) -> SuAction {
        SuAction(match &self.0 {
            Inner::Embedded(inner) => Inner::Embedded(inner.create_action(name, action_type)),
            Inner::FFI() => todo!(),
        })
    }

    pub fn create_action_layer(&self, name: &str, default_priority: u32) {
        todo!()
    }
}

#[derive(Clone)]
pub struct SuAction(Inner<action::Action>);

pub use suinput_types::action::ChildActionType;

impl SuAction {
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
