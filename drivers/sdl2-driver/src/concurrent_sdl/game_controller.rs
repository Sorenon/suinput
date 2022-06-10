use std::ffi::CStr;
use std::ptr::NonNull;

use nalgebra::{Vector2, Vector3};
use sdl2_sys::{
    SDL_GameControllerAxis, SDL_GameControllerButton, SDL_GameControllerGetAxis,
    SDL_GameControllerGetButton, SDL_GameControllerGetNumTouchpadFingers,
    SDL_GameControllerGetProduct, SDL_GameControllerGetSensorData,
    SDL_GameControllerGetTouchpadFinger, SDL_GameControllerGetType, SDL_GameControllerGetVendor,
    SDL_GameControllerHasSensor, SDL_GameControllerName, SDL_GameControllerSetSensorEnabled,
    SDL_GameControllerType, SDL_SensorType,
};
use sdl2_sys::{
    SDL_GameControllerOpen, SDL_GameControllerUpdate, SDL_IsGameController, SDL_bool,
    _SDL_GameController,
};

use super::get_error;
use super::joystick::SdlDeviceIndex;
use super::Result;

pub fn update_game_controllers() {
    unsafe {
        //This function is thread safe (SDL_LockJoysticks)
        SDL_GameControllerUpdate();
    }
}

//It is ok to call this twice for the same index, it just returns the same controller again
pub fn open_game_controller(joystick_index: SdlDeviceIndex) -> Result<GameController> {
    unsafe {
        //SDL_GameControllerOpen is thread safe (SDL_LockJoysticks)
        Ok(GameController(
            NonNull::new(SDL_GameControllerOpen(joystick_index as i32)).ok_or_else(get_error)?,
        ))
    }
}

pub fn is_game_controller(joystick_index: SdlDeviceIndex) -> bool {
    unsafe { SDL_IsGameController(joystick_index as i32) == SDL_bool::SDL_TRUE }
}

#[derive(Debug)]
pub struct GameController(NonNull<_SDL_GameController>);

impl GameController {
    pub fn has_sensor(&self, ty: SDL_SensorType) -> bool {
        unsafe { SDL_GameControllerHasSensor(self.0.as_ptr(), ty) == SDL_bool::SDL_TRUE }
    }

    pub fn set_sensor_state(&self, ty: SDL_SensorType, enabled: bool) -> Result<()> {
        if unsafe {
            SDL_GameControllerSetSensorEnabled(
                self.0.as_ptr(),
                ty,
                if enabled {
                    SDL_bool::SDL_TRUE
                } else {
                    SDL_bool::SDL_FALSE
                },
            )
        } == -1
        {
            Err(get_error())
        } else {
            Ok(())
        }
    }

    pub fn get_vendor(&self) -> u16 {
        unsafe { SDL_GameControllerGetVendor(self.0.as_ptr()) }
    }

    pub fn get_pid(&self) -> u16 {
        unsafe { SDL_GameControllerGetProduct(self.0.as_ptr()) }
    }

    pub fn get_type(&self) -> SDL_GameControllerType {
        unsafe { SDL_GameControllerGetType(self.0.as_ptr()) }
    }

    pub fn get_name(&self) -> String {
        unsafe {
            let name = SDL_GameControllerName(self.0.as_ptr());

            CStr::from_ptr(name as *const _)
                .to_str()
                .unwrap()
                .to_owned()
        }
    }

    pub fn get_axis_state(&self, axis: SDL_GameControllerAxis) -> i16 {
        unsafe { SDL_GameControllerGetAxis(self.0.as_ptr(), axis) }
    }

    pub fn get_button(&self, button: SDL_GameControllerButton) -> bool {
        unsafe { SDL_GameControllerGetButton(self.0.as_ptr(), button) == 1 }
    }

    pub fn get_num_touchpad_fingers(&self, touchpad: u32) -> u32 {
        unsafe {
            SDL_GameControllerGetNumTouchpadFingers(self.0.as_ptr(), touchpad as i32)
                .try_into()
                .unwrap()
        }
    }

    pub fn get_touchpad_finger(&self, touchpad: u32, finger: u32) -> TouchpadFinger {
        let mut state = 0;
        let mut out = TouchpadFinger::default();
        unsafe {
            if SDL_GameControllerGetTouchpadFinger(
                self.0.as_ptr(),
                touchpad as i32,
                finger as i32,
                &mut state,
                &mut out.x,
                &mut out.y,
                &mut out.pressure,
            ) == 0
            {
                out.down = state == 1;
            }
        }
        out
    }

    pub fn get_trigger(&self, left: bool) -> f32 {
        self.get_axis_state(if left {
            SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_TRIGGERLEFT
        } else {
            SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_TRIGGERRIGHT
        }) as f32
            / 32767.
    }

    pub fn get_thumbstick(&self, left: bool) -> Vector2<f32> {
        Vector2::new(
            self.get_axis_state(if left {
                SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_LEFTX
            } else {
                SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_RIGHTX
            }) as f32
                / 32767.,
            self.get_axis_state(if left {
                SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_LEFTY
            } else {
                SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_RIGHTX
            }) as f32
                / 32767.,
        )
    }

    pub fn get_sensor_state(&self, sensor: SDL_SensorType, out: &mut [f32]) -> Result<()> {
        if unsafe {
            SDL_GameControllerGetSensorData(
                self.0.as_ptr(),
                sensor,
                out.as_mut_ptr(),
                out.len() as i32,
            )
        } == -1
        {
            Err(get_error())
        } else {
            Ok(())
        }
    }

    pub fn get_gyro_state(&self) -> Result<Vector3<f32>> {
        let mut out = Vector3::default();
        self.get_sensor_state(SDL_SensorType::SDL_SENSOR_GYRO, out.as_mut_slice())?;
        out.x = out.x.to_degrees();
        out.y = out.y.to_degrees();
        out.z = out.z.to_degrees();
        Ok(out)
    }

    pub fn get_accel_state(&self) -> Result<Vector3<f32>> {
        let mut out = Vector3::default();
        self.get_sensor_state(SDL_SensorType::SDL_SENSOR_ACCEL, out.as_mut_slice())?;
        Ok(out)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct TouchpadFinger {
    pub down: bool,
    pub x: f32,
    pub y: f32,
    pub pressure: f32,
}

impl Drop for GameController {
    fn drop(&mut self) {
        println!("dropped");
        // unsafe { SDL_GameControllerClose(self.0.as_ptr()) }
    }
}
