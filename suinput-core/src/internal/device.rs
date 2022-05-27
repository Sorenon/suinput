use std::{collections::HashMap, time::Instant};

use suinput_types::{
    event::{InputComponentEvent, InputEvent},
    SuPath,
};

use super::input_component::{InputComponentData, InputComponentState};

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
