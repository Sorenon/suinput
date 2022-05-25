use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    sync::Arc,
    time::Instant,
};

use suinput_types::{
    event::{Cursor, InputComponentEvent, InputEvent},
    SuPath,
};
use thunderdome::{Arena, Index};

use crate::{
    binding_layout,
    instance::{self, ActionEvent, ActionEventEnum, Instance},
    runtime::PathManager,
};

#[derive(Debug)]
pub struct InputComponentData {
    pub last_update_time: Instant,
    pub state: InputComponentState,
}

#[derive(Debug)]
pub enum InputComponentState {
    Button(bool),
    Cursor(Cursor),
    NonApplicable,
}

#[derive(Debug, Default)]
pub struct DeviceState {
    pub input_components: HashMap<SuPath /* /inputs/ */, InputComponentData>,
}

impl DeviceState {
    pub fn update_input(&mut self, event: InputEvent) {
        //TODO check against device type
        self.input_components.insert(
            event.path,
            InputComponentData {
                last_update_time: Instant::now(),
                state: match event.data {
                    InputComponentEvent::Button(pressed) => InputComponentState::Button(pressed),
                    InputComponentEvent::Cursor(cursor) => InputComponentState::Cursor(cursor),
                    _ => InputComponentState::NonApplicable,
                },
            },
        );
    }
}

#[derive(Debug, Clone)]
pub struct InteractionProfile {
    pub id: SuPath,
    pub user2device: HashMap<SuPath /* /user/ */, SuPath /* /device/ */>,
    pub device2user: HashMap<SuPath /* /device/ */, Vec<SuPath /* /user/ */>>,
}

impl InteractionProfile {
    pub fn new_keyboard_mouse_cursor_profile(path_manager: &PathManager) -> Self {
        let user2device = [
            (
                path_manager.get_path("/user/desktop/keyboard").unwrap(),
                path_manager
                    .get_path("/device/standard/hid_keyboard")
                    .unwrap(),
            ),
            (
                path_manager.get_path("/user/desktop/mouse").unwrap(),
                path_manager
                    .get_path("/device/standard/generic_mouse")
                    .unwrap(),
            ),
            (
                path_manager.get_path("/user/desktop/cursor").unwrap(),
                path_manager
                    .get_path("/device/standard/system_cursor")
                    .unwrap(),
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

        InteractionProfile {
            id: path_manager
                .get_path("/interaction_profile/standard/keyboard_mouse_cursor")
                .unwrap(),
            user2device,
            device2user,
        }
    }
}

#[derive(Debug)]
pub struct InteractionProfileState {
    profile: InteractionProfile,
    devices: HashMap<SuPath /* /user/ */, HashSet<Index>>,
    input_components: HashMap<(SuPath, SuPath) /* /user/ , /inputs/ */, InputComponentData>,
}

impl InteractionProfileState {
    pub fn new(profile: InteractionProfile) -> Self {
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
        devices: &Arena<(SuPath, DeviceState)>,
        instances: &Vec<Arc<Instance>>,
    ) {
        let event_device_id = Index::from_bits(event.device).unwrap();

        for (user_path, device_ids) in &self.devices {
            if device_ids.contains(&Index::from_bits(event.device).unwrap()) {
                let new_state = match event.data {
                    InputComponentEvent::Button(event_pressed) => {
                        if device_ids
                            .iter()
                            .filter(|id| **id != event_device_id)
                            .find(|id| {
                                let (_, device_state) = devices.get(**id).unwrap();
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
                            Some(InputComponentState::Button(event_pressed))
                        }
                    }
                    InputComponentEvent::Move2D(_) => Some(InputComponentState::NonApplicable),
                    InputComponentEvent::Cursor(cursor) => {
                        Some(InputComponentState::Cursor(cursor))
                    }
                };

                if let Some(new_state) = new_state {
                    //TODO split path
                    for instance in instances {
                        process_bindings(
                            &self.profile,
                            &self.input_components,
                            *user_path,
                            event,
                            &instance,
                        );
                    }

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

    pub fn device_removed(&mut self, id: Index, devices: &Arena<(SuPath, DeviceState)>) {}
}

fn process_bindings(
    interaction_profile: &InteractionProfile,
    interaction_profile_state: &HashMap<(SuPath, SuPath), InputComponentData>,
    user_path: SuPath,
    event: &InputEvent,
    instance: &Instance,
) {
    let player = instance.player.read();

    if let Some(binding_layout) = player.active_binding_layout.get(&interaction_profile.id) {
        if let Some(component_bindings) = binding_layout.bindings.get(&user_path) {
            if let Some(actions) = component_bindings.get(&event.path) {
                for listener in instance.listeners.read().iter() {
                    for action in actions {
                        listener.handle_event(ActionEvent {
                            action_handle: *action,
                            time: Instant::now(),
                            data: match event.data {
                                InputComponentEvent::Button(state) => ActionEventEnum::Boolean {
                                    state,
                                    changed: interaction_profile_state
                                        .get(&(user_path, event.path))
                                        .map_or(false, |old_state| match old_state {
                                            InputComponentData {
                                                state: InputComponentState::Button(state),
                                                ..
                                            } => *state,
                                            _ => panic!("TODO store old state in action"),
                                        })
                                        != state,
                                },
                                InputComponentEvent::Move2D(state) => ActionEventEnum::Delta2D { state: state.value },
                                InputComponentEvent::Cursor(state) => ActionEventEnum::Cursor { state: state.normalized_screen_coords },
                            },
                        })
                    }
                }
            }
        }
    }
}
