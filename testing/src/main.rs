use std::sync::Arc;

use suinput::{
    driver_interface::{DriverRuntimeInterface, EmbeddedDriverRuntimeInterface},
    SuInputRuntime,
};
use winit::{
    dpi::LogicalPosition,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop)?;

    let runtime = SuInputRuntime::new();

    let mut windows_driver = windows_driver::WindowsDesktopDriver::initialize(
        DriverRuntimeInterface(Arc::new(EmbeddedDriverRuntimeInterface {
            paths: runtime.paths,
            sender: runtime.driver2runtime_sender,
        })),
    )?;

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
                event: WindowEvent::ReceivedCharacter(char),
                ..
            } => {
                window
                    .set_cursor_position(LogicalPosition::new(0, 10))
                    .unwrap();
                println!("{char}");
            }
            _ => (),
        }
    });
}
