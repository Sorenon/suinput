use std::collections::HashMap;

use strum::IntoEnumIterator;
use suinput_types::{
    controller_paths::GameControllerPaths,
    keyboard::{HIDScanCode, KeyboardPaths},
    SuPath,
};

use crate::internal::paths::CommonPaths;

use super::{input_component::InputComponentType, paths::DevicePath};

#[derive(Debug, Clone)]
pub struct DeviceType {
    pub id: DevicePath,
    pub input_components: HashMap<SuPath, InputComponentType>,
}

impl DeviceType {
    pub fn create_keyboard(paths: &CommonPaths, keyboard_paths: &KeyboardPaths) -> Self {
        Self {
            id: paths.keyboard,
            input_components: HIDScanCode::iter()
                .map(|scan_code| (keyboard_paths.get(scan_code), InputComponentType::Button))
                .collect::<HashMap<SuPath, InputComponentType>>(),
        }
    }

    pub fn create_mouse(paths: &CommonPaths) -> Self {
        Self {
            id: paths.mouse,
            input_components: [
                (paths.mouse_left_click, InputComponentType::Button),
                (paths.mouse_middle_click, InputComponentType::Button),
                (paths.mouse_right_click, InputComponentType::Button),
                (paths.mouse_button4_click, InputComponentType::Button),
                (paths.mouse_button5_click, InputComponentType::Button),
                (paths.mouse_move, InputComponentType::Move2D),
                (paths.mouse_scroll, InputComponentType::Move2D),
            ]
            .into_iter()
            .collect(),
        }
    }

    pub fn create_cursor(paths: &CommonPaths) -> Self {
        Self {
            id: paths.system_cursor,
            input_components: [(paths.cursor_point, InputComponentType::Cursor)]
                .into_iter()
                .collect(),
        }
    }

    pub fn create_dualsense(paths: &GameControllerPaths) -> Self {
        Self {
            id: paths.device_dual_sense,
            input_components: [(paths.left_shoulder_click, InputComponentType::Button)]
                .into_iter()
                .collect(),
        }
    }
}
