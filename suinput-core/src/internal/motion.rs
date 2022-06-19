//Rewrite of https://github.com/JibbSmart/GamepadMotionHelpers

use nalgebra::{UnitQuaternion, UnitVector3, Vector3};

#[derive(Debug, Default, Clone, Copy)]
struct MotionCalibration {
    pub gyro_offset: Vector3<f32>,
    pub accel_magnitude: f32,
    pub num_samples: u32,
}

#[derive(Debug, Default, Clone, Copy)]
struct SensorMinMaxWindow {
    pub min_gyro: Vector3<f32>,
    pub max_gyro: Vector3<f32>,
    pub mean_gyro: Vector3<f32>,
    pub min_accel: Vector3<f32>,
    pub max_accel: Vector3<f32>,
    pub mean_accel: Vector3<f32>,
    pub start_accel: Vector3<f32>,
    pub num_samples: u32,
    pub time_sampled: f32,
}

#[derive(Debug, Clone, Copy)]
struct AutoCalibration {
    pub min_max_window: SensorMinMaxWindow,
    pub smoothed_angular_velocity_gyro: Vector3<f32>,
    pub smoothed_angular_velocity_accel: Vector3<f32>,
    pub smoothed_prev_accel: Vector3<f32>,
    pub prev_accel: Vector3<f32>,

    min_delta_gyro: Vector3<f32>,
    min_delta_accel: Vector3<f32>,
    recalibrate_threshold: f32,
    sensor_fusion_skipped_time: f32,
    time_steady_sensor_fusion: f32,
    time_steady_stillness: f32,
}

#[derive(Debug, Default, Clone, Copy)]
struct Motion {
    pub orientation: UnitQuaternion<f32>,
    pub accel: Vector3<f32>,
    pub grav: Vector3<f32>,

    pub smooth_accel: Vector3<f32>,
    pub shakiness: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum CalibrationMode {
    Manual,
    Stillness { sensor_fusion: bool },
    SensorFusion,
}

#[derive(Debug, Clone, Copy)]
pub struct GamepadMotionSettings {
    pub min_stillness_samples: u32,
    pub min_stillness_collection_time: f32,
    pub min_stillness_correction_time: f32,
    pub max_stillness_error: f32,
    pub stillness_sample_deterioration_rate: f32,
    pub stillness_error_climb_rate: f32,
    pub stillness_error_drop_on_recalibrate: f32,
    pub stillness_calibration_ease_in_time: f32,
    pub stillness_calibration_half_time: f32,

    pub stillness_gyro_delta: f32,
    pub stillness_accel_delta: f32,

    pub sensor_fusion_calibration_smoothing_strength: f32,
    pub sensor_fusion_angular_acceleration_threshold: f32,
    pub sensor_fusion_calibration_ease_in_time: f32,
    pub sensor_fusion_calibration_half_time: f32,

    pub gravity_correction_shakiness_max_threshold: f32,
    pub gravity_correction_shakiness_min_threshold: f32,

    pub gravity_correction_still_speed: f32,
    pub gravity_correction_shaky_speed: f32,

    pub gravity_correction_gyro_factor: f32,
    pub gravity_correction_gyro_min_threshold: f32,
    pub gravity_correction_gyro_max_threshold: f32,

