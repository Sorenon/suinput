use strum::IntoEnumIterator;
use suinput_types::{
    keyboard::{HIDScanCode, KeyboardPaths},
    SuPath,
};

use crate::internal::paths::CommonPaths;
use crate::internal::types::HashMap;

use super::{
    input_component::InputComponentType,
    paths::{DevicePath, InputPath},
};

#[derive(Debug, Clone)]
pub struct DeviceType {
    pub id: DevicePath,
    pub input_components: HashMap<SuPath, InputComponentType>,

    //We assume that a device will only have one of each motion sensor
    //This can be changed in future
    pub gyro: Option<InputPath>,
    pub accel: Option<InputPath>,
}

impl DeviceType {
    pub fn create_keyboard(paths: &CommonPaths, keyboard_paths: &KeyboardPaths) -> Self {
        Self {
            id: paths.keyboard,
            input_components: HIDScanCode::iter()
                .map(|scan_code| (keyboard_paths.get(scan_code), InputComponentType::Button))
                .collect::<HashMap<SuPath, InputComponentType>>(),
            gyro: None,
            accel: None,
        }
    }
}
