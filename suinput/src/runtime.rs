use std::path::Path;

use crate::Inner;
pub use suinput_core::driver_interface::RuntimeInterface;
pub use suinput_core::driver_interface::SuInputDriver;
pub use suinput_core::types::*;
pub use suinput_types::event::PathFormatError;

use suinput_core::*;

pub use suinput_types::SuPath;

/// The selected SuInput Runtime for this process.
///
/// This is an abstraction over either the embedded runtime or an external runtime.
#[derive(Clone)]
pub struct SuInputRuntime(pub(crate) Inner<runtime::Runtime>);

impl SuInputRuntime {
    //TODO Move these to Instance
    pub fn add_instance_driver<F, T, E>(&self, f: F) -> core::result::Result<usize, E>
    where
        F: FnOnce(RuntimeInterface) -> core::result::Result<T, E>,
        T: SuInputDriver + 'static,
    {
        match &self.0 {
            Inner::Embedded(inner) => inner.add_driver(f),
            Inner::FFI() => todo!(),
        }
    }

    //TODO clean up this garbage
    pub fn add_runtime_driver<F, T, E>(&self, f: F) -> core::result::Result<usize, E>
    where
        F: FnOnce(RuntimeInterface) -> core::result::Result<T, E>,
        T: SuInputDriver + 'static,
    {
        match &self.0 {
            Inner::Embedded(inner) => inner.add_driver(f),
            Inner::FFI() => todo!(),
        }
    }

    /// Create an SuInstance for accessing the SuInput API
    pub fn create_instance(&self) -> super::instance::SuInstance {
        super::instance::SuInstance(match &self.0 {
            Inner::Embedded(inner) => Inner::Embedded(inner.create_instance(None)),
            Inner::FFI() => todo!(),
        })
    }

    pub fn destroy(&self) {
        match &self.0 {
            Inner::Embedded(inner) => inner.destroy(),
            Inner::FFI() => todo!(),
        }
    }
}
