use std::{
    ops::Add,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

use flume::{Receiver, Sender};
use parking_lot::{Mutex, RwLock};

use suinput::{
    driver_interface::{
        DriverInterface, RuntimeInterface, RuntimeInterfaceError, RuntimeInterfaceTrait,
    },
    event::{InputEvent, PathFormatError, PathManager},
    SuPath,
};
use thunderdome::Arena;

use crate::{SuInstance, SuInstanceEnum};

use super::instance::Instance;

pub struct Runtime {
    driver2runtime_sender: Sender<(usize, Driver2RuntimeEvent)>,
    paths: Arc<RwLock<PathManager>>,
    thread: JoinHandle<()>,
    shared_state: Arc<RuntimeState>,
    drivers: RwLock<Vec<Box<dyn DriverInterface>>>,

    instances: Mutex<Vec<Arc<Instance>>>,
}

#[derive(Default)]
struct RuntimeState {
    devices: RwLock<Arena<SuPath>>,
    driver_response_senders: Mutex<Vec<Sender<Driver2RuntimeEventResponse>>>,
}

impl Runtime {
    pub fn new() -> Self {
        let (driver2runtime_sender, driver2runtime_receiver) = flume::bounded(100);

        let shared_state = Arc::<RuntimeState>::default();
        let input_thread = spawn_thread(driver2runtime_receiver, shared_state.clone());

        Self {
            driver2runtime_sender,
            paths: Arc::new(RwLock::new(PathManager::default())),
            thread: input_thread,
            // devices,
            drivers: Default::default(),
            // driver_response_senders,
            shared_state,
            instances: Default::default(),
        }
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

        self.shared_state
            .driver_response_senders
            .lock()
            .push(runtime2driver_sender);

        {
            let mut drivers = self.drivers.write();
            drivers.push(Box::new(driver));

            runtime_interface.ready.store(true, Ordering::Relaxed);

            drivers
        }
        .get_mut(idx)
        .unwrap()
        .initialize();

        Ok(idx)
    }

    pub fn set_windows(&self, windows: &[usize]) {
        for driver in self.drivers.write().iter_mut() {
            driver.set_windows(windows);
        }
    }

    pub fn create_instance(self: &Arc<Self>, name: String, persistent_unique_id: ()) -> Arc<Instance> {
        let instance = Arc::new(Instance::new(self, name, persistent_unique_id));
        self.instances.lock().push(instance.clone());
        instance
    }

    pub fn destroy(&self) {
        for driver in self.drivers.write().iter_mut() {
            driver.destroy()
        }
    }
}

fn spawn_thread(
    driver2runtime_receiver: Receiver<(usize, Driver2RuntimeEvent)>,
    state: Arc<RuntimeState>,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        while let Ok((driver, event)) = driver2runtime_receiver.recv() {
            println!("{:?}", event);

            match event {
                Driver2RuntimeEvent::RegisterDevice(ty) => {
                    //TODO: Device ID persistence
                    let device_id = state.devices.write().insert(ty);

                    state
                        .driver_response_senders
                        .lock()
                        .get(driver)
                        .unwrap()
                        .send(Driver2RuntimeEventResponse::DeviceId(device_id.to_bits()))
                        .unwrap();
                }
                _ => (),
            }
        }
    })
}

/**
 * DriverRuntimeInterface implementation for rust drivers embedded into the runtime
 */
#[derive(Debug)]
pub struct EmbeddedDriverRuntimeInterface {
    ready: AtomicBool,
    paths: Arc<RwLock<PathManager>>,
    sender: flume::Sender<(usize, Driver2RuntimeEvent)>,
    receiver: flume::Receiver<Driver2RuntimeEventResponse>,
    idx: usize,
}

impl RuntimeInterfaceTrait for EmbeddedDriverRuntimeInterface {
    fn register_new_device(&self, device_type: SuPath) -> Result<u64, RuntimeInterfaceError> {
        if !self.ready.load(Ordering::Relaxed) {
            return Err(RuntimeInterfaceError::DriverUninitialized);
        }

        self.sender
            .send((self.idx, Driver2RuntimeEvent::RegisterDevice(device_type)))
            .unwrap();

        match self
            .receiver
            .recv_deadline(Instant::now().add(Duration::from_secs(5)))
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
            .send((self.idx, Driver2RuntimeEvent::DisconnectDevice(device_id)))
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
            .send((self.idx, Driver2RuntimeEvent::Input(component_event)))
            .unwrap();
        Ok(())
    }

    fn get_path(&self, path_string: &str) -> Result<SuPath, PathFormatError> {
        self.paths.try_write().unwrap().get_path(path_string)
    }

    fn get_path_string(&self, path: SuPath) -> Option<String> {
        self.paths.try_read().unwrap().get_path_string(path)
    }
}

#[derive(Debug, Clone, Copy)]
enum Driver2RuntimeEventResponse {
    DeviceId(u64),
}

#[derive(Debug, Clone, Copy)]
pub enum Driver2RuntimeEvent {
    RegisterDevice(SuPath),
    DisconnectDevice(u64),
    Input(InputEvent),
}
