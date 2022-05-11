use std::{fmt::Debug, ops::Deref, sync::Arc};

use crate::{event::*, SuPath};

/**
 * The connection from a driver to the runtime
 */
#[derive(Debug, Clone)]
pub struct DriverRuntimeInterface(pub Arc<dyn DriverRuntimeInterfaceTrait>);

impl Deref for DriverRuntimeInterface {
    type Target = dyn DriverRuntimeInterfaceTrait;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

pub trait DriverRuntimeInterfaceTrait: Debug + Send + Sync {
    fn register_new_device(&self, device_type: SuPath) -> u64;
    fn disconnect_device(&self, device_id: u64);
    fn send_component_event(&self, component_event: InputEvent);
    fn get_path(&self, path_string: &str) -> Result<SuPath, PathFormatError>;
    fn get_path_string(&self, path: SuPath) -> Option<String>;
}

pub trait DriverInterface: Send + Sync {
    //Force a refresh
    fn poll(&self);
    //TODO
    fn get_component_state(&self, device: usize, path: SuPath) -> ();
    fn destroy(&mut self);
}
