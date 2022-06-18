use std::{collections::HashMap, sync::Arc, time::Instant};

use nalgebra::Vector3;
use suinput_types::{
    event::{InputComponentEvent, InputEvent},
    SuPath,
};

use super::{
    device_type::DeviceType,
    input_component::{InputComponentData, InputComponentType},
    paths::InputPath,
};

#[derive(Debug)]
pub struct DeviceState {
    pub ty: Arc<DeviceType>,
    pub input_component_states: HashMap<InputPath, InputComponentData>,
    motion: TmpMotion,
}

#[derive(Debug, Default)]
struct TmpMotion {
    ticks: u64,
    cal_data: Vector3<f32>,
}

impl DeviceState {
    pub fn new(ty: Arc<DeviceType>) -> Self {
        Self {
            ty,
            input_component_states: HashMap::new(),
            motion: TmpMotion::default(),
        }
    }

    pub fn handle_batch(
        &mut self,
        time: Instant,
        batch: HashMap<SuPath, InputComponentEvent>,
    ) -> Result<(), ()> {
        if self.ty.gyro.is_some() || self.ty.accel.is_some() {
            let test = if let Some(path) = self.ty.gyro {
                let reading = if let Some(component) = batch.get(&path) {
                    Some(component.get_gyro()?)
                } else {
                    None
                };
                reading
            } else {
                None
            };

            let accel_update =
                if let Some(component) = self.ty.accel.map(|path| batch.get(&path)).flatten() {
                    Some(component.get_accel()?)
                } else {
                    None
                };
        }

        Ok(())

        // for (path, event) in &batch {
        //     if let Some(component_ty) = self.ty.input_components.get(path) {
        //         match (component_ty, event) {
        //             (
        //                 InputComponentType::Gyro(calibrated),
        //                 InputComponentEvent::Gyro(new_gyro_reading),
        //             ) => {

        //             }
        //             (InputComponentType::Accel, InputComponentEvent::Accel(_)) => todo!(),
        //             _ => (),
        //         }
        //     }
        // }
    }
    //process_input_event -> applies calibration

    pub fn process_input_event(&mut self, mut event: InputEvent) -> Option<InputEvent> {
        match event.data {
            InputComponentEvent::Gyro(raw_angular_velocity) => {
                let raw_angular_velocity: Vector3<f32> = raw_angular_velocity.into();

                if self.motion.ticks < 400 {
                    self.motion.ticks += 1;

                    if self.motion.ticks == 1 {
                        self.motion.cal_data = raw_angular_velocity;
                    } else {
                        self.motion.cal_data =
                            self.motion.cal_data.lerp(&raw_angular_velocity, 0.1);
                    }

                    if self.motion.ticks == 400 {
                        println!("CALIBRATION DONE! -> {:?}", self.motion.cal_data)
                    } else {
                        return None;
                    }
                }

                event.data =
                    InputComponentEvent::Gyro((raw_angular_velocity - self.motion.cal_data).into());
            }
            _ => (),
        }

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

#[derive(Debug)]
enum Test {
    Accel(AccelState),
    Gyro(GyroState),
}

#[derive(Debug)]
struct AccelState {
    smoothed_linear_velocity: Vector3<f32>,
}

#[derive(Debug)]
struct GyroState {
    smoothed_raw_angular_velocity: Vector3<f32>,
}

// #[derive(Debug)]
// pub enum CalibrationData {
//     Joystick { deadzone: f32 },
//     Gyro { idle: Vector3<f32> },
// }
