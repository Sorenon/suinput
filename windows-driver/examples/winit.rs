use windows_sys::Win32::UI::WindowsAndMessaging::UnhookWindowsHookEx;
use winit::{
    event::{DeviceEvent, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder, dpi::LogicalPosition,
};

fn main() -> Result<(), anyhow::Error> {
    unsafe {
        let hook_handle = windows_driver::hooks::inject_hook()?;

        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop)?;

        std::thread::spawn(windows_driver::raw_input::window::run);

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id,
                } if window_id == window.id() => {
                    *control_flow = ControlFlow::Exit;
                    UnhookWindowsHookEx(hook_handle);
                }
                Event::WindowEvent {
                    event: WindowEvent::MouseWheel { delta, .. },
                    window_id,
                } if window_id == window.id() => {
                    println!("Winit Scroll Event {delta:?}");
                }
                Event::DeviceEvent {
                    event: DeviceEvent::MouseWheel { delta },
                    device_id,
                } => {
                    panic!("Winit Raw Input Scroll Event {device_id:?}-{delta:?}");
                }
                Event::WindowEvent { event: WindowEvent::ReceivedCharacter(char), .. } => {
                    window.set_cursor_position(LogicalPosition::new(0, 10)).unwrap();
                    println!("{char}");
                }
                _ => (),
            }
        });
    }
}
