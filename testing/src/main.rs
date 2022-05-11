use runtime::SuInputRuntime;
use winit::{
    dpi::LogicalPosition,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop)?;

    let mut runtime = SuInputRuntime::new();

    runtime.add_driver(|runtime_interface| {
        windows_driver::WindowsDesktopDriver::initialize(runtime_interface, true, true)
    })?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => {
                *control_flow = ControlFlow::Exit;
                runtime.destroy();
            }
            Event::WindowEvent {
                event: WindowEvent::ReceivedCharacter(char),
                ..
            } => {
                window
                    .set_cursor_position(LogicalPosition::new(0, 10))
                    .unwrap();
                println!("c-{char}");
            }
            _ => (),
        }
    });
}
