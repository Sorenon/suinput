use std::sync::Arc;

use crate::internal::types::{hash_map::Entry, HashMap};

use suinput_types::SuPath;

use super::{
    device_type::DeviceType,
    device_types::DeviceTypes,
    input_component::InputComponentType,
    paths::{DevicePath, InputPath, UserPath},
};

#[derive(Debug, Clone)]
pub struct InteractionProfileType {
    pub id: SuPath,
    pub user2device: HashMap<UserPath, Arc<DeviceType>>,
    pub device2user: HashMap<DevicePath, Vec<UserPath>>,
}

impl InteractionProfileType {
    pub fn new_desktop_profile<F: Fn(&str) -> SuPath>(
        device_types: &DeviceTypes,
        get_path: &F,
    ) -> Self {
        let user2device = [
            (
                get_path("/user/desktop/keyboard"),
                get_path("/devices/standard/hid_keyboard"),
            ),
            (
                get_path("/user/desktop/mouse"),
                get_path("/devices/standard/generic_mouse"),
            ),
        ]
        .into_iter()
        .map(|(user_path, device_path)| (user_path, device_types.get(device_path).unwrap().clone()))
        .collect::<HashMap<_, _>>();

        let device2user = user2device.iter().fold(
            HashMap::<SuPath, Vec<SuPath>>::new(),
            |mut device2user, (&user_path, device)| {
                match device2user.entry(device.id) {
                    Entry::Occupied(mut vec) => vec.get_mut().push(user_path),
                    Entry::Vacant(empty) => {
                        empty.insert(vec![user_path]);
                    }
                }
                device2user
            },
        );

        InteractionProfileType {
            id: get_path("/interaction_profiles/standard/desktop"),
            user2device,
            device2user,
        }
    }

    pub fn new_dualsense_profile<F: Fn(&str) -> SuPath>(
        device_types: &DeviceTypes,
        get_path: &F,
    ) -> Self {
        let user2device = [(
            get_path("/user/gamepad"),
            get_path("/devices/sony/dualsense"),
        )]
        .into_iter()
        .map(|(user_path, device_path)| (user_path, device_types.get(device_path).unwrap().clone()))
        .collect::<HashMap<_, _>>();

        let device2user = user2device.iter().fold(
            HashMap::<SuPath, Vec<SuPath>>::new(),
            |mut device2user, (&user_path, device)| {
                match device2user.entry(device.id) {
                    Entry::Occupied(mut vec) => vec.get_mut().push(user_path),
                    Entry::Vacant(empty) => {
                        empty.insert(vec![user_path]);
                    }
                }
                device2user
            },
        );

        InteractionProfileType {
            id: get_path("/interaction_profiles/sony/dualsense"),
            user2device,
            device2user,
        }
    }

    pub fn get_component_type(
        &self,
        user_path: UserPath,
        input_path: InputPath,
    ) -> Option<InputComponentType> {
        if let Some(device_type) = self.user2device.get(&user_path) {
            device_type.input_components.get(&input_path).copied()
        } else {
            None
        }
    }
}
