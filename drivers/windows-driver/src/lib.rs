use std::{ops::Deref, thread::JoinHandle};

use hooks::Hooks;
use suinput_types::{
    driver_interface::{DriverInterface, RuntimeInterface},
    SuPath,
};
use windows_sys::Win32::Foundation::{GetLastError, HANDLE};

//TODO replace with hid_cm
pub mod hid;
pub mod hid_cm;
pub mod hooks;
pub mod paths;
pub mod raw_input;
pub mod raw_input_driver;

#[derive(Debug, thiserror::Error, Clone, Copy)]
pub enum Error {
    #[error("Win32 error 0x{0:0X}")]
    Win32(u32),
    #[error("CfgMgr error 0x{0:0X}")]
    CfgMgr(u32),
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

    On zoom the touchpad hid interface sends a virtual ctrl key

    Always:
    Clicks (Quick tap: Mouse left click, Quick tap then hold: Mouse left click, Quick tap two contacts: Mouse right click)
    Mouse Move (Single contact)
    Zoom (Pinch) Sends scroll + virtual ctrl (MakeCode = 0, v_key = VK_CONTROL)

    Focused Only:
    Pan (Double contact)
*/

pub struct Win32DesktopDriver {
    runtime_interface: RuntimeInterface,
    raw_input_thread: Option<JoinHandle<()>>,
    hooks: Option<Hooks>,
    running: bool,
}

impl Win32DesktopDriver {
    pub fn new(runtime_interface: RuntimeInterface) -> Result<Self> {
        Ok(Self {
            runtime_interface,
            raw_input_thread: None,
            hooks: None,
            running: true,
        })
    }
}

impl DriverInterface for Win32DesktopDriver {
    fn initialize(&mut self) {
        self.hooks = Some(Hooks::new(&self.runtime_interface));
        let runtime_interface_clone = self.runtime_interface.clone();
        self.raw_input_thread = Some(std::thread::spawn(move || {
            raw_input_driver::run(runtime_interface_clone.0.deref())
        }));
    }

    fn poll(&self) {
        todo!()
    }

    fn get_component_state(&self, device: usize, path: SuPath) -> () {
        todo!()
    }

    fn set_windows(&mut self, windows: &[usize]) {
        if let Some(hooks) = &mut self.hooks {
            hooks.set_windows(windows).unwrap();
        }
    }

    fn destroy(&mut self) {
        std::mem::drop(self.hooks.take());
        self.running = false;
    }
}

impl Drop for Win32DesktopDriver {
    fn drop(&mut self) {
        if self.running {
            println!("WARNING Windows Driver was dropped before being destroyed");
            self.destroy();
        }
    }
}
