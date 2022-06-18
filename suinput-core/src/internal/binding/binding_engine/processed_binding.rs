use std::time::Instant;

use nalgebra::ComplexField;
use suinput_types::{
    action::ActionStateEnum,
    event::{InputComponentEvent, InputEvent},
};

#[derive(Debug, Clone)]
pub enum ProcessedBinding {
    Button2Bool,
    Button2Value,
    Move2d2Delta2d {
        sensitivity: (f64, f64),
    },
    Cursor2Cursor,
    Trigger2Bool,
    Trigger2Value,
    Joystick2Axis2d,
    Gyro2Delta2d {
        sensitivity: (f64, f64),
        last_time: Option<Instant>,
    },
}

impl ProcessedBinding {
    /// Returns None if the action state should not be changed / an even should not fire
    pub(crate) fn on_event(&mut self, event: &InputEvent) -> Option<ActionStateEnum> {
        match (self, event.data) {
            (ProcessedBinding::Button2Bool, InputComponentEvent::Button(state)) => {
                Some(ActionStateEnum::Boolean(state))
            }
            (ProcessedBinding::Button2Value, InputComponentEvent::Button(state)) => {
                Some(ActionStateEnum::Value(if state { 1.0 } else { 0.0 }))
            }
            (ProcessedBinding::Trigger2Bool, InputComponentEvent::Trigger(state)) => {
                Some(ActionStateEnum::Boolean(state > 0.5))
            }
            (ProcessedBinding::Trigger2Value, InputComponentEvent::Trigger(state)) => {
                Some(ActionStateEnum::Value(state))
            }
            (
                ProcessedBinding::Move2d2Delta2d { sensitivity },
                InputComponentEvent::Move2D(delta),
            ) => Some(ActionStateEnum::Delta2D((
                delta.x * sensitivity.0,
                delta.y * sensitivity.1,
            ))),
            (ProcessedBinding::Cursor2Cursor, InputComponentEvent::Cursor(cursor)) => {
                Some(ActionStateEnum::Cursor(cursor.normalized_screen_coords))
            }
            (ProcessedBinding::Joystick2Axis2d, InputComponentEvent::Joystick(state)) => {
                Some(ActionStateEnum::Axis2d(state))
            }
            (
                ProcessedBinding::Gyro2Delta2d {
                    sensitivity,
                    last_time,
                },
                InputComponentEvent::Gyro(angular_velocity),
            ) => {
                if let Some(last_time) = last_time {
                    let now = Instant::now();
                    let delta_time = (now - *last_time).as_secs_f32();
                    *last_time = now;

                    let delta = (
                        (angular_velocity.x * delta_time) as f64 * sensitivity.0,
                        (angular_velocity.y * delta_time) as f64 * sensitivity.1,
                    );

                    if delta.0.abs() < 0.001 || delta.1.abs() < 0.001 {
                        return None;
                    }

                    Some(ActionStateEnum::Delta2D(delta))
                } else {
                    *last_time = Some(Instant::now());
                    None
                }
            }
            _ => todo!(),
        }
    }
}
