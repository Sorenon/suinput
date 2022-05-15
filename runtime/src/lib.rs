use std::{
    ops::{Add, Deref},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

use flume::{Receiver, Sender};
use parking_lot::{Mutex, RwLock};
use raw_window_handle::RawWindowHandle;
use suinput::{
    driver_interface::{
        DriverInterface, RuntimeInterface, RuntimeInterfaceError, RuntimeInterfaceTrait,
    },
    event::{InputEvent, PathFormatError, PathManager},
    SuPath,
};
use thunderdome::Arena;

//Should the runtime API be multithreaded friendly without the need for external locks?
//Pros:
//Increases FFI stability due to lack of Fearless Concurrency
//Simplifies engine integration
//Cons:
//Increases implementation complexity
//Might only be needed for the action api
pub struct SuInputRuntime {
    driver2runtime_sender: Sender<(usize, Driver2RuntimeEvent)>,
    paths: Arc<RwLock<PathManager>>,
    thread: JoinHandle<()>,
    shared_state: Arc<RuntimeState>,
    drivers: Vec<Box<dyn DriverInterface>>,
}

#[derive(Default)]
struct RuntimeState {
    devices: RwLock<Arena<SuPath>>,
    driver_response_senders: Mutex<Vec<Sender<Driver2RuntimeEventResponse>>>,
}

impl SuInputRuntime {
    pub fn new() -> Self {
        let (driver2runtime_sender, driver2runtime_receiver) = flume::bounded(100);

        let shared_state = Arc::<RuntimeState>::default();
        let input_thread = spawn_thread(driver2runtime_receiver, shared_state.clone());

        Self {
            driver2runtime_sender,
            paths: Arc::new(RwLock::new(PathManager::default())),
            thread: input_thread,
            // devices,
            drivers: Vec::new(),
            // driver_response_senders,
            shared_state,
        }
    }

    pub fn add_driver<F, T, E>(&mut self, f: F) -> Result<usize, E>
    where
        F: FnOnce(RuntimeInterface) -> Result<T, E>,
        T: DriverInterface + 'static,
    {
        let (runtime2driver_sender, runtime2driver_receiver) = flume::bounded(1);

        let idx = self.drivers.len();

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
        self.drivers.push(Box::new(driver));
        runtime_interface.ready.store(true, Ordering::Relaxed);
        self.drivers.get_mut(idx).unwrap().initialize();
        Ok(idx)
    }

    pub fn set_windows(&mut self, windows: &[usize]) {
        for driver in &mut self.drivers {
            driver.set_windows(windows);
        }
    }

    pub fn set_windows_rwh(&mut self, raw_window_handles: &[RawWindowHandle]) {
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

    pub fn destroy(&mut self) {
        for driver in &mut self.drivers {
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
