use std::{
    collections::HashMap,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

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
};
/*
    TODO sort out the controller dupe bug when using winit
    I would sort this out now but with GDK Input around the corner I don't know if my time will have been wasted

    Notes:
    The bug only occurs when running the winit event loop
    If we can't fix it in SDL we could just compare the joystick's system ids to the ones exposed by win32
    Hopefully GDK input will mitigate this problem
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

        fuck();

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
    game_controllers: HashMap<SdlJoystick, ControllerDevice>,
}

impl ThreadState {
    fn tick(&mut self) {
        //Lock the joystick system so the number of controllers does not change
        update_game_controllers();

        let _lock = lock_joystick_system();
        // let lock_time = Instant::now();

        let num_joysticks = num_joysticks().unwrap();

        // println!("{num_joysticks:?}");

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
                            ControllerDevice::new(device_index, &self.interface, &self.paths),
                        );
                    }
                }
            }

            self.old_joysticks = new_joysticks;
        }

        for controller in self.game_controllers.values_mut() {
            if controller.sdl.get_type() == SDL_GameControllerType::SDL_CONTROLLER_TYPE_PS5 {
                let old_state = controller.state;

                controller.state = DualSense::new(&controller.sdl);

                let gyro = controller
                    .sdl
                    .get_gyro_state()
                    .unwrap_or(Default::default());

                if let Some(idx) = controller.idx {
                    if old_state.shoulder_left != controller.state.shoulder_left {
                        self.interface
                            .send_component_event(suinput_types::event::InputEvent {
                                device: idx,
                                path: self.paths.left_shoulder_click,
                                time: suinput_types::Time(0),
                                data: suinput_types::event::InputComponentEvent::Button(
                                    controller.state.shoulder_left,
                                ),
                            })
                            .unwrap();
                    }
                }

                // let acceld = controller.sdl.get_accel_state().unwrap();

                // println!("{gyro:?}")
            }
        }
    }
}

#[derive(Debug)]
struct ControllerDevice {
    sdl: GameController,
    has_gyro: bool,
    has_accel: bool,

    idx: Option<u64>,
    state: DualSense,
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
            state: DualSense::default(),
            sdl,
            has_gyro,
            has_accel,
            idx,
        }
    }
}

enum ControllerType {
    Xbox360,
    //(Start, Back) -> (Menu, View) and Haptic Triggers
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

#[derive(Debug, Default, Clone, Copy)]
struct DualSense {
    diamond_up: bool,
    diamond_left: bool,
    diamond_down: bool,
    diamond_right: bool,
    dpad_up: bool,
    dpad_left: bool,
    dpad_down: bool,
    dpad_right: bool,
    touchpad_click: bool,
    thumbstick_left_click: bool,
    thumbstick_right_click: bool,
    share: bool,
    options: bool,
    guide: bool,
    mute: bool,
    shoulder_left: bool,
    shoulder_right: bool,

    thumbstick_left: (f32, f32),
    thumbstick_right: (f32, f32),
    trigger_left: f32,
    trigger_right: f32,

    touchpad_1: TouchpadFinger,
    touchpad_2: TouchpadFinger,
}

impl DualSense {
    pub fn new(controller: &GameController) -> Self {
        use sdl2_sys::SDL_GameControllerButton as Button;

        if controller.get_button(Button::SDL_CONTROLLER_BUTTON_Y) {
            println!("a");
        }

        let a = Self {
            diamond_up: controller.get_button(Button::SDL_CONTROLLER_BUTTON_Y),
            diamond_left: controller.get_button(Button::SDL_CONTROLLER_BUTTON_X),
            diamond_down: controller.get_button(Button::SDL_CONTROLLER_BUTTON_A),
            diamond_right: controller.get_button(Button::SDL_CONTROLLER_BUTTON_B),
            dpad_up: controller.get_button(Button::SDL_CONTROLLER_BUTTON_DPAD_UP),
            dpad_left: controller.get_button(Button::SDL_CONTROLLER_BUTTON_DPAD_LEFT),
            dpad_down: controller.get_button(Button::SDL_CONTROLLER_BUTTON_DPAD_DOWN),
            dpad_right: controller.get_button(Button::SDL_CONTROLLER_BUTTON_DPAD_RIGHT),
            touchpad_click: controller.get_button(Button::SDL_CONTROLLER_BUTTON_TOUCHPAD),
            thumbstick_left_click: controller.get_button(Button::SDL_CONTROLLER_BUTTON_LEFTSTICK),
            thumbstick_right_click: controller.get_button(Button::SDL_CONTROLLER_BUTTON_RIGHTSTICK),
            share: controller.get_button(Button::SDL_CONTROLLER_BUTTON_BACK),
            options: controller.get_button(Button::SDL_CONTROLLER_BUTTON_START),
            guide: controller.get_button(Button::SDL_CONTROLLER_BUTTON_GUIDE),
            mute: controller.get_button(Button::SDL_CONTROLLER_BUTTON_MISC1),
            shoulder_left: controller.get_button(Button::SDL_CONTROLLER_BUTTON_LEFTSHOULDER),
            shoulder_right: controller.get_button(Button::SDL_CONTROLLER_BUTTON_RIGHTSHOULDER),

            thumbstick_left: controller.get_thumbstick(true),
            thumbstick_right: controller.get_thumbstick(false),
            trigger_left: controller.get_trigger(true),
            trigger_right: controller.get_trigger(false),

            touchpad_1: controller.get_touchpad_finger(0, 0),
            touchpad_2: controller.get_touchpad_finger(0, 1),
        };

        // println!("{a:?}");

        a
    }
}
