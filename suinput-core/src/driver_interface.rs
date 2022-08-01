use std::{fmt::Debug, ops::Deref, sync::Arc, time::Instant};

use thiserror::Error;

use suinput_types::{event::*, SuPath};

use crate::internal::types::HashMap;

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
    fn start_batch_input_update(&self, device: u64, time: Instant) -> BatchInputUpdate;
    fn send_batch_input_update(
        &self,
        batch_update: BatchInputUpdate,
    ) -> Result<(), RuntimeInterfaceError>;
    fn get_path(&self, path_string: &str) -> Result<SuPath, PathFormatError>;
    fn get_path_string(&self, path: SuPath) -> Option<String>;
}

#[derive(Debug, Clone)]
pub struct BatchInputUpdate {
    pub(crate) device: u64,
    pub(crate) time: Instant,
    pub(crate) inner: HashMap<SuPath, InputComponentEvent>,
}

impl BatchInputUpdate {
    pub(crate) fn new(device: u64, time: Instant) -> Self {
        Self {
            device,
            time,
            inner: HashMap::new(),
        }
    }

    pub fn add_event(&mut self, path: SuPath, event: InputComponentEvent) {
        self.inner.insert(path, event);
    }
}

pub trait SuInputDriver: Send + Sync {
    fn initialize(&mut self);
    //Force a refresh
    fn poll(&self);
    //TODO
    fn get_component_state(&self, device: usize, path: SuPath);
    fn set_windows(&mut self, _windows: &[usize]) {}
    fn destroy(&mut self);
}
