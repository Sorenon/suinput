use std::sync::Arc;

use raw_window_handle::RawWindowHandle;
use suinput::driver_interface::{DriverInterface, RuntimeInterface};

mod inner;

pub struct SuInputRuntime(SuInputRuntimeEnum);

#[allow(dead_code)]
pub(crate) enum SuInputRuntimeEnum {
    Embedded(Arc<inner::runtime::Runtime>),
    FFI(/*TODO*/),
}

impl SuInputRuntime {
    pub fn new_tmp() -> Self {
        SuInputRuntime(SuInputRuntimeEnum::Embedded(Arc::new(
            inner::runtime::Runtime::new(),
        )))
    }

    pub fn add_driver<F, T, E>(&self, f: F) -> Result<usize, E>
    where
        F: FnOnce(RuntimeInterface) -> Result<T, E>,
        T: DriverInterface + 'static,
    {
        match &self.0 {
            SuInputRuntimeEnum::Embedded(inner) => inner.add_driver(f),
            SuInputRuntimeEnum::FFI() => todo!(),
        }
    }

    pub fn set_windows(&self, windows: &[usize]) {
        match &self.0 {
            SuInputRuntimeEnum::Embedded(inner) => inner.set_windows(windows),
            SuInputRuntimeEnum::FFI() => todo!(),
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
            SuInputRuntimeEnum::Embedded(inner) => inner.destroy(),
            SuInputRuntimeEnum::FFI() => todo!(),
        }
    }

    pub fn create_instance(&self, name: String) -> SuInstance {
        match &self.0 {
            SuInputRuntimeEnum::Embedded(inner) => {
                SuInstance(SuInstanceEnum::Embedded(inner.create_instance(name, ())))
            }
            SuInputRuntimeEnum::FFI() => todo!(),
        }
    }
}

pub struct SuInstance(SuInstanceEnum);

#[allow(dead_code)]
pub(crate) enum SuInstanceEnum {
    Embedded(Arc<inner::instance::Instance>),
    FFI(/*TODO*/),
}
