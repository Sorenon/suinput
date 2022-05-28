use dashmap::{mapref::one::Ref, DashMap};
use suinput_types::keyboard::KeyboardPaths;

use super::{
    device_type::DeviceType,
    paths::{CommonPaths, DevicePath},
};

pub struct DeviceTypes {
    cache: DashMap<DevicePath, DeviceType>,
}

impl DeviceTypes {
    pub fn new(common_paths: &CommonPaths, keyboard_paths: &KeyboardPaths) -> Self {
        Self {
            cache: [
                DeviceType::create_mouse(&common_paths),
                DeviceType::create_keyboard(&common_paths, &keyboard_paths),
                DeviceType::create_cursor(&common_paths),
            ]
            .into_iter()
            .map(|device_type| (device_type.id, device_type))
            .collect(),
        }
    }

    pub fn get(&self, path: DevicePath) -> Option<Ref<'_, DevicePath, DeviceType>> {
        self.cache.get(&path)
    }
}
