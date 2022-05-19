use runtime_api::ActionType;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::windows::WindowExtWindows,
    window::WindowBuilder,
};

fn main() -> Result<(), anyhow::Error> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop)?;

    let runtime = loader::load_runtime();
    runtime.add_driver(windows_driver::Win32DesktopDriver::new)?;
    runtime.set_windows(&[window.hwnd() as _]);

    let instance = runtime.create_instance("Test Instance".into());
    let action_set = instance.create_action_set("my_first_action_set".into(), 0);
    let action = action_set.create_action("my_first_action".into(), ActionType::Boolean);

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
            _ => (),
        }
    });
}