use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};

use suinput_types::{
    event::{InputComponentEvent, InputEvent},
    SuPath,
};
use thunderdome::{Arena, Index};

use crate::instance::Instance;

use super::{
    binding::working_user::WorkingUser,
    device::DeviceState,
    input_component::{InputComponentData, InputComponentState},
    interaction_profile_type::InteractionProfileType,
    paths::{InputPath, UserPath},
};

#[derive(Debug)]
pub(crate) struct InteractionProfileState {
    profile: InteractionProfileType,
    devices: HashMap<UserPath, HashSet<Index>>,
    input_components: HashMap<(UserPath, InputPath), InputComponentData>,
}

impl InteractionProfileState {
    pub fn new(profile: InteractionProfileType) -> Self {
        Self {
            devices: profile
                .user2device
                .keys()
                .map(|user_path| (*user_path, HashSet::new()))
                .collect(),
            profile,
            input_components: HashMap::new(),
        }
    }

    pub fn device_added(&mut self, device_id: Index, ty: SuPath) {
        if let Some(user_paths) = self.profile.device2user.get(&ty) {
            for user_path in user_paths {
                self.devices.get_mut(user_path).unwrap().insert(device_id);
            }
        }
    }

    pub fn update_component(
        &mut self,
        event: &InputEvent,
        devices: &Arena<(SuPath, DeviceState, Index)>,
        tmp_instance: &Instance,
        tmp_user: &mut WorkingUser,
    ) {
        let event_device_id = Index::from_bits(event.device).unwrap();

        for (user_path, device_ids) in &self.devices {
            if device_ids.contains(&Index::from_bits(event.device).unwrap()) {
                let new_state = match event.data {
                    InputComponentEvent::Button(event_pressed) => {
                        if event_pressed {
                            Some(InputComponentState::Button(true))
                        } else {
                            if device_ids
                                .iter()
                                .filter(|id| **id != event_device_id)
                                .find(|id| {
                                    let (_, device_state, _) = devices.get(**id).unwrap();
                                    match device_state.input_components.get(&event.path) {
                                        Some(InputComponentData {
                                            state: InputComponentState::Button(device_pressed),
                                            ..
                                        }) => *device_pressed,
                                        Some(_) => todo!(
                                        "TODO add interaction profile checks so this can't happen"
                                    ),
                                        None => false,
                                    }
                                })
                                .is_some()
                            {
                                None
                            } else {
                                Some(InputComponentState::Button(false))
                            }
                        }
                    }
                    InputComponentEvent::Move2D(_) => Some(InputComponentState::NonApplicable),
                    InputComponentEvent::Cursor(cursor) => {
                        Some(InputComponentState::Cursor(cursor))
                    }
                };

                if let Some(new_state) = new_state {
                    // for instance in instances {
                    //     process_bindings(
                    //         &self.profile,
                    //         *user_path,
                    //         event,
                    //         &instance,
                    //         tmp_user,
                    //     );
                    // }
                    tmp_user.on_event(&self.profile, *user_path, event, tmp_instance);

                    self.input_components.insert(
                        (*user_path, event.path),
                        InputComponentData {
                            last_update_time: Instant::now(),
                            state: new_state,
                        },
                    );
                }
            }
        }
    }

    pub fn device_removed(&mut self, id: Index, devices: &Arena<(SuPath, DeviceState, Index)>) {}
}