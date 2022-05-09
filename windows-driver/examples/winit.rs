use std::sync::{Arc, RwLock};

use suinput::{
    driver_interface::{DriverRuntimeInterface, DriverRuntimeInterfaceTrait},
    event::PathManager,
    SuPath,
};
use winit::{
    event::{DeviceEvent, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[derive(Debug)]
pub struct DummyDriverManager(pub Arc<RwLock<PathManager>>);

impl DriverRuntimeInterfaceTrait for DummyDriverManager {
    fn send_device_event(&self, device_event: suinput::event::DeviceEvent) {
        println!("{:?}", device_event);
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
        self.0.try_write().unwrap().get_path(path_string)
    }

    fn get_path_string(&self, path: SuPath) -> Option<String> {
        self.0.try_read().unwrap().get_path_string(path)
    }
}

fn main() -> Result<(), anyhow::Error> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop)?;

    let path_manager = Arc::new(RwLock::new(PathManager::default()));

    let runtime_interface = DriverRuntimeInterface(Arc::new(DummyDriverManager(path_manager)));

    let mut windows_driver = windows_driver::WindowsDesktopDriver::initialize(runtime_interface)?;

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
                event: WindowEvent::MouseWheel { delta, .. },
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
