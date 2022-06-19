use std::{
    collections::{HashMap, HashSet},
    time::Instant,
    vec::IntoIter,
};

use suinput_types::{
    event::{InputComponentEvent, InputEvent},
    SuPath,
};
use thunderdome::{Arena, Index};

use super::{
    device::DeviceState,
    input_component::{InputComponentData, InputComponentState},
    input_events::{Axis2d, InputEventSources, InputEventType, Value},
    interaction_profile_type::InteractionProfileType,
    motion::GamepadMotion,
    paths::{InputPath, UserPath},
};

#[derive(Debug)]
pub(crate) struct InteractionProfileState {
    pub ty: InteractionProfileType,
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
            ty: profile,
            input_components: HashMap::new(),
        }
    }

    pub fn device_added(&mut self, device_id: Index, ty: SuPath) {
        if let Some(user_paths) = self.ty.device2user.get(&ty) {
            for user_path in user_paths {
                self.devices.get_mut(user_path).unwrap().insert(device_id);
            }
        }
    }

    pub fn update_component<
        F: FnMut(&InteractionProfileState, UserPath, &InputEvent, &Arena<(DeviceState, Index)>),
    >(
        &mut self,
        event: &InputEvent,
        devices: &Arena<(DeviceState, Index)>,
        mut process_bindings: F,
    ) {
        let event_device_id = Index::from_bits(event.device).unwrap();

        for (user_path, device_ids) in &self.devices {
            if device_ids.contains(&event_device_id) {
                let helper = IESHelper {
                    profile: &self,
                    devices,
                };

                let new_state = match event.data {
                    InputComponentEvent::Button(event_pressed) => helper
                        .aggregate::<bool>((*user_path, event.path), event_pressed, event_device_id)
                        .map(|(state, _)| InputComponentState::Button(state)),
                    InputComponentEvent::Move2D(_) => Some(InputComponentState::NonApplicable),
                    InputComponentEvent::Cursor(cursor) => {
                        Some(InputComponentState::Cursor(cursor))
                    }
                    InputComponentEvent::Trigger(state) => helper
                        .aggregate::<Value>((*user_path, event.path), state, event_device_id)
                        .map(|state| InputComponentState::Trigger(state)),
                    InputComponentEvent::Joystick(state) => helper
                        .aggregate::<Axis2d>(
                            (*user_path, event.path),
                            state.into(),
                            event_device_id,
                        )
                        .map(|state| InputComponentState::Joystick(state)),
                    InputComponentEvent::Gyro(_) =>
                    //TODO only have one active gyro for component per interaction profile
                    {
                        Some(InputComponentState::NonApplicable)
                    }
                    InputComponentEvent::Accel(_) => None,
                };

                if let Some(new_state) = new_state {
                    process_bindings(&self, *user_path, event, devices);

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

    pub fn get_motion(
        &self,
        user_path: UserPath,
        devices: &Arena<(DeviceState, Index)>,
    ) -> Result<GamepadMotion, ()> {
        //TODO store this state in Self
        //TODO have better method of selected active motion device

        let device_ids = self.devices.get(&user_path).ok_or(())?;
        Ok(if let Some(device) = device_ids.iter().next() {
            devices.get(*device).ok_or(())?.0.motion
        } else {
            GamepadMotion::new()
        })
    }

    pub fn device_removed(&mut self, id: Index, devices: &Arena<(DeviceState, Index)>) {
        todo!()
    }
}

struct IESHelper<'a> {
    profile: &'a InteractionProfileState,
    devices: &'a thunderdome::Arena<(DeviceState, Index)>,
}

impl<'a> InputEventSources for IESHelper<'a> {
    type Index = (UserPath, InputPath);
    type SourceIndex = Index;

    type Sources = IntoIter<Self::SourceIndex>;

    fn get_state<I: InputEventType>(&self, idx: Self::Index) -> Option<I::Value> {
        self.profile
            .input_components
            .get(&idx)
            .map(|data| I::from_ics(&data.state))
    }

    fn get_source_state<I: InputEventType>(
        &self,
        (_, input_path): Self::Index,
        source_idx: Self::SourceIndex,
    ) -> Option<I::Value> {
        let (device_state, _) = self.devices.get(source_idx).unwrap();
        device_state
            .input_component_states
            .get(&input_path)
            .map(|data| I::from_ics(&data.state))
    }

    fn get_sources<I: InputEventType>(&self, (user_path, _): Self::Index) -> Self::Sources {
        self.profile
            .devices
            .get(&user_path)
            .unwrap()
            .iter()
            .map(|idx| *idx)
            .collect::<Vec<_>>()
            .into_iter()
    }
}
