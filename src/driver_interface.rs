use std::{
    fmt::Debug,
    ops::Deref,
    sync::{Arc, RwLock},
};

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
    fn send_device_event(&self, device_event: DeviceEvent);
    fn send_component_event(&self, component_event: InputEvent);
    fn get_path(&self, path_string: &str) -> Result<SuPath, PathFormatError>;
    fn get_path_string(&self, path: SuPath) -> Option<String>;
}

/**
 * DriverRuntimeInterface implementation for rust drivers embedded into the runtime
 */
#[derive(Debug)]
pub struct EmbeddedDriverRuntimeInterface {
    pub paths: Arc<RwLock<PathManager>>,
    pub sender: flume::Sender<GenericDriverEvent>,
}

impl DriverRuntimeInterfaceTrait for EmbeddedDriverRuntimeInterface {
    fn send_device_event(&self, device_event: DeviceEvent) {
        self.sender
            .send(GenericDriverEvent::Device(device_event))
            .unwrap()
    }

    fn send_component_event(&self, component_event: InputEvent) {
        self.sender
            .send(GenericDriverEvent::Input(component_event))
            .unwrap()
    }

    fn get_path(&self, path_string: &str) -> Result<SuPath, PathFormatError> {
        self.paths.try_write().unwrap().get_path(path_string)
    }

    fn get_path_string(&self, path: SuPath) -> Option<String> {
        self.paths.try_read().unwrap().get_path_string(path)
    }
}
