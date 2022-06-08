use std::{fmt::Debug, ops::Deref, sync::Arc};

use thiserror::Error;

use crate::{event::*, SuPath};

/**
 * The connection from a driver to the runtime
 */
#[derive(Debug, Clone)]
pub struct RuntimeInterface(pub Arc<dyn RuntimeInterfaceTrait>);

impl Deref for RuntimeInterface {
    type Target = dyn RuntimeInterfaceTrait;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

#[derive(Debug, Error)]
pub enum RuntimeInterfaceError {
    #[error("Driver Uninitialized")]
    DriverUninitialized,
}

pub trait RuntimeInterfaceTrait: Debug + Send + Sync {
    fn register_new_device(&self, device_type: SuPath) -> Result<u64, RuntimeInterfaceError>;
    fn disconnect_device(&self, device_id: u64) -> Result<(), RuntimeInterfaceError>;
    fn send_component_event(
        &self,
        component_event: InputEvent,
    ) -> Result<(), RuntimeInterfaceError>;
    fn get_path(&self, path_string: &str) -> Result<SuPath, PathFormatError>;
    fn get_path_string(&self, path: SuPath) -> Option<String>;
}

pub trait SuInputDriver: Send + Sync {
    fn initialize(&mut self);
    //Force a refresh
    fn poll(&self);
    //TODO
    fn get_component_state(&self, device: usize, path: SuPath) -> ();
    fn set_windows(&mut self, _windows: &[usize]) {}
    fn destroy(&mut self);
}
