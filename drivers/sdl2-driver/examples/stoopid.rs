// use suinput_types::driver_interface::SuInputDriver;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    // let event_loop = EventLoop::new();
    // let window = WindowBuilder::new().build(&event_loop).unwrap();

    // let mut driver = sdl2_driver::SDLGameControllerGenericDriver::new(false).unwrap();
    // driver.initialize();

    // let mut event_pump = sdl_context.event_pump().unwrap();

    // 'running: loop {
    //     for event in event_pump.poll_iter() {
    //         match event {
    //             Event::Quit { .. }
    //             | Event::KeyDown {
    //                 keycode: Some(Keycode::Escape),
    //                 ..
    //             } => break 'running,
    //             _ => {}
    //         }
    //     }
    // }

    // event_loop.run(move |event, _, control_flow| {
    //     *control_flow = ControlFlow::Wait;

    //     match event {
    //         Event::WindowEvent {
    //             event: WindowEvent::CloseRequested,
    //             window_id,
    //         } if window_id == window.id() => {
    //             *control_flow = ControlFlow::Exit;
    //             driver.destroy();
    //         }
    //         _ => (),
    //     }
    // });

    // loop {
    //     std::thread::sleep_ms(100000);
    // }
}