    pub gravity_correction_minimum_speed: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct GamepadMotion {
    pub settings: GamepadMotionSettings,

    gyro: Vector3<f32>,
    raw_accel: Vector3<f32>,
    motion: Motion,
    motion_calibration: MotionCalibration,
    auto_calibration: AutoCalibration,
    calibration_mode: CalibrationMode,

    manually_calibrating: bool,
}

fn lerp(a: f32, b: f32, d: f32) -> f32 {
    a + (b - a) * d
}

fn inv_lerp(a: f32, b: f32, v: f32) -> f32 {
    (v - a) / (b - a)
}

impl Motion {
    /// The gyro inputs should be calibrated degrees per second but have no other processing. Acceleration is in G units (1 = approx. 9.8m/s^2)
    pub fn update(
        &mut self,
        settings: &GamepadMotionSettings,
        gyro: &Vector3<f32>,
        accel: &Vector3<f32>,
        gravity_length: f32,
        delta_time: f32,
    ) {
        //http://gyrowiki.jibbsmart.com/blog:finding-gravity-with-sensor-fusion
        const SMOOTHED_ACCEL_HALF_LIFE: f32 = 0.25;

        let angle_speed = gyro.magnitude().to_radians();
        let rotation = if angle_speed > 0. {
            let rotation = UnitQuaternion::from_axis_angle(
                &UnitVector3::new_normalize(*gyro),
                angle_speed * delta_time,
            );
            self.orientation *= rotation;
            rotation
        } else {
            UnitQuaternion::default()
        };

        if accel.magnitude() > 0. {
            let accel_dir = UnitVector3::new_normalize(*accel);

            // account for rotation when tracking smoothed acceleration
            self.smooth_accel = rotation.inverse() * self.smooth_accel;

            let smooth_factor = if SMOOTHED_ACCEL_HALF_LIFE <= 0. {
                0.
            } else {
                (-delta_time / SMOOTHED_ACCEL_HALF_LIFE).exp2()
                // == 0.5.pow(delta_time / SMOOTHED_ACCEL_HALF_LIFE)
            };

            self.shakiness *= smooth_factor;
            self.shakiness = self.shakiness.max((accel - self.smooth_accel).magnitude());
            self.smooth_accel = accel.lerp(&self.smooth_accel, smooth_factor);

            // update grav by rotation
            self.grav = rotation.inverse() * self.grav;
            // we want to close the gap between grav and raw acceleration. What's the difference
            let grav_to_accel = accel_dir.scale(-gravity_length) - self.grav;
            let grav_to_accel_dir = UnitVector3::new_normalize(grav_to_accel);
            // adjustment rate
            let mut grav_correction_speed = if settings.gravity_correction_shakiness_min_threshold
                < settings.gravity_correction_shakiness_max_threshold
            {
                lerp(
                    settings.gravity_correction_still_speed,
                    settings.gravity_correction_shaky_speed,
                    inv_lerp(
                        settings.gravity_correction_shakiness_min_threshold,
                        settings.gravity_correction_shakiness_max_threshold,
                        self.shakiness,
                    )
                    .clamp(0., 1.),
                )
            } else {
                if self.shakiness < settings.gravity_correction_gyro_max_threshold {
                    settings.gravity_correction_still_speed
                } else {
                    settings.gravity_correction_shaky_speed
                }
            };
            // we also limit it to be no faster than a given proportion of the gyro rate, or the minimum gravity correction speed
            let gyro_grav_correction_limit = (angle_speed
                * settings.gravity_correction_gyro_factor)
                .max(settings.gravity_correction_minimum_speed);

            if grav_correction_speed > gyro_grav_correction_limit {
                let close_enough_factor = if settings.gravity_correction_gyro_min_threshold
                    < settings.gravity_correction_gyro_max_threshold
                {
                    //However if the difference is big enough we correct it at a faster rate
                    inv_lerp(
                        settings.gravity_correction_gyro_min_threshold,
                        settings.gravity_correction_gyro_max_threshold,
                        grav_to_accel.magnitude(),
                    )
                    .clamp(0., 1.)
                } else {
                    if grav_to_accel.magnitude() < settings.gravity_correction_gyro_max_threshold {
                        0.
                    } else {
                        1.
                    }
                };
                grav_correction_speed = lerp(
                    gyro_grav_correction_limit,
                    grav_correction_speed,
                    close_enough_factor,
                );
            }

            let grav_to_accel_delta = grav_to_accel_dir.scale(grav_correction_speed * delta_time);

            if grav_to_accel_delta.magnitude_squared() < grav_to_accel.magnitude_squared() {
                self.grav += grav_to_accel_delta;
            } else {
                self.grav = accel_dir.scale(-gravity_length);
            }

            //If there is a deviation between orientation and the local gravity direction we rotate to orientation to match
            let world_space_grav_dir =
                self.orientation.inverse() * UnitVector3::new_normalize(self.grav);
            let error_angle = Vector3::new(0., -1., 0.)
                .dot(&world_space_grav_dir)
                .clamp(-1., 1.)
                .acos();
            let flattened =
                UnitVector3::new_normalize(Vector3::new(0., -1., 0.).cross(&world_space_grav_dir));
            let correction_quat = UnitQuaternion::from_axis_angle(&flattened, error_angle);
            self.orientation = correction_quat * self.orientation;

            //Remove gravity from the acceleration to approximate linear acceleration
            self.accel = accel + self.grav;
        } else {
            self.grav = rotation.inverse() * self.grav;
            //If the accelerator is 0 than we are in free fall, thus the linear acceleration is gravity
            self.accel = self.grav;
        }
    }
}

impl SensorMinMaxWindow {
    pub fn reset(&mut self, remainder: f32) {
        self.num_samples = 0;
        self.time_sampled = remainder;
    }

