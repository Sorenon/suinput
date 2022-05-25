use std::collections::HashMap;

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use suinput_types::{
    driver_interface::RuntimeInterface,
    event::{Cursor, InputComponentEvent, InputEvent},
    SuPath, Time,
};
use windows_sys::Win32::{
    Foundation::HANDLE,
    UI::WindowsAndMessaging::{
        CallNextHookEx, GetPhysicalCursorPos, GetWindowThreadProcessId, SetWindowsHookExW,
        UnhookWindowsHookEx, CWPSTRUCT, MSG, PM_REMOVE, WH_CALLWNDPROC, WH_GETMESSAGE, WM_CHAR,
        WM_SETCURSOR,
    },
};

use crate::Error;

static HOOK_STATE: Lazy<RwLock<Option<HookState>>> = Lazy::new(|| RwLock::new(None));

struct HookState {
    interface: RuntimeInterface,
    cursor_move: SuPath,
    cursor_device: u64,
}

pub struct Hooks {
    hooks: HashMap<u32, (HANDLE, HANDLE)>,
}

impl Hooks {
    pub fn new(interface: &RuntimeInterface) -> Self {
        assert!(HOOK_STATE.read().is_none());

        *HOOK_STATE.write() = Some(HookState {
            interface: interface.clone(),
            cursor_move: interface.get_path("/input/cursor/point").unwrap(),
            cursor_device: interface.register_new_device(
                interface
                    .get_path("/device/standard/system_cursor")
                    .unwrap(),
            ).unwrap(),
        });

        Self {
            hooks: HashMap::new(),
        }
    }

    pub fn set_windows(&mut self, windows: &[usize]) -> Result<(), Error> {
        let mut old_hooks = HashMap::new();
        std::mem::swap(&mut old_hooks, &mut self.hooks);

        for &window in windows {
            let thread_id =
                unsafe { GetWindowThreadProcessId(window as isize, std::ptr::null_mut()) };
            if self.hooks.contains_key(&thread_id) {
                continue;
            }
            if let Some(hooks) = old_hooks.remove(&thread_id) {
                self.hooks.insert(thread_id, hooks);
            } else {
                unsafe {
                    let hook_handle =
                        SetWindowsHookExW(WH_CALLWNDPROC, Some(call_wnd_proc), 0, thread_id);
                    if hook_handle == 0 {
                        return Err(Error::win32());
                    }
                    let hook_handle2 =
                        SetWindowsHookExW(WH_GETMESSAGE, Some(call_get_message), 0, thread_id);
                    if hook_handle2 == 0 {
                        return Err(Error::win32());
                    }
                    self.hooks.insert(thread_id, (hook_handle, hook_handle2));
                }
            }
        }

        for (_, hooks) in old_hooks {
            unsafe {
                UnhookWindowsHookEx(hooks.0);
                UnhookWindowsHookEx(hooks.1);
            }
        }

        Ok(())
    }
}

impl Drop for Hooks {
    fn drop(&mut self) {
        *HOOK_STATE.write() = None;
        self.set_windows(&[]).unwrap();
    }
}

unsafe extern "system" fn call_wnd_proc(a: i32, b: usize, cwp_ptr: isize) -> isize {
    let cwp = &*(cwp_ptr as usize as *const CWPSTRUCT);

    if cwp.message == WM_SETCURSOR {
        let mut point = std::mem::zeroed();
        if GetPhysicalCursorPos(&mut point) == 0 {
            panic!("{}", Error::win32());
        }

        let hook_state_guard = HOOK_STATE.try_read().unwrap();
        let hook_state = hook_state_guard.as_ref().unwrap();

        hook_state.interface.send_component_event(InputEvent {
            device: hook_state.cursor_device,
            path: hook_state.cursor_move,
            time: Time(0),
            data: InputComponentEvent::Cursor(Cursor {
                normalized_screen_coords: (point.x as f64, point.y as f64),
            }),
        });
    }

    CallNextHookEx(0, a, b, cwp_ptr)
}

unsafe extern "system" fn call_get_message(code: i32, remove: usize, msg_ptr: isize) -> isize {
    //TODO figure out if it would be better to use this for cursor movement instead
    //TODO experiment with hooking WM_CHAR
    if remove as u32 == PM_REMOVE {
        let msg = &*(msg_ptr as usize as *const MSG);

        if msg.message == WM_CHAR {
            // let mut point = std::mem::zeroed();
            // if GetPhysicalCursorPos(&mut point) == 0 {
            //     panic!("{}", Error::win32());
            // }
            // println!("2.{}, {}", point.x, point.y);
        }
    }

    CallNextHookEx(0, code, remove, msg_ptr)
}
