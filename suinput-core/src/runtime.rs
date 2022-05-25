use std::{
    ops::{Add, Deref},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

use dashmap::DashMap;
use flume::{Receiver, Sender};
use parking_lot::{Mutex, RwLock};

use regex::Regex;
use suinput_types::{
    driver_interface::{
        DriverInterface, RuntimeInterface, RuntimeInterfaceError, RuntimeInterfaceTrait,
    },
    event::{InputEvent, PathFormatError},
    SuPath,
};
use thunderdome::{Arena, Index};

use crate::{
    instance,
    interaction_profile::{DeviceState, InteractionProfile, InteractionProfileState},
};

use super::instance::Instance;

pub struct Runtime {
    driver2runtime_sender: Sender<(usize, Driver2RuntimeEvent)>,
    pub(crate) paths: Arc<PathManager>,
    _thread: JoinHandle<()>,
    shared_state: Arc<RuntimeState>,
    drivers: RwLock<Vec<Box<dyn DriverInterface>>>,

    instances: Arc<RwLock<Vec<Arc<Instance>>>>,
}

#[derive(Default)]
struct RuntimeState {
    driver_response_senders: Mutex<Vec<Sender<Driver2RuntimeEventResponse>>>,
}

impl Runtime {
    pub fn new() -> Self {
        let (driver2runtime_sender, driver2runtime_receiver) = flume::bounded(100);

        let paths = Arc::new(PathManager::new());
        let shared_state = Arc::<RuntimeState>::default();
        let instances = Arc::new(RwLock::default());
        let input_thread = spawn_thread(
            driver2runtime_receiver,
            shared_state.clone(),
            paths.clone(),
            instances.clone(),
        );

        Self {
            driver2runtime_sender,
            paths,
            _thread: input_thread,
            // devices,
            drivers: Default::default(),
            // driver_response_senders,
            shared_state,
            instances,
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
        let instance = Arc::new(Instance::new(self, name, ()));
        self.instances.write().push(instance.clone());
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
    paths: Arc<PathManager>,
    instances: Arc<RwLock<Vec<Arc<Instance>>>>,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let mut devices = Arena::<(SuPath, DeviceState)>::new();

        let mut kbd_mse_csr_profile = InteractionProfileState::new(
            InteractionProfile::new_keyboard_mouse_cursor_profile(&paths),
        );

        while let Ok((driver, event)) = driver2runtime_receiver.recv() {
            // println!("{:?}", event);

            match event {
                Driver2RuntimeEvent::RegisterDevice(ty) => {
                    //TODO: Device ID persistence
                    let device_id = devices.insert((ty, DeviceState::default()));

                    state
                        .driver_response_senders
                        .lock()
                        .get(driver)
                        .expect("Could not access driver response channel")
                        .send(Driver2RuntimeEventResponse::DeviceId(device_id.to_bits()))
                        .expect("Driver response channel closed unexpectedly");

                    kbd_mse_csr_profile.device_added(device_id, ty);
                }
                Driver2RuntimeEvent::Input(event) => {
                    let instances = instances.read();
                    kbd_mse_csr_profile.update_component(&event, &devices, &instances);

                    let device = devices
                        .get_mut(Index::from_bits(event.device).unwrap())
                        .unwrap();
                    let device_state = &mut device.1;

                    device_state.update_input(event);
                }
                Driver2RuntimeEvent::DisconnectDevice(id) => {
                    let index = Index::from_bits(id).unwrap();

                    kbd_mse_csr_profile.device_removed(index, &devices);
                    devices.remove(index);
                }
            }
        }
    })
}

#[derive(Debug)]
pub struct PathManager(DashMap<String, SuPath>, DashMap<SuPath, String>, Regex);

impl PathManager {
    pub fn new() -> Self {
        let regex = Regex::new(r#"^(/(\.*[a-z0-9-_]+\.*)+)+$"#).unwrap();
        Self(DashMap::new(), DashMap::new(), regex)
    }

    pub fn get_path(&self, path_string: &str) -> Result<SuPath, PathFormatError> {
        if let Some(path) = self.0.get(path_string) {
            return Ok(*path.deref());
        }

        if self.2.is_match(path_string) {
            let path = SuPath(self.0.len() as u32);
            self.0.insert(path_string.to_owned(), path);
            self.1.insert(path, path_string.to_owned());
            Ok(path)
        } else {
            Err(PathFormatError)
        }
    }

    pub fn get_path_string(&self, path: SuPath) -> Option<String> {
        self.1.get(&path).map(|inner| inner.clone())
    }
}

/**
 * DriverRuntimeInterface implementation for rust drivers embedded into the runtime
 */
#[derive(Debug)]
pub struct EmbeddedDriverRuntimeInterface {
    ready: AtomicBool,
    paths: Arc<PathManager>,
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
        self.paths.get_path(path_string)
    }

    fn get_path_string(&self, path: SuPath) -> Option<String> {
        self.paths.get_path_string(path)
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
