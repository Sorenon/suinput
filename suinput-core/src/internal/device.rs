use std::{sync::Arc, time::Instant};

use nalgebra::Vector3;
use suinput_types::{
    event::{InputComponentEvent, InputEvent},
    SuPath,
};

use super::{
    device_type::DeviceType, input_component::InputComponentData, motion::GamepadMotion,
    paths::InputPath,
};
use crate::internal::types::HashMap;

#[derive(Debug)]
pub struct DeviceState {
    pub ty: Arc<DeviceType>,
    pub input_component_states: HashMap<InputPath, InputComponentData>,
    pub motion: GamepadMotion,
    last_update: Option<Instant>,
}

impl DeviceState {
    pub fn new(ty: Arc<DeviceType>) -> Self {
        Self {
            ty,
            input_component_states: HashMap::new(),
            motion: GamepadMotion::new(),
            last_update: None,
        }
    }

    pub fn handle_batch(
        &mut self,
        time: Instant,
        batch: &HashMap<SuPath, InputComponentEvent>,
    ) -> Result<(), ()> {
        if self.ty.gyro.is_some() && self.ty.accel.is_some() {
            let gyro_path = self.ty.gyro.unwrap();
            let accel_path = self.ty.accel.unwrap();

            let gyro: Vector3<f32> = match batch.get(&gyro_path).unwrap() {
                InputComponentEvent::Gyro(gyro) => (*gyro).into(),
                _ => return Ok(()),
            };

            let accel: Vector3<f32> = match batch.get(&accel_path).unwrap() {
                InputComponentEvent::Accel(accel) => (*accel).into(),
                _ => return Ok(()),
            };

            let delta_time = if let Some(last_time) = &mut self.last_update {
                let delta_time = (time - *last_time).as_secs_f32();
                *last_time = time;
                delta_time
            } else {
                self.last_update = Some(time);
                0.
            };

            self.motion.process_motion(&gyro, &accel, delta_time);
        }

        Ok(())
    }

    pub fn process_input_event(&mut self, event: InputEvent) -> Option<InputEvent> {
        // if let Some(calibration) = self.calibration_data.get(&event.path) {
        //     match (event.data, calibration) {
        //         (InputComponentEvent::Joystick(value), CalibrationData::Joystick { deadzone }) => {
        //             if Vector2::<f32>::from(value).magnitude() <= *deadzone {
        //                 return None;
        //             }
        //         }
        //         (InputComponentEvent::Gyro(value), CalibrationData::Gyro { idle }) => {
        //             event.data =
        //                 InputComponentEvent::Gyro((Vector3::<f32>::from(value) - idle).into());
        //         }
        //         _ => panic!("Mismatched {event:?} and {calibration:?}"),
        //     }
        // }

        // self.input_component_states.insert(
        //     event.path,
        //     InputComponentData {
        //         last_update_time: Instant::now(),
        //         state: match event.data {
        //             InputComponentEvent::Button(pressed) => InputComponentState::Button(pressed),
        //             InputComponentEvent::Cursor(cursor) => InputComponentState::Cursor(cursor),
        //             _ => InputComponentState::NonApplicable,
        //         },
        //     },
        // );

        Some(event)
    }
}
