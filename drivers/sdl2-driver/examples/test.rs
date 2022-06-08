use sdl2::{event::Event, keyboard::Keycode};
use suinput_types::driver_interface::SuInputDriver;

fn main() {
    // let sdl_context = sdl2::init().unwrap();
    // let video = sdl_context.video().unwrap();
    // let _window = video
    //     .window("SDL2 Driver Testing", 400, 300)
    //     .build()
    //     .unwrap();

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

    // driver.destroy();
}
