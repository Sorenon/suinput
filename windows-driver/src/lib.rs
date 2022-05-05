use windows_sys::Win32::Foundation::GetLastError;

pub mod hooks;
pub mod raw_input;
pub mod hid;

#[derive(Debug, thiserror::Error, Clone, Copy)]
pub enum Error {
    #[error("Win32 error 0x{0:0X}")]
    Win32(u32),
}

impl Error {
    pub fn win32() -> Self {
        unsafe { Self::Win32(GetLastError()) }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/*
    Precision TouchPad findings:
    https://docs.microsoft.com/en-us/windows-hardware/design/component-guidelines/precision-touchpad-implementation-guide
    All inputs are provided through the OS mouse + keyboard (id = 0)
    Some inputs are only provided if the process is focused
    If the user is making a gesture (eg pinch or zoom) clicks will not be sent
    If GetMessageW is filtered by a hWnd, scroll inertia will not work

    Always:
    Clicks (Quick tap: Mouse left click, Quick tap then hold: Mouse left click, Quick tap two contacts: Mouse right click)
    Mouse Move (Single contact)
    Zoom (Pinch) Sends scroll + virtual ctrl (MakeCode = 0, v_key = VK_CONTROL)

    Focused Only:
    Pan (Double contact)
*/