    pub fn add_sample(&mut self, gyro: &Vector3<f32>, accel: &Vector3<f32>, delta_time: f32) {
        if self.num_samples == 0 {
            self.max_gyro = *gyro;
            self.min_gyro = *gyro;
            self.mean_gyro = *gyro;
            self.max_accel = *accel;
            self.min_accel = *accel;
            self.mean_accel = *accel;
            self.start_accel = *accel;
            self.num_samples = 1;
            self.time_sampled += delta_time;
        } else {
            self.max_gyro = self.max_gyro.zip_map(gyro, f32::max);
            self.min_gyro = self.min_gyro.zip_map(gyro, f32::min);
            self.max_accel = self.max_accel.zip_map(accel, f32::max);
            self.min_accel = self.min_accel.zip_map(accel, f32::min);

            self.num_samples += 1;
            self.time_sampled += delta_time;

            // https://en.wikipedia.org/wiki/Algorithms_for_calculating_variance#Welford's_online_algorithm
            let delta = gyro - self.mean_gyro;
            self.mean_gyro += delta * (1. / self.num_samples as f32);
            let delta = accel - self.mean_accel;
            self.mean_accel += delta * (1. / self.num_samples as f32);
        }
    }
}

impl AutoCalibration {
    pub fn new() -> Self {
        Self {
            min_max_window: Default::default(),
            smoothed_angular_velocity_gyro: Default::default(),
            smoothed_angular_velocity_accel: Default::default(),
            smoothed_prev_accel: Default::default(),
            prev_accel: Default::default(),

            min_delta_gyro: Vector3::from_element(10.),
            min_delta_accel: Vector3::from_element(10.),
            recalibrate_threshold: 1.,
            sensor_fusion_skipped_time: 0.,
            time_steady_sensor_fusion: 0.,
            time_steady_stillness: 0.,
        }
    }

