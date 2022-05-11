use std::sync::Arc;

use parking_lot::RwLock;
use suinput::{
    driver_interface::{DriverInterface, DriverRuntimeInterface, DriverRuntimeInterfaceTrait},
    event::PathManager,
    SuPath,
};
use winit::{
    event::{DeviceEvent, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::windows::WindowExtWindows,
    window::WindowBuilder,
};

#[derive(Debug)]
pub struct DummyDriverManager(pub Arc<RwLock<PathManager>>, pub RwLock<Vec<SuPath>>);

impl DriverRuntimeInterfaceTrait for DummyDriverManager {
    fn register_new_device(&self, device_type: SuPath) -> u64 {
        let mut vec = self.1.try_write().unwrap();
        vec.push(device_type);
        (vec.len() - 1) as u64
    }

    fn disconnect_device(&self, device_id: u64) {
        println!(
            "{device_id} ({:?}) disconnected ",
            self.1.try_read().unwrap().get(device_id as usize)
        );
    }

    fn send_component_event(&self, component_event: suinput::event::InputEvent) {
        println!(
            "Input Event {{ device: {:?}, path: '{}', data: {:?} }}",
            component_event.device,
            self.get_path_string(component_event.path).unwrap(),
            component_event.data,
        );
    }

    fn get_path(&self, path_string: &str) -> Result<SuPath, suinput::event::PathFormatError> {
        self.0.write().get_path(path_string)
    }

    fn get_path_string(&self, path: SuPath) -> Option<String> {
        self.0.read().get_path_string(path)
    }
}

fn main() -> Result<(), anyhow::Error> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop)?;

    let path_manager = Arc::new(RwLock::new(PathManager::default()));

    let runtime_interface = DriverRuntimeInterface(Arc::new(DummyDriverManager(
        path_manager,
        RwLock::new(Vec::new()),
    )));

    let mut windows_driver =
        windows_driver::Win32DesktopDriver::initialize(runtime_interface, true, true)?;

    windows_driver.set_windows(&[window.hwnd() as _]);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => {
                *control_flow = ControlFlow::Exit;
                windows_driver.destroy();
            }
            Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta: _, .. },
                window_id,
            } if window_id == window.id() => {
                // println!("Winit Scroll Event {delta:?}");
            }
            Event::DeviceEvent {
                event: DeviceEvent::MouseWheel { delta },
                device_id,
            } => {
                panic!("Winit Raw Input Scroll Event {device_id:?}-{delta:?}");
            }
            // Event::WindowEvent { event: WindowEvent::ReceivedCharacter(char), .. } => {
            //     window.set_cursor_position(LogicalPosition::new(0, 10)).unwrap();
            //     println!("{char}");
            // }
            _ => (),
        }
    });
}
