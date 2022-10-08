use std::path::Path;

pub use suinput_core::driver_interface::RuntimeInterface;
pub use suinput_core::driver_interface::SuInputDriver;
pub use suinput_core::types::*;
pub use suinput_types::event::PathFormatError;
use crate::{Inner};

use suinput_core::*;

pub use suinput_types::SuPath;

#[derive(Clone)]
pub struct SuInputRuntime(pub(crate) Inner<runtime::Runtime>);

impl SuInputRuntime {
    //TODO Move these to Instance
    pub fn add_instance_driver<F, T, E>(&self, f: F) -> Result<usize, E>
        where
            F: FnOnce(RuntimeInterface) -> Result<T, E>,
            T: SuInputDriver + 'static,
    {
        match &self.0 {
            Inner::Embedded(inner) => inner.add_driver(f),
            Inner::FFI() => todo!(),
        }
    }

    pub fn add_runtime_driver<F, T, E>(&self, f: F) -> Result<usize, E>
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

    pub fn create_instance(&self, storage_path: Option<&Path>) -> super::instance::SuInstance {
        super::instance::SuInstance(match &self.0 {
            Inner::Embedded(inner) => Inner::Embedded(inner.create_instance(storage_path)),
            Inner::FFI() => todo!(),
        })
    }
}