    pub fn add_sample_stillness(
        &mut self,
        gyro: &Vector3<f32>,
        accel: &Vector3<f32>,
        delta_time: f32,
        do_sensor_fusion: bool,
        settings: &GamepadMotionSettings,
        calibration_data: &mut MotionCalibration,
    ) -> bool {
        if gyro.x == 0.
            && gyro.y == 0.
            && gyro.z == 0.
            && accel.x == 0.
            && accel.y == 0.
            && accel.z == 0.
        {
            // zeroes are almost certainly not valid inputs
            return false;
        }

        self.min_max_window.add_sample(gyro, accel, delta_time);
        //get deltas
        let gyro_delta = self.min_max_window.max_gyro - self.min_max_window.min_gyro;
        let accel_delta = self.min_max_window.max_accel - self.min_max_window.min_accel;

        let climb_this_tick =
            Vector3::<f32>::from_element(settings.stillness_sample_deterioration_rate * delta_time);

        if settings.stillness_gyro_delta < 0. {
            self.min_delta_gyro += climb_this_tick;
        } else {
            self.min_delta_gyro = Vector3::from_element(settings.stillness_gyro_delta)
        }

        if settings.stillness_accel_delta < 0. {
            self.min_delta_accel += climb_this_tick;
        } else {
            self.min_delta_accel = Vector3::from_element(settings.stillness_accel_delta)
        }

        if self.min_max_window.num_samples < settings.min_stillness_samples
            || self.min_max_window.time_sampled < settings.min_stillness_collection_time
        {
            self.recalibrate_threshold = f32::min(
                self.recalibrate_threshold + settings.stillness_error_climb_rate * delta_time,
                settings.max_stillness_error,
            );
            return false;
        }

        self.min_delta_gyro = self.min_delta_gyro.zip_map(&gyro_delta, f32::min);
        self.min_delta_accel = self.min_delta_accel.zip_map(&accel_delta, f32::min);

        if gyro_delta.x <= self.min_delta_gyro.x * self.recalibrate_threshold
            && gyro_delta.y <= self.min_delta_gyro.y * self.recalibrate_threshold
            && gyro_delta.z <= self.min_delta_gyro.z * self.recalibrate_threshold
            && accel_delta.x <= self.min_delta_accel.x * self.recalibrate_threshold
            && accel_delta.y <= self.min_delta_accel.y * self.recalibrate_threshold
            && accel_delta.z <= self.min_delta_accel.z * self.recalibrate_threshold
        {
            self.time_steady_stillness = f32::min(
                self.time_steady_stillness + delta_time,
                settings.stillness_calibration_ease_in_time,
            );

            let calibration_ease_in = if settings.stillness_calibration_ease_in_time < 0. {
                1.
            } else {
                self.time_steady_stillness / settings.stillness_calibration_ease_in_time
            };

            let calibrated_gyro = &self.min_max_window.mean_gyro;

            let old_gyro_bias =
                calibration_data.gyro_offset / (calibration_data.num_samples as f32).max(1.);
            let stillness_lerp_factor = if settings.stillness_calibration_half_time <= 0. {
                0.
            } else {
                ((calibration_ease_in * -delta_time) / settings.stillness_calibration_half_time)
                    .exp2()
            };
            let mut new_gyro_bias = calibrated_gyro.lerp(&old_gyro_bias, stillness_lerp_factor);

            if do_sensor_fusion {
                let prev_normal = UnitVector3::new_normalize(self.min_max_window.start_accel);
                let this_normal = UnitVector3::new_normalize(*accel);
                let mut angular_vel = this_normal.cross(&prev_normal);
                if angular_vel.magnitude() > 0. {
                    let this_dot_prev = this_normal.dot(&prev_normal).clamp(-1., 1.);
                    let angle_change = this_dot_prev.acos().to_degrees();
                    let angle_per_second = angle_change / self.min_max_window.time_sampled;
                    angular_vel.normalize_mut();
                    angular_vel *= angle_per_second;
                }

                let axis_calibration_strength = this_normal.abs();
                let sensor_fusion_bias =
                    (calibrated_gyro - angular_vel).lerp(&old_gyro_bias, stillness_lerp_factor);

                new_gyro_bias.zip_zip_apply(
                    &axis_calibration_strength,
                    &sensor_fusion_bias,
                    |new_gyro_bias, axis_calibration_strength, sensor_fusion_bias| {
                        if axis_calibration_strength <= 0.7 {
                            *new_gyro_bias = sensor_fusion_bias;
                        }
                    },
                );
            }

            calibration_data.gyro_offset = new_gyro_bias;
            calibration_data.accel_magnitude = self.min_max_window.mean_accel.magnitude();
            calibration_data.num_samples = 1;

            return true;
        }

        if self.time_steady_stillness > 0. {
            self.recalibrate_threshold -= settings.stillness_error_drop_on_recalibrate;
            self.recalibrate_threshold = self.recalibrate_threshold.max(1.);
            self.time_steady_stillness = 0.;
        } else {
            self.recalibrate_threshold = f32::min(
                self.recalibrate_threshold + settings.stillness_error_climb_rate * delta_time,
                settings.max_stillness_error,
            );
        }

        self.min_max_window.reset(0.);
        return false;
    }

    //TODO AddSampleSensorFusion

    fn no_sample_sensor_fusion(&mut self) {
        self.time_steady_sensor_fusion = 0.;
        self.sensor_fusion_skipped_time = 0.;
        self.prev_accel = Vector3::zeros();
        self.smoothed_prev_accel = Vector3::zeros();
        self.smoothed_angular_velocity_gyro = Vector3::zeros();
        self.smoothed_angular_velocity_accel = Vector3::zeros();
    }

