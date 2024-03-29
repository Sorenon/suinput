use std::time::Instant;

use nalgebra::{UnitVector3, Vector2, Vector3};
use suinput_types::{
    action::ActionStateEnum,
    event::{InputComponentEvent, InputEvent},
    SuPath,
};

use crate::internal::{
    device::DeviceState,
    input_component::{InputComponentData, InputComponentState, InternalActionState},
    interaction_profile::InteractionProfileState,
    parallel_arena::ParallelArena,
    paths::InteractionProfilePath,
    paths::{InputPath, UserPath},
};

#[derive(Debug, Clone)]
pub struct ProcessedInputBinding {
    pub ty: ProcessedBindingType,
    pub state: InternalActionState,
    pub action: u64,
    pub input_component: (UserPath, InputPath),
}

impl ProcessedInputBinding {
    pub fn save_state(&mut self, action_state: &ActionStateEnum) {
        match action_state {
            ActionStateEnum::Boolean(state) => self.state = InternalActionState::Boolean(*state),
            ActionStateEnum::Value(state) => self.state = InternalActionState::Value(*state),
            ActionStateEnum::Axis1d(state) => self.state = InternalActionState::Axis1d(*state),
            ActionStateEnum::Axis2d(state) => {
                self.state = InternalActionState::Axis2d((*state).into())
            }
            _ => (),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ProcessedBindingType {
    Button2Bool,
    Button2Value,
    Move2d2Delta2d {
        sensitivity: (f64, f64),
    },
    Trigger2Bool,
    Trigger2Value,
    Joystick2Axis2d,
    Gyro2Delta2d {
        last_time: Option<Instant>,
        space: GyroBindingSpace,
        // cut_off_speed: f32,
        // cut_off_recovery: f32,
        // smooth_threshold: f32,
        // smooth_time: f32,
        sensitivity: Sensitivity<f32>,
    },
}

impl ProcessedBindingType {
    /// Returns None if the action state should not be changed / an even should not fire
    pub(crate) fn on_event(
        &mut self,
        user_path: SuPath,
        event: &InputEvent,
        interaction_profile: &InteractionProfileState,
        devices: &ParallelArena<(DeviceState, InteractionProfilePath)>,
    ) -> Option<ActionStateEnum> {
        match (self, event.data) {
            (ProcessedBindingType::Button2Bool, InputComponentEvent::Button(state)) => {
                Some(ActionStateEnum::Boolean(state))
            }
            (ProcessedBindingType::Button2Value, InputComponentEvent::Button(state)) => {
                Some(ActionStateEnum::Value(if state { 1.0 } else { 0.0 }))
            }
            (ProcessedBindingType::Trigger2Bool, InputComponentEvent::Trigger(state)) => {
                Some(ActionStateEnum::Boolean(state > 0.5))
            }
            (ProcessedBindingType::Trigger2Value, InputComponentEvent::Trigger(state)) => {
                Some(ActionStateEnum::Value(state))
            }
            (
                ProcessedBindingType::Move2d2Delta2d { sensitivity },
                InputComponentEvent::Move2D(delta),
            ) => Some(ActionStateEnum::Delta2d(mint::Vector2 {
                x: delta.x * sensitivity.0,
                y: delta.y * sensitivity.1,
            })),
            (ProcessedBindingType::Joystick2Axis2d, InputComponentEvent::Joystick(state)) => {
                Some(ActionStateEnum::Axis2d(state))
            }
            (
                ProcessedBindingType::Gyro2Delta2d {
                    last_time,
                    space,
                    sensitivity,
                },
                InputComponentEvent::Gyro(_),
            ) => {
                if let Some(last_time) = last_time {
                    let now = Instant::now();
                    let delta_time = (now - *last_time).as_secs_f32();
                    *last_time = now;

                    let motion = interaction_profile.get_motion(user_path, devices).unwrap();
                    let angular_velocity = motion.get_calibrated_gyro();

                    let delta = space.transform(angular_velocity, motion.get_gravity());

                    if delta.x.abs() < 0.001 && delta.y.abs() < 0.001 {
                        return None;
                    }

                    let sensitivity = match *sensitivity {
                        Sensitivity::Linear(sensitivity) => sensitivity,
                        Sensitivity::Acceleration {
                            slow_threshold,
                            fast_threshold,
                            slow_scale,
                            fast_scale,
                        } => {
                            let speed = delta.magnitude();
                            let slow_fast_factor =
                                inv_lerp(slow_threshold, fast_threshold, speed).clamp(0., 1.);
                            lerp(slow_scale, fast_scale, slow_fast_factor)
                        }
                    };

                    //TODO investigate turning sign relation
                    Some(ActionStateEnum::Delta2d(mint::Vector2 {
                        x: (-delta.x * delta_time * sensitivity) as f64,
                        y: (delta.y * delta_time * sensitivity) as f64,
                    }))
                } else {
                    *last_time = Some(Instant::now());
                    None
                }
            }
            _ => todo!(),
        }
    }

    pub(crate) fn activate(&mut self, data: InputComponentData) -> Option<ActionStateEnum> {
        match (self, data.state) {
            (ProcessedBindingType::Button2Bool, InputComponentState::Button(state)) => {
                if state {
                    Some(ActionStateEnum::Boolean(true))
                } else {
                    None
                }
            }
            (ProcessedBindingType::Button2Value, InputComponentState::Button(state)) => {
                if state {
                    Some(ActionStateEnum::Value(1.0))
                } else {
                    None
                }
            }
            (ProcessedBindingType::Trigger2Bool, InputComponentState::Trigger(state)) => {
                if state > 0.5 {
                    Some(ActionStateEnum::Boolean(true))
                } else {
                    None
                }
            }
            (ProcessedBindingType::Trigger2Value, InputComponentState::Trigger(state)) => {
                if state != 0. {
                    Some(ActionStateEnum::Value(state))
                } else {
                    None
                }
            }
            (ProcessedBindingType::Joystick2Axis2d, InputComponentState::Joystick(state)) => {
                if state.magnitude_squared() != 0. {
                    Some(ActionStateEnum::Axis2d(state.into()))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub(crate) fn interrupt(
        &mut self,
        binding_state: &InternalActionState,
    ) -> Option<ActionStateEnum> {
        Some(match binding_state {
            InternalActionState::Boolean(state) => {
                if *state {
                    ActionStateEnum::Boolean(false)
                } else {
                    return None;
                }
            }
            InternalActionState::Value(state) => {
                if *state != 0. {
                    ActionStateEnum::Value(0.)
                } else {
                    return None;
                }
            }
            InternalActionState::Axis1d(state) => {
                if *state != 0. {
                    ActionStateEnum::Axis1d(0.)
                } else {
                    return None;
                }
            }
            InternalActionState::Axis2d(state) => {
                if state.x != 0. || state.y != 0. {
                    ActionStateEnum::Axis2d(mint::Vector2 { x: 0., y: 0. })
                } else {
                    return None;
                }
            }
            InternalActionState::NonApplicable => return None,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Sensitivity<T> {
    Linear(T),
    Acceleration {
        slow_threshold: T,
        fast_threshold: T,
        slow_scale: T,
        fast_scale: T,
    },
}

//http://gyrowiki.jibbsmart.com/blog:player-space-gyro-and-alternatives-explained
#[derive(Debug, Clone, Copy)]
pub enum GyroBindingSpace {
    LocalSpace {
        x_axis: Axis,
    },
    LocalCombinedYawRoll,
    PlayerSpace {
        //default 60° (2) for Yaw
        //default 45° (1.41) for Roll
        relax_factor: f32,
        x_axis: Axis,
    },
    WorldSpace {
        x_axis: Axis,
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Axis {
    Yaw,
    Roll,
}

impl GyroBindingSpace {
    //https://github.com/rust-lang/rust/issues/57241
    pub fn calc_relax_factor(degrees: f32) -> f32 {
        1. / f32::sin((90. - degrees).to_radians())
    }

    fn transform(&self, angular_velocity: Vector3<f32>, grav: Vector3<f32>) -> Vector2<f32> {
        match self {
            GyroBindingSpace::LocalSpace { x_axis } => match x_axis {
                Axis::Yaw => Vector2::new(angular_velocity.y, angular_velocity.x),
                Axis::Roll => Vector2::new(angular_velocity.z, angular_velocity.x),
            },
            GyroBindingSpace::LocalCombinedYawRoll => {
                let yaw_axis = Vector2::new(angular_velocity.y, angular_velocity.z);
                let yaw_dir = if yaw_axis.x.abs() > yaw_axis.y.abs() {
                    yaw_axis.x.signum()
                } else {
                    yaw_axis.y.signum()
                };
                Vector2::new(yaw_axis.magnitude() * yaw_dir, angular_velocity.x)
            }
            GyroBindingSpace::PlayerSpace {
                relax_factor,
                x_axis,
            } => {
                let grav = UnitVector3::new_normalize(grav);
                let mut x = 0.;
                match x_axis {
                    Axis::Yaw => {
                        let world_yaw = angular_velocity.y * grav.y + angular_velocity.z * grav.z;

                        x = world_yaw.signum()
                            * f32::min(
                                world_yaw.abs() * relax_factor,
                                Vector2::new(angular_velocity.y, angular_velocity.z).magnitude(),
                            );
                    }
                    Axis::Roll => {
                        // project pitch axis onto gravity plane
                        let grav_dot_pitch_axis = grav.x; // shortcut for (1, 0, 0).Dot(gravNorm)
                        let pitch_vector = Vector3::new(
                            1. - grav.x * grav_dot_pitch_axis,
                            0. - grav.y * grav_dot_pitch_axis,
                            0. - grav.z * grav_dot_pitch_axis,
                        );

                        if pitch_vector.magnitude_squared() != 0. {
                            let roll_vector = pitch_vector.cross(&grav);
                            if roll_vector.magnitude_squared() != 0. {
                                let roll_vector = UnitVector3::new_normalize(roll_vector);
                                let world_roll = angular_velocity.y * roll_vector.y
                                    + angular_velocity.z * roll_vector.z;

                                // some info about the controller's orientation that we'll use to smooth over boundaries
                                let flatness = grav.y.abs(); // 1 when controller is flat
                                let upness = grav.z.abs(); // 1 when controller is upright
                                let side_reduction = f32::clamp(
                                    (f32::max(flatness, upness) - 0.125) / 0.125,
                                    0.,
                                    1.,
                                );

                                x = world_roll.signum()
                                    * side_reduction
                                    * f32::min(
                                        world_roll.abs() * relax_factor,
                                        Vector2::new(angular_velocity.y, angular_velocity.z)
                                            .magnitude(),
                                    );
                            }
                        }
                    }
                };
                Vector2::new(-x, angular_velocity.x)
            }
            GyroBindingSpace::WorldSpace { x_axis: _ } => todo!(),
        }
    }
}

fn lerp(a: f32, b: f32, d: f32) -> f32 {
    a + (b - a) * d
}

fn inv_lerp(a: f32, b: f32, v: f32) -> f32 {
    (v - a) / (b - a)
}
