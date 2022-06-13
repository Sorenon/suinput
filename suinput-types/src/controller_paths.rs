use crate::SuPath;

pub struct GameControllerPaths {
    pub device_xbox360: SuPath,
    pub device_xbox_one: SuPath,
    pub device_xbox_series_xs: SuPath,
    pub device_xbox_elite: SuPath,

    pub device_dual_shock_3: SuPath,
    pub device_dual_shock_4: SuPath,
    pub device_dual_sense: SuPath,

    pub device_switch_pro: SuPath,
    pub device_left_joycon: SuPath,
    pub device_right_joycon: SuPath,
    pub device_gamecube: SuPath,

    pub device_luna: SuPath,
    pub device_stadia: SuPath,

    pub device_generic: SuPath,

    pub interaction_profile_dualsense: SuPath,

    //TODO joycons + gamecube
    pub diamond_up: SuPath,
    pub diamond_left: SuPath,
    pub diamond_down: SuPath,
    pub diamond_right: SuPath,
    pub guide: SuPath,
    pub left_stick_click: SuPath,
    pub right_stick_click: SuPath,
    pub left_shoulder: SuPath,
    pub right_shoulder: SuPath,
    pub dpad_up: SuPath,
    pub dpad_left: SuPath,
    pub dpad_right: SuPath,
    pub dpad_down: SuPath,
    // misc1: SuPath, //TODO
    pub paddle1: SuPath,
    pub paddle2: SuPath,
    pub paddle3: SuPath,
    pub paddle4: SuPath,
    pub touchpad_click: SuPath,
    pub left_trigger_click: SuPath,
    pub right_trigger_click: SuPath,

    pub back: SuPath,
    pub create: SuPath,

    pub start: SuPath,
    pub options: SuPath,

    pub mute: SuPath,

    pub left_trigger: SuPath,
    pub right_trigger: SuPath,

    pub left_joystick: SuPath,
    pub right_joystick: SuPath,

    pub gyro: SuPath,
    pub accel: SuPath,

    pub touchpad: SuPath,
}

impl GameControllerPaths {
    pub fn new<F: Fn(&str) -> SuPath>(get_path: F) -> Self {
        Self {
            device_xbox360: get_path("/devices/microsoft/xbox_360"),
            device_xbox_one: get_path("/devices/microsoft/xbox_one"),
            device_xbox_series_xs: get_path("/devices/microsoft/xbox_one_series_xs"),
            device_xbox_elite: get_path("/devices/microsoft/xbox_elite"),
            device_dual_shock_3: get_path("/devices/sony/dualshock3"),
            device_dual_shock_4: get_path("/devices/sony/dualshock4"),
            device_dual_sense: get_path("/devices/sony/dualsense"),
            device_switch_pro: get_path("/devices/nintendo/switch_pro"),
            device_left_joycon: get_path("/devices/nintendo/joycon_left"),
            device_right_joycon: get_path("/devices/nintendo/joycon_right"),
            device_gamecube: get_path("/devices/nintendo/gamecube"),
            device_luna: get_path("/devices/amazon/luna"),
            device_stadia: get_path("/devices/google/stadia"),
            device_generic: get_path("/devices/sdl/generic_game_controller"),

            interaction_profile_dualsense: get_path("/interaction_profiles/sony/dualsense"),

            diamond_up: get_path("/input/diamond_up/click"),
            diamond_left: get_path("/input/diamond_left/click"),
            diamond_down: get_path("/input/diamond_down/click"),
            diamond_right: get_path("/input/diamond_right/click"),
            // back: todo!(),
            // guide: todo!(),
            // start: todo!(),
            left_stick_click: get_path("/input/joystick_left/click"),
            right_stick_click: get_path("/input/joystick_right/click"),
            left_shoulder: get_path("/input/shoulder_left/click"),
            right_shoulder: get_path("/input/shoulder_right/click"),
            dpad_up: get_path("/input/dpad_up/click"),
            dpad_left: get_path("/input/dpad_left/click"),
            dpad_right: get_path("/input/dpad_right/click"),
            dpad_down: get_path("/input/dpad_down/click"),
            // misc1: todo!(),
            paddle1: get_path("/input/paddle_top_right/click"),
            paddle2: get_path("/input/paddle_top_left/click"),
            paddle3: get_path("/input/paddle_bottom_right/click"),
            paddle4: get_path("/input/paddle_bottom_left/click"),
            touchpad_click: get_path("/input/touchpad/click"),

            left_trigger_click: get_path("/input/trigger_left/click"),
            right_trigger_click: get_path("/input/trigger_right/click"),
            left_trigger: get_path("/input/trigger_left/value"),
            right_trigger: get_path("/input/trigger_right/value"),
            left_joystick: get_path("/input/joystick_left/position"),
            right_joystick: get_path("/input/joystick_right/position"),
            gyro: get_path("/input/motion/gyro"),
            accel: get_path("/input/motion/accel"),
            touchpad: get_path("/input/touchpad/points"),
            guide: get_path("/input/guide/click"),
            back: get_path("/input/back/click"),
            create: get_path("/input/create/click"),
            start: get_path("/input/start/click"),
            options: get_path("/input/options/click"),
            mute: get_path("/input/mute/click"),
        }
    }
}
