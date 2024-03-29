use std::sync::Arc;

use dashmap::{mapref::one::Ref, DashMap};
use suinput_types::keyboard::KeyboardPaths;

use super::{
    config::serial_device_type,
    device_type::DeviceType,
    paths::{CommonPaths, DevicePath, PathManager},
};

pub struct DeviceTypes {
    cache: DashMap<DevicePath, Arc<DeviceType>>,
}

impl DeviceTypes {
    pub fn new(
        common_paths: &CommonPaths,
        keyboard_paths: &KeyboardPaths,
        paths: &PathManager,
    ) -> Self {
        Self {
            cache: [DeviceType::create_keyboard(common_paths, keyboard_paths)]
                .into_iter()
                .chain(serial_device_type::deserialize(paths))
                .map(|device_type| (device_type.id, Arc::new(device_type)))
                .collect(),
        }
    }

    pub fn get(&self, path: DevicePath) -> Option<Ref<'_, DevicePath, Arc<DeviceType>>> {
        self.cache.get(&path)
    }
}