    fn no_sample_stillness(&mut self) {
        self.min_max_window.reset(0.);
    }
}

impl GamepadMotionSettings {
    pub fn new() -> Self {
        Self {
            min_stillness_samples: 10,
            min_stillness_collection_time: 0.5,
            min_stillness_correction_time: 2.,
            max_stillness_error: 1.25,
            stillness_sample_deterioration_rate: 0.2,
            stillness_error_climb_rate: 0.1,
            stillness_error_drop_on_recalibrate: 0.1,
            stillness_calibration_ease_in_time: 3.,
            stillness_calibration_half_time: 0.1,

            stillness_gyro_delta: -1.,
            stillness_accel_delta: -1.,

            sensor_fusion_calibration_smoothing_strength: 2.,
            sensor_fusion_angular_acceleration_threshold: 20.,
            sensor_fusion_calibration_ease_in_time: 3.,
            sensor_fusion_calibration_half_time: 0.1,

            gravity_correction_shakiness_max_threshold: 0.4,
            gravity_correction_shakiness_min_threshold: 0.01,

            gravity_correction_still_speed: 1.,
            gravity_correction_shaky_speed: 0.1,

            gravity_correction_gyro_factor: 0.1,
            gravity_correction_gyro_min_threshold: 0.05,
            gravity_correction_gyro_max_threshold: 0.25,

            gravity_correction_minimum_speed: 0.01,
        }
    }
}

impl GamepadMotion {
    pub fn new() -> Self {
        Self {
            settings: GamepadMotionSettings::new(),
            gyro: Vector3::zeros(),
            raw_accel: Vector3::zeros(),
            motion: Motion::default(),
            motion_calibration: MotionCalibration::default(),
            auto_calibration: AutoCalibration::new(),
            calibration_mode: CalibrationMode::Stillness {
                sensor_fusion: true,
            },
            manually_calibrating: false,
        }
    }

    pub fn process_motion(&mut self, gyro: &Vector3<f32>, accel: &Vector3<f32>, delta_time: f32) {
        if gyro.magnitude_squared() == 0. && accel.magnitude_squared() == 0. {
            // all zeroes are almost certainly not valid inputs
            return;
        }

        if self.manually_calibrating {
            self.push_sensor_sample(gyro, accel.magnitude());
            self.auto_calibration.no_sample_sensor_fusion();
            self.auto_calibration.no_sample_stillness();
        } else {
            match self.calibration_mode {
                CalibrationMode::Manual => {
                    self.auto_calibration.no_sample_sensor_fusion();
                    self.auto_calibration.no_sample_stillness();
                }
                CalibrationMode::Stillness { sensor_fusion } => {
                    self.auto_calibration.add_sample_stillness(
                        &gyro,
                        &accel,
                        delta_time,
                        sensor_fusion,
                        &self.settings,
                        &mut self.motion_calibration,
                    );
                    self.auto_calibration.no_sample_sensor_fusion();
                }
                CalibrationMode::SensorFusion => {
                    self.auto_calibration.no_sample_stillness();
                    todo!()
                }
            }
        }

        let (gyro_offset, accel_magnitude) = self.get_calibrated_sensor();

        self.gyro = gyro - gyro_offset;
        self.raw_accel = *accel;

        self.motion.update(
            &self.settings,
            &self.gyro,
            &self.raw_accel,
            accel_magnitude,
            delta_time,
        );
    }

    fn push_sensor_sample(&mut self, gyro: &Vector3<f32>, accel_magnitude: f32) {
        self.motion_calibration.num_samples += 1;
        self.motion_calibration.gyro_offset += gyro;
        self.motion_calibration.accel_magnitude += accel_magnitude;
    }

    fn get_calibrated_sensor(&self) -> (Vector3<f32>, f32) {
        if self.motion_calibration.num_samples == 0 {
            (Vector3::zeros(), 1.)
        } else {
            let inverse_samples = 1. / self.motion_calibration.num_samples as f32;
            (
                self.motion_calibration.gyro_offset * inverse_samples,
                self.motion_calibration.accel_magnitude * inverse_samples,
            )
        }
    }

    pub fn get_calibrated_gyro(&self) -> Vector3<f32> {
        return self.gyro;
    }

    pub fn get_gravity(&self) -> Vector3<f32> {
        return self.motion.grav;
    }

    pub fn get_linear_acceleration(&self) -> Vector3<f32> {
        return self.motion.accel;
    }

    pub fn get_raw_acceleration(&self) -> Vector3<f32> {
        return self.raw_accel;
    }

    pub fn get_orientation(&self) -> UnitQuaternion<f32> {
        return self.motion.orientation;
    }

    pub fn start_continuous_calibration(&mut self) {
        self.manually_calibrating = true;
    }

    pub fn pause_continuous_calibration(&mut self) {
        self.manually_calibrating = true;
    }

    pub fn reset_continuous_calibration(&mut self) {
        self.motion_calibration = MotionCalibration::default();
    }
}
