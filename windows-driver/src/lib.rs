use std::{ops::Deref, thread::JoinHandle};

use suinput::driver_interface::DriverRuntimeInterface;
use windows_sys::Win32::Foundation::{GetLastError, HANDLE};

pub mod hid;
pub mod hooks;
pub mod paths;
pub mod raw_input;
pub mod raw_input_driver;

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

    Winit with Poll set does not receive scroll inertia even if we call GetMessageW

    Always:
    Clicks (Quick tap: Mouse left click, Quick tap then hold: Mouse left click, Quick tap two contacts: Mouse right click)
    Mouse Move (Single contact)
    Zoom (Pinch) Sends scroll + virtual ctrl (MakeCode = 0, v_key = VK_CONTROL)

    Focused Only:
    Pan (Double contact)
*/

pub struct WindowsDesktopDriver {
    pub runtime_interface: DriverRuntimeInterface,
    pub raw_input_thread: JoinHandle<()>,
    pub hooks: (HANDLE, HANDLE),
    running: bool,
}

impl WindowsDesktopDriver {
    pub fn initialize(runtime_interface: DriverRuntimeInterface) -> Result<Self> {
        let hooks = hooks::inject_hooks(&runtime_interface)?;

        let runtime_interface_clone = runtime_interface.clone();
        let raw_input_thread =
            std::thread::spawn(move || raw_input_driver::run(runtime_interface_clone.0.deref()));

        Ok(Self {
            runtime_interface,
            raw_input_thread,
            hooks,
            running: true,
        })
    }

    pub fn destroy(&mut self) {
        hooks::remove_hooks(self.hooks);
        self.running = false;
    }
}

impl Drop for WindowsDesktopDriver {
    fn drop(&mut self) {
        if self.running {
            println!("WARNING Windows Driver was dropped before being destroyed");
            self.destroy();
        }
    }
}
