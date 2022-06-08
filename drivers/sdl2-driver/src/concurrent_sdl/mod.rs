use std::ffi::{CStr, CString};

use sdl2_sys::{SDL_GetError, SDL_SetHint, SDL_bool};

pub mod game_controller;
pub mod joystick;

pub type Result<T> = core::result::Result<T, String>;

//https://github.com/libsdl-org/SDL/blob/6b4bd5a75924917e45c70d55596ac2d51267c19e/include/SDL_events.h#L1067

fn get_error() -> String {
    unsafe {
        let err = SDL_GetError();
        CStr::from_ptr(err as *const _).to_str().unwrap().to_owned()
    }
}

pub fn set_hint(name: &str, value: &str) -> bool {
    let name = CString::new(name).unwrap();
    let value = CString::new(value).unwrap();

    unsafe { SDL_SetHint(name.as_ptr(), value.as_ptr()) == SDL_bool::SDL_TRUE }
}
