use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

use flume::Sender;
use parking_lot::{Mutex, RwLock};

use suinput_types::{
    driver_interface::{
        DriverInterface, RuntimeInterface, RuntimeInterfaceError, RuntimeInterfaceTrait,
    },
    event::{InputEvent, PathFormatError},
    keyboard::KeyboardPaths,
    SuPath,
};

use crate::internal::{
    device_type::DeviceType,
    device_types::{self, DeviceTypes},
    interaction_profile_types::{self, InteractionProfileTypes},
    paths::{CommonPaths, PathManager},
    worker_thread::{self, WorkerThreadEvent},
};

use super::instance::Instance;

pub struct Runtime {
    pub(crate) paths: Arc<PathManager>,
    pub(crate) common_paths: CommonPaths,
    pub(crate) device_types: DeviceTypes,
    pub(crate) interaction_profile_types: InteractionProfileTypes,

    pub(crate) driver2runtime_sender: Sender<worker_thread::WorkerThreadEvent>,
    _thread: JoinHandle<()>,
    pub(crate) driver_response_senders: Mutex<Vec<Sender<Driver2RuntimeEventResponse>>>,
    drivers: RwLock<Vec<Box<dyn DriverInterface>>>,

    pub(crate) instances: RwLock<Vec<Arc<Instance>>>,
}

impl Runtime {
    pub fn new() -> Arc<Self> {
        let (driver2runtime_sender, driver2runtime_receiver) = flume::bounded(100);

        let paths = Arc::new(PathManager::new());

        let common_paths = CommonPaths::new(|str| paths.get_path(str).unwrap());
        let keyboard_paths = KeyboardPaths::new(|str| paths.get_path(str).unwrap());
        let device_types = DeviceTypes::new(&common_paths, &keyboard_paths);
        let interaction_profile_types =
            InteractionProfileTypes::new(|str| paths.get_path(str).unwrap());

        let ready = Arc::new(Mutex::new(()));
        let lock = ready.lock();

        let runtime = Arc::new_cyclic(|arc| Self {
            driver2runtime_sender,
            paths,
            _thread: worker_thread::spawn_thread(
                driver2runtime_receiver,
                arc.to_owned(),
                ready.clone(),
            ),
            drivers: Default::default(),
            driver_response_senders: Default::default(),
            instances: Default::default(),
            device_types,
            common_paths,
            interaction_profile_types,
        });

        std::mem::drop(lock);

        runtime
    }

    pub fn add_driver<F, T, E>(&self, f: F) -> Result<usize, E>
    where
        F: FnOnce(RuntimeInterface) -> Result<T, E>,
        T: DriverInterface + 'static,
    {
        let (runtime2driver_sender, runtime2driver_receiver) = flume::bounded(1);

        let idx = self.drivers.read().len();

        let runtime_interface = Arc::new(EmbeddedDriverRuntimeInterface {
            ready: AtomicBool::new(false),
            paths: self.paths.clone(),
            sender: self.driver2runtime_sender.clone(),
            idx,
            receiver: runtime2driver_receiver,
        });

        let driver = f(RuntimeInterface(runtime_interface.clone()))?;

        self.driver_response_senders
            .lock()
            .push(runtime2driver_sender);

        {
            let mut drivers = self.drivers.write();
            drivers.push(Box::new(driver));
            runtime_interface.ready.store(true, Ordering::Relaxed);
            drivers.get_mut(idx).unwrap().initialize();
        }

        Ok(idx)
    }

    pub fn set_windows(&self, windows: &[usize]) {
        for driver in self.drivers.write().iter_mut() {
            driver.set_windows(windows);
        }
    }

    pub fn create_instance(self: &Arc<Self>, name: String) -> Arc<Instance> {
        let mut instances = self.instances.write();
        let instance = Arc::new(Instance::new(self, instances.len() as u64 + 1, name, ()));
        instances.push(instance.clone());
        instance
    }

    pub fn destroy(&self) {
        for driver in self.drivers.write().iter_mut() {
            driver.destroy()
        }
    }
}

/**
 * DriverRuntimeInterface implementation for rust drivers embedded into the runtime
 */
#[derive(Debug)]
pub struct EmbeddedDriverRuntimeInterface {
    ready: AtomicBool,
    paths: Arc<PathManager>,
    sender: flume::Sender<worker_thread::WorkerThreadEvent>,
    receiver: flume::Receiver<Driver2RuntimeEventResponse>,
    idx: usize,
}

impl RuntimeInterfaceTrait for EmbeddedDriverRuntimeInterface {
    fn register_new_device(&self, device_type: SuPath) -> Result<u64, RuntimeInterfaceError> {
        if !self.ready.load(Ordering::Relaxed) {
            return Err(RuntimeInterfaceError::DriverUninitialized);
        }

        self.sender
            .send(WorkerThreadEvent::DriverEvent {
                id: self.idx,
                event: Driver2RuntimeEvent::RegisterDevice(device_type),
            })
            .unwrap();

        match self
            .receiver
            .recv_deadline(Instant::now() + Duration::from_secs(5))
            .unwrap()
        {
            Driver2RuntimeEventResponse::DeviceId(id) => return Ok(id),
        }
    }

    fn disconnect_device(&self, device_id: u64) -> Result<(), RuntimeInterfaceError> {
        if !self.ready.load(Ordering::Relaxed) {
            return Err(RuntimeInterfaceError::DriverUninitialized);
        }

        self.sender
            .send(WorkerThreadEvent::DriverEvent {
                id: self.idx,
                event: Driver2RuntimeEvent::DisconnectDevice(device_id),
            })
            .unwrap();
        Ok(())
    }

    fn send_component_event(
        &self,
        component_event: InputEvent,
    ) -> Result<(), RuntimeInterfaceError> {
        if !self.ready.load(Ordering::Relaxed) {
            return Err(RuntimeInterfaceError::DriverUninitialized);
        }

        self.sender
            .send(WorkerThreadEvent::DriverEvent {
                id: self.idx,
                event: Driver2RuntimeEvent::Input(component_event),
            })
            .unwrap();
        Ok(())
    }

    fn get_path(&self, path_string: &str) -> Result<SuPath, PathFormatError> {
        self.paths.get_path(path_string)
    }

    fn get_path_string(&self, path: SuPath) -> Option<String> {
        self.paths.get_path_string(path)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Driver2RuntimeEventResponse {
    DeviceId(u64),
}

#[derive(Debug, Clone, Copy)]
pub enum Driver2RuntimeEvent {
    RegisterDevice(SuPath),
    DisconnectDevice(u64),
    Input(InputEvent),
}
