use std::{
    ops::Deref,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::JoinHandle,
    time::{Duration, Instant}, path::Path,
};

use flume::Sender;
use itertools::Itertools;
use parking_lot::{Mutex, RwLock};

use suinput_types::{
    controller_paths::GameControllerPaths, event::InputEvent, event::PathFormatError,
    keyboard::KeyboardPaths, SuPath,
};
use thunderdome::Arena;

use crate::{
    driver_interface::*,
    internal::{
        device_types::DeviceTypes,
        interaction_profile_types::InteractionProfileTypes,
        paths::{CommonPaths, PathManager},
        worker_thread::{self, WorkerThreadEvent},
    },
    session::Session,
};

use super::instance::Instance;

pub struct Runtime {
    pub(crate) paths: Arc<PathManager>,
    pub(crate) common_paths: CommonPaths,
    pub(crate) controller_paths: GameControllerPaths,
    pub(crate) device_types: DeviceTypes,
    pub(crate) interaction_profile_types: InteractionProfileTypes,

    pub(crate) worker_thread_sender: Sender<worker_thread::WorkerThreadEvent>,
    _thread: JoinHandle<()>,
    pub(crate) driver_response_senders: Mutex<Vec<Sender<Driver2RuntimeEventResponse>>>,
    drivers: RwLock<Vec<Box<dyn SuInputDriver>>>,

    pub(crate) instances: RwLock<Vec<Arc<Instance>>>,
    pub(crate) sessions: RwLock<Arena<Arc<Session>>>,
}

impl Runtime {
    pub fn new() -> Arc<Self> {
        let (worker_thread_sender, worker_thread_receiver) = flume::bounded(100);

        let paths = Arc::new(PathManager::new());

        let common_paths = CommonPaths::new(|str| paths.get_path(str).unwrap());
        let keyboard_paths = KeyboardPaths::new(|str| paths.get_path(str).unwrap());
        let controller_paths = GameControllerPaths::new(|str| paths.get_path(str).unwrap());
        let device_types = DeviceTypes::new(&common_paths, &keyboard_paths, &paths);
        let interaction_profile_types =
            InteractionProfileTypes::new(&device_types, |str| paths.get_path(str).unwrap());

        let ready = Arc::new(Mutex::new(()));
        let lock = ready.lock();

        let runtime = Arc::new_cyclic(|arc| Self {
            worker_thread_sender,
            paths,
            _thread: worker_thread::spawn_thread(
                worker_thread_receiver,
                arc.to_owned(),
                ready.clone(),
            ),
            drivers: Default::default(),
            driver_response_senders: Default::default(),
            instances: Default::default(),
            device_types,
            common_paths,
            interaction_profile_types,
            sessions: Default::default(),
            controller_paths,
        });

        std::mem::drop(lock);

        runtime
    }

    pub fn add_driver<F, T, E>(&self, f: F) -> Result<usize, E>
    where
        F: FnOnce(RuntimeInterface) -> Result<T, E>,
        T: SuInputDriver + 'static,
    {
        let (runtime2driver_sender, runtime2driver_receiver) = flume::bounded(1);

        let idx = self.drivers.read().len();

        let runtime_interface = Arc::new(EmbeddedDriverRuntimeInterface {
            ready: AtomicBool::new(false),
            paths: self.paths.clone(),
            sender: self.worker_thread_sender.clone(),
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

    pub fn refresh_windows(&self) {
        let sessions = self.sessions.read();
        let window_guards = sessions
            .iter()
            .map(|(_, session)| session.window.lock())
            .collect::<Vec<_>>();

        let windows = window_guards
            .iter()
            .filter_map(|guard| *guard.deref())
            .map(|non_zero| non_zero.into())
            .unique()
            .collect::<Vec<usize>>();

        for driver in self.drivers.write().iter_mut() {
            driver.set_windows(&windows);
        }
    }

    pub fn create_instance(self: &Arc<Self>, storage_path: Option<&Path>) -> Arc<Instance> {
        let mut instances = self.instances.write();
        let instance = Arc::new(Instance::new(self, instances.len() as u64 + 1, storage_path));
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
            .send(WorkerThreadEvent::Driver {
                id: self.idx,
                event: Driver2RuntimeEvent::RegisterDevice(device_type),
            })
            .unwrap();

        match self
            .receiver
            .recv_deadline(Instant::now() + Duration::from_secs(5))
            .unwrap()
        {
            Driver2RuntimeEventResponse::DeviceId(id) => Ok(id),
        }
    }

    fn disconnect_device(&self, device_id: u64) -> Result<(), RuntimeInterfaceError> {
        if !self.ready.load(Ordering::Relaxed) {
            return Err(RuntimeInterfaceError::DriverUninitialized);
        }

        self.sender
            .send(WorkerThreadEvent::Driver {
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
            .send(WorkerThreadEvent::Driver {
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

    fn start_batch_input_update(&self, device: u64, time: Instant) -> BatchInputUpdate {
        //TODO cache BatchInputUpdate to prevent extra heap allocations
        BatchInputUpdate::new(device, time)
    }

    fn send_batch_input_update(
        &self,
        batch_update: BatchInputUpdate,
    ) -> Result<(), RuntimeInterfaceError> {
        if !self.ready.load(Ordering::Relaxed) {
            return Err(RuntimeInterfaceError::DriverUninitialized);
        }

        self.sender
            .send(WorkerThreadEvent::Driver {
                id: self.idx,
                event: Driver2RuntimeEvent::BatchInput(batch_update),
            })
            .unwrap();
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Driver2RuntimeEventResponse {
    DeviceId(u64),
}

#[derive(Debug, Clone)]
pub enum Driver2RuntimeEvent {
    RegisterDevice(SuPath),
    DisconnectDevice(u64),
    Input(InputEvent),
    BatchInput(BatchInputUpdate),
}
