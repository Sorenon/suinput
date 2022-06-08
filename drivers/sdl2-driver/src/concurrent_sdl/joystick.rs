use sdl2_sys::{
    SDL_JoystickGetDeviceInstanceID, SDL_LockJoysticks, SDL_NumJoysticks, SDL_UnlockJoysticks,
};

use super::get_error;
use super::Result;
pub type SdlDeviceIndex = u32;
pub type SdlJoystick = u32;

pub fn num_joysticks() -> Result<SdlDeviceIndex> {
    unsafe { SDL_NumJoysticks().try_into().map_err(|_| get_error()) }
}

pub fn get_instance_id(device_index: SdlDeviceIndex) -> Result<SdlJoystick> {
    unsafe {
        SDL_JoystickGetDeviceInstanceID(device_index as i32)
            .try_into()
            .map_err(|_| get_error())
    }
}

#[must_use = "if unused the Mutex will immediately unlock"]
pub struct JoystickLockGuard;

pub fn lock_joystick_system() -> JoystickLockGuard {
    unsafe {
        SDL_LockJoysticks();
    }
    JoystickLockGuard
}

impl Drop for JoystickLockGuard {
    fn drop(&mut self) {
        unsafe {
            SDL_UnlockJoysticks();
        }
    }
}
