use std::collections::{hash_map::Entry, HashMap};

use suinput_types::SuPath;

#[derive(Debug, Clone)]
pub struct InteractionProfileType {
    pub id: SuPath,
    pub user2device: HashMap<SuPath /* /user/ */, SuPath /* /device/ */>,
    pub device2user: HashMap<SuPath /* /device/ */, Vec<SuPath /* /user/ */>>,
}

impl InteractionProfileType {
    pub fn new_desktop_profile<F: Fn(&str) -> SuPath>(get_path: &F) -> Self {
        let user2device = [
            (
                get_path("/user/desktop/keyboard"),
                get_path("/devices/standard/hid_keyboard"),
            ),
            (
                get_path("/user/desktop/mouse"),
                get_path("/devices/standard/generic_mouse"),
            ),
            (
                get_path("/user/desktop/cursor"),
                get_path("/devices/standard/system_cursor"),
            ),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>();

        let device2user = user2device.iter().fold(
            HashMap::<SuPath, Vec<SuPath>>::new(),
            |mut device2user, (&user_path, &device_path)| {
                match device2user.entry(device_path) {
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

    pub fn new_dualsense_profile<F: Fn(&str) -> SuPath>(get_path: &F) -> Self {
        let user2device = [(
            get_path("/user/gamepad"),
            get_path("/devices/sony/dualsense"),
        )]
        .into_iter()
        .collect::<HashMap<_, _>>();

        let device2user = user2device.iter().fold(
            HashMap::<SuPath, Vec<SuPath>>::new(),
            |mut device2user, (&user_path, &device_path)| {
                match device2user.entry(device_path) {
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
}
