use std::{
    ops::Add,
    sync::Arc,
    thread::JoinHandle,
    time::{Duration, Instant},
};

use flume::{Receiver, Sender};
use parking_lot::{Mutex, RwLock};
use suinput::{
    driver_interface::{DriverInterface, DriverRuntimeInterface, DriverRuntimeInterfaceTrait},
    event::{InputEvent, PathFormatError, PathManager},
    SuPath,
};
use thunderdome::Arena;

type DriverResponseSenders = Arc<Mutex<Vec<Sender<Driver2RuntimeEventResponse>>>>;

//TODO make SuInputRuntime Send + Sync
pub struct SuInputRuntime {
    driver2runtime_sender: Sender<(usize, Driver2RuntimeEvent)>,
    paths: Arc<RwLock<PathManager>>,
    thread: JoinHandle<()>,
    devices: Arc<RwLock<Arena<SuPath>>>,
    drivers: Vec<Box<dyn DriverInterface>>,
    driver_response_senders: DriverResponseSenders,
}

impl SuInputRuntime {
    pub fn new() -> Self {
        let (driver2runtime_sender, driver2runtime_receiver) = flume::bounded(100);

        let devices = Arc::new(RwLock::new(Arena::new()));
        let driver_response_senders = DriverResponseSenders::default();
        let input_thread = spawn_thread(
            driver_response_senders.clone(),
            driver2runtime_receiver,
            devices.clone(),
        );

        Self {
            driver2runtime_sender,
            paths: Arc::new(RwLock::new(PathManager::default())),
            thread: input_thread,
            devices,
            drivers: Vec::new(),
            driver_response_senders,
        }
    }

    //TODO improve this
    pub fn add_driver<F, T, E>(&mut self, f: F) -> Result<usize, E>
    where
        F: FnOnce(DriverRuntimeInterface) -> Result<T, E>,
        T: DriverInterface + 'static,
    {
        let (runtime2driver_sender, runtime2driver_receiver) = flume::bounded(1);

        {
            self.driver_response_senders
                .lock()
                .push(runtime2driver_sender);
        }

        match f(DriverRuntimeInterface(Arc::new(
            EmbeddedDriverRuntimeInterface {
                paths: self.paths.clone(),
                sender: self.driver2runtime_sender.clone(),
                idx: self.drivers.len(),
                receiver: runtime2driver_receiver,
            },
        ))) {
            Ok(driver) => {
                self.drivers.push(Box::new(driver));
                Ok(self.drivers.len() - 1)
            }
            Err(err) => {
                self.driver_response_senders.lock().pop();
                Err(err)
            }
        }
    }

    pub fn destroy(&mut self) {
        for driver in &mut self.drivers {
            driver.destroy()
        }
    }
}

fn spawn_thread(
    driver_response_senders: DriverResponseSenders,
    driver2runtime_receiver: Receiver<(usize, Driver2RuntimeEvent)>,
    devices: Arc<RwLock<Arena<SuPath>>>,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        while let Ok((driver, event)) = driver2runtime_receiver.recv() {
            println!("{:?}", event);

            match event {
                Driver2RuntimeEvent::RegisterDevice(ty) => {
                    //Current task: Device ID persistence

                    println!("s");

                    let device_id = devices.write().insert(ty);
                    driver_response_senders
                        .lock()
                        .get(driver)
                        .unwrap()
                        .send(Driver2RuntimeEventResponse::DeviceId(device_id.to_bits()))
                        .unwrap();

                    println!("s2");
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
    paths: Arc<RwLock<PathManager>>,
    sender: flume::Sender<(usize, Driver2RuntimeEvent)>,
    receiver: flume::Receiver<Driver2RuntimeEventResponse>,
    idx: usize,
}

impl DriverRuntimeInterfaceTrait for EmbeddedDriverRuntimeInterface {
    fn register_new_device(&self, device_type: SuPath) -> u64 {
        self.sender
            .send((self.idx, Driver2RuntimeEvent::RegisterDevice(device_type)))
            .unwrap();

        println!("r");

        match self
            .receiver
            .recv_deadline(Instant::now().add(Duration::from_secs(5)))
            .unwrap()
        {
            Driver2RuntimeEventResponse::DeviceId(id) => return id,
        }
    }

    fn disconnect_device(&self, device_id: u64) {
        self.sender
            .send((self.idx, Driver2RuntimeEvent::DisconnectDevice(device_id)))
            .unwrap()
    }

    fn send_component_event(&self, component_event: InputEvent) {
        self.sender
            .send((self.idx, Driver2RuntimeEvent::Input(component_event)))
            .unwrap()
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
