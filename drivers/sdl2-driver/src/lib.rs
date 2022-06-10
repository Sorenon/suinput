use std::{
    collections::HashMap,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use nalgebra::Vector2;
use sdl2_sys::{
    SDL_GameControllerType, SDL_InitSubSystem, SDL_QuitSubSystem, SDL_SensorType,
    SDL_INIT_GAMECONTROLLER,
};

mod concurrent_sdl;

use concurrent_sdl::*;
use game_controller::*;
use joystick::*;
use suinput_types::{
    controller_paths::GameControllerPaths,
    driver_interface::{RuntimeInterface, SuInputDriver},
    event::{InputComponentEvent, InputEvent},
    SuPath, Time,
};
/*
    TODO sort out the controller dupe bug when using winit
    I would sort this out now but with GDK Input around the corner I don't know if my time will have been wasted

    Notes:
    The bug only occurs when running the winit event loop
    If we can't fix it in SDL we could just compare the joystick's system ids to the ones exposed by win32
    Hopefully GDK input will mitigate this problem

    Should I make a standalone alternative to SDL2's gamecontroller support?
    Pros:
    Can be pure rust for added simplicity and safety
    Can tweak as much as needed with no fear of breaking backwards compatibility
    Weaker link to the somewhat limited SDL GameController Database
    Smaller binary size
    Tighter integration with SuInput's device relationship system
    Smaller chance of conflicting with some Game Engines' existing controller support
    Cons:
    Large undertaking
    Requires purchasing and testing each exotic controller type SDL supports
    Looses the Steam integration
    May require purchasing a Mac
*/

pub struct SDLGameControllerGenericDriver {
    interface: RuntimeInterface,
    destroyed: AtomicBool,
}

impl SDLGameControllerGenericDriver {
    ///Should be called from the main thread
    pub fn new(background: bool, interface: RuntimeInterface) -> core::result::Result<Self, ()> {
        set_hint("SDL_HINT_GAMECONTROLLER_USE_BUTTON_LABELS", "0");
        set_hint(
            "SDL_HINT_JOYSTICK_ALLOW_BACKGROUND_EVENTS",
            if background { "1" } else { "0" },
        );
        set_hint(
            "SDL_HINT_JOYSTICK_HIDAPI_XBOX",
            if background || !cfg!(windows) {
                "1"
            } else {
                "0"
            },
        );
        set_hint("SDL_HINT_JOYSTICK_HIDAPI_JOY_CONS", "1");
        set_hint("SDL_HINT_JOYSTICK_HIDAPI_SWITCH_HOME_LED", "0");
        set_hint("SDL_HINT_JOYSTICK_HIDAPI_PS4_RUMBLE", "1");
        set_hint("SDL_HINT_JOYSTICK_HIDAPI_PS5_RUMBLE", "1");
        set_hint("SDL_HINT_JOYSTICK_THREAD", "1");

        unsafe {
            //Automatically loads the joystick subsystem
            assert_eq!(SDL_InitSubSystem(SDL_INIT_GAMECONTROLLER), 0);
        }

        Ok(Self {
            destroyed: AtomicBool::new(false),
            interface,
        })
    }

    ///Should be called from the main thread
    pub fn destroy(&self) {
        if self.destroyed.load(Ordering::Relaxed) {
            println!("WARN: Tried to destroy SDL driver twice");
            return;
        }

        unsafe {
            //Automatically quits the joystick subsystem
            SDL_QuitSubSystem(SDL_INIT_GAMECONTROLLER);
        }

        self.destroyed.store(true, Ordering::Relaxed);
    }
}

impl SuInputDriver for SDLGameControllerGenericDriver {
    fn initialize(&mut self) {
        let interface = self.interface.clone();
        std::thread::spawn(|| {
            println!("start");
            let mut thread_state = ThreadState {
                paths: GameControllerPaths::new(|str| interface.get_path(str).unwrap()),
                interface,
                old_joysticks: Vec::new(),
                game_controllers: HashMap::new(),
            };

            loop {
                thread_state.tick();

                //TODO spin sleep
                std::thread::sleep(Duration::from_millis(3));
            }
        });
    }

    fn poll(&self) {
        todo!()
    }

    fn get_component_state(&self, device: usize, path: suinput_types::SuPath) -> () {
        todo!()
    }

    fn destroy(&mut self) {
        //TODO
    }
}

impl Drop for SDLGameControllerGenericDriver {
    fn drop(&mut self) {
        assert!(self.destroyed.load(Ordering::Relaxed))
    }
}

struct ThreadState {
    interface: RuntimeInterface,
    paths: GameControllerPaths,
    old_joysticks: Vec<SdlJoystick>,
    game_controllers: HashMap<SdlJoystick, (ControllerDevice, DeviceState)>,
}

impl ThreadState {
    fn tick(&mut self) {
        self.check_controllers();

        update_game_controllers();
        let _lock = lock_joystick_system();

        // let lock_time = Instant::now();

        for (controller, state) in self.game_controllers.values_mut() {
            if controller.sdl.get_type() == SDL_GameControllerType::SDL_CONTROLLER_TYPE_PS5 {
                state.update(&controller, &self.interface, &self.paths);
            }
        }
    }

    fn check_controllers(&mut self) {
        update_game_controllers();

        let _lock = lock_joystick_system();

        let num_joysticks = num_joysticks().unwrap();

        let new_joysticks = (0..num_joysticks)
            .map(|device_index| get_instance_id(device_index).unwrap())
            .collect::<Vec<_>>();

        if num_joysticks != self.old_joysticks.len() as u32
            || self
                .old_joysticks
                .iter()
                .find(|instance_id| !new_joysticks.contains(&instance_id))
                .is_some()
        {
            self.game_controllers
                .retain(|joystick_id, _| new_joysticks.contains(joystick_id));

            for (device_index, joystick) in new_joysticks.iter().enumerate() {
                if !self.game_controllers.contains_key(joystick) {
                    let device_index = device_index as u32;
                    if is_game_controller(device_index) {
                        self.game_controllers.insert(
                            device_index,
                            (
                                ControllerDevice::new(device_index, &self.interface, &self.paths),
                                DeviceState::default(),
                            ),
                        );
                    }
                }
            }

            self.old_joysticks = new_joysticks;
        }
    }
}

#[derive(Debug)]
struct ControllerDevice {
    sdl: GameController,
    has_gyro: bool,
    has_accel: bool,
    has_touchpad: bool,

    idx: Option<u64>,

    start: SuPath,
    back: SuPath,
    misc1: SuPath,
}

impl ControllerDevice {
    pub fn new(
        device_index: SdlDeviceIndex,
        interface: &RuntimeInterface,
        paths: &GameControllerPaths,
    ) -> Self {
        let sdl = open_game_controller(device_index).unwrap();

        let has_gyro = sdl.has_sensor(SDL_SensorType::SDL_SENSOR_GYRO);
        let has_accel = sdl.has_sensor(SDL_SensorType::SDL_SENSOR_ACCEL);
        let has_touchpad = true; //TODO

        if has_gyro {
            sdl.set_sensor_state(SDL_SensorType::SDL_SENSOR_GYRO, true)
                .unwrap();
        }

        if has_accel {
            sdl.set_sensor_state(SDL_SensorType::SDL_SENSOR_ACCEL, true)
                .unwrap();
        }

        println!("{} connected", sdl.get_name());

        let idx = if sdl.get_type() == SDL_GameControllerType::SDL_CONTROLLER_TYPE_PS5 {
            Some(
                interface
                    .register_new_device(paths.device_dual_sense)
                    .unwrap(),
            )
        } else {
            None
        };

        Self {
            // state: DualSense::default(),
            sdl,
            has_gyro,
            has_accel,
            idx,
            has_touchpad,
            back: paths.create,
            start: paths.options,
            misc1: paths.mute,
        }
    }
}

enum ControllerType {
    Xbox360,
    //Haptic Triggers
    XboxOne,
    //Has share button
    XboxOneSeriesSX,
    //Back paddles
    XboxElite,

    DualShock3,
    //Touchpad,
    DualShock4,
    //HD Rumble, Adaptive Triggers
    DualSense,

    SwitchPro,
    LeftJoycon,
    //IR and NFC
    RightJoycon,
    GameCube,

    //Has an extra 'alexa' button
    Luna,
    //Has special stadia buttons
    Stadia,

    //For these should we use a HID Driver instead?

    //SDL_GameControllerDB, essentially just an Xbox Elite controller
    Generic,
    //Lets users manually map axis and buttons to named paths
    //This could end up being linked to an SDL_GameControllerDB-like database
    Custom(/*TODO*/),
}

const SDL_BUTTON_NUM: usize =
    sdl2_sys::SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_TOUCHPAD as usize + 1;

#[derive(Debug, Default)]
struct DeviceState {
    buttons: [bool; SDL_BUTTON_NUM],
    left_thumbstick: Vector2<f32>,
    right_thumbstick: Vector2<f32>,
    left_trigger: f32,
    right_trigger: f32,

    touchpad_1: TouchpadFinger,
    touchpad_2: TouchpadFinger,
}

impl DeviceState {
    pub fn update(
        &mut self,
        device: &ControllerDevice,
        interface: &RuntimeInterface,
        paths: &GameControllerPaths,
    ) {
        use sdl2_sys::SDL_GameControllerButton as Button;

        for button_idx in 0..SDL_BUTTON_NUM {
            let (button, path) = match button_idx {
                0 => (Button::SDL_CONTROLLER_BUTTON_A, paths.diamond_down),
                1 => (Button::SDL_CONTROLLER_BUTTON_B, paths.diamond_right),
                2 => (Button::SDL_CONTROLLER_BUTTON_X, paths.diamond_left),
                3 => (Button::SDL_CONTROLLER_BUTTON_Y, paths.diamond_up),
                4 => (Button::SDL_CONTROLLER_BUTTON_BACK, device.back),
                5 => (Button::SDL_CONTROLLER_BUTTON_GUIDE, paths.guide),
                6 => (Button::SDL_CONTROLLER_BUTTON_START, device.start),
                7 => (
                    Button::SDL_CONTROLLER_BUTTON_LEFTSTICK,
                    paths.left_stick_click,
                ),
                8 => (
                    Button::SDL_CONTROLLER_BUTTON_RIGHTSTICK,
                    paths.right_stick_click,
                ),
                9 => (
                    Button::SDL_CONTROLLER_BUTTON_LEFTSHOULDER,
                    paths.left_shoulder,
                ),
                10 => (
                    Button::SDL_CONTROLLER_BUTTON_RIGHTSHOULDER,
                    paths.right_shoulder,
                ),
                11 => (Button::SDL_CONTROLLER_BUTTON_DPAD_UP, paths.dpad_up),
                12 => (Button::SDL_CONTROLLER_BUTTON_DPAD_DOWN, paths.dpad_down),
                13 => (Button::SDL_CONTROLLER_BUTTON_DPAD_LEFT, paths.dpad_left),
                14 => (Button::SDL_CONTROLLER_BUTTON_DPAD_RIGHT, paths.dpad_right),
                15 => (Button::SDL_CONTROLLER_BUTTON_MISC1, device.misc1),
                16 => (Button::SDL_CONTROLLER_BUTTON_PADDLE1, paths.paddle1),
                17 => (Button::SDL_CONTROLLER_BUTTON_PADDLE2, paths.paddle2),
                18 => (Button::SDL_CONTROLLER_BUTTON_PADDLE3, paths.paddle3),
                19 => (Button::SDL_CONTROLLER_BUTTON_PADDLE4, paths.paddle4),
                20 => (Button::SDL_CONTROLLER_BUTTON_TOUCHPAD, paths.touchpad_click),
                _ => unreachable!(),
            };

            let state = device.sdl.get_button(button);
            if state != self.buttons[button_idx] {
                interface
                    .send_component_event(InputEvent {
                        device: device.idx.unwrap(),
                        path: path,
                        time: Time(0),
                        data: InputComponentEvent::Button(state),
                    })
                    .unwrap();
                self.buttons[button_idx] = state;
            }
        }

        let left_thumbstick = device.sdl.get_thumbstick(true);
        if left_thumbstick != self.left_thumbstick {
            interface
                .send_component_event(InputEvent {
                    device: device.idx.unwrap(),
                    path: paths.left_joystick,
                    time: Time(0),
                    data: InputComponentEvent::Joystick(left_thumbstick.into()),
                })
                .unwrap();
            self.left_thumbstick = left_thumbstick;
        }

        let right_thumbstick = device.sdl.get_thumbstick(false);
        if right_thumbstick != self.right_thumbstick {
            interface
                .send_component_event(InputEvent {
                    device: device.idx.unwrap(),
                    path: paths.right_joystick,
                    time: Time(0),
                    data: InputComponentEvent::Joystick(right_thumbstick.into()),
                })
                .unwrap();
            self.right_thumbstick = right_thumbstick;
        }

        let left_trigger = device.sdl.get_trigger(true);
        if left_trigger != self.left_trigger {
            interface
                .send_component_event(InputEvent {
                    device: device.idx.unwrap(),
                    path: paths.left_trigger,
                    time: Time(0),
                    data: InputComponentEvent::Trigger(left_trigger),
                })
                .unwrap();
            self.left_trigger = left_trigger;
        }

        let right_trigger = device.sdl.get_trigger(false);
        if right_trigger != self.right_trigger {
            interface
                .send_component_event(InputEvent {
                    device: device.idx.unwrap(),
                    path: paths.right_trigger,
                    time: Time(0),
                    data: InputComponentEvent::Trigger(right_trigger),
                })
                .unwrap();
            self.right_trigger = right_trigger;
        }

        if device.has_gyro {
            let gyro = device.sdl.get_gyro_state().unwrap();
            interface
                .send_component_event(InputEvent {
                    device: device.idx.unwrap(),
                    path: paths.gyro,
                    time: Time(0),
                    data: InputComponentEvent::Gyro(gyro.into()),
                })
                .unwrap();
        }

        if device.has_accel {
            let accel = device.sdl.get_accel_state().unwrap();
            interface
                .send_component_event(InputEvent {
                    device: device.idx.unwrap(),
                    path: paths.accel,
                    time: Time(0),
                    data: InputComponentEvent::Accel(accel.into()),
                })
                .unwrap();
        }

        //TODO
        if device.has_touchpad {
            let point_1 = device.sdl.get_touchpad_finger(0, 0);
            if point_1 != self.touchpad_1 {
                self.touchpad_1 = point_1;
            }

            let point_2 = device.sdl.get_touchpad_finger(0, 1);
            if point_2 != self.touchpad_2 {
                self.touchpad_2 = point_2;
            }
        }
    }
}
