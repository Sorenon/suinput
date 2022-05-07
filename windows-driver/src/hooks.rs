use windows_sys::Win32::{
    System::Threading::GetCurrentThreadId,
    UI::WindowsAndMessaging::{
        CallNextHookEx, GetPhysicalCursorPos, SetWindowsHookExW, CWPRETSTRUCT, CWPSTRUCT, MSG,
        PM_REMOVE, WH_CALLWNDPROC, WH_GETMESSAGE, WM_CHAR, WM_SETCURSOR, WM_UNICHAR, WM_MOUSEMOVE,
    },
};

use crate::Error;

/**
 * Inject a hook into the application's window to get cursor move and text events
 */
pub fn inject_hook() -> Result<isize, Error> {
    unsafe {
        let thread = GetCurrentThreadId();
        let hook_handle = SetWindowsHookExW(WH_CALLWNDPROC, Some(call_wnd_proc), 0, thread);
        let hook_handle2 = SetWindowsHookExW(WH_GETMESSAGE, Some(call_get_message), 0, thread);
        if hook_handle == 0 {
            Err(Error::win32())
        } else {
            Ok(hook_handle)
        }
    }
}

unsafe extern "system" fn call_wnd_proc(a: i32, b: usize, cwp_ptr: isize) -> isize {
    let cwp = &*(cwp_ptr as usize as *const CWPSTRUCT);

    if cwp.message == WM_SETCURSOR {
        let mut point = std::mem::zeroed();
        if GetPhysicalCursorPos(&mut point) == 0 {
            panic!("{}", Error::win32());
        }
        // println!("1.{}, {}", point.x, point.y);
    }

    CallNextHookEx(0, a, b, cwp_ptr)
}

unsafe extern "system" fn call_get_message(code: i32, remove: usize, msg_ptr: isize) -> isize {
    if remove as u32 == PM_REMOVE {
        let msg = &*(msg_ptr as usize as *const MSG);

        if msg.message == WM_CHAR {
            let mut point = std::mem::zeroed();
            if GetPhysicalCursorPos(&mut point) == 0 {
                panic!("{}", Error::win32());
            }
            // println!("2.{}, {}", point.x, point.y);
        }
    }

    CallNextHookEx(0, code, remove, msg_ptr)
}
