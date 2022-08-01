use std::{ops::Deref, thread::JoinHandle};

use hooks::Hooks;
use suinput::driver_interface::{RuntimeInterface, SuInputDriver};
use suinput_types::SuPath;
use windows_sys::Win32::Foundation::GetLastError;

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

pub struct Win32HookingWindowDriver {
    runtime_interface: RuntimeInterface,
    hooks: Option<Hooks>,
    running: bool,
}

impl Win32HookingWindowDriver {
    pub fn new(runtime_interface: RuntimeInterface) -> Result<Self> {
        Ok(Self {
            runtime_interface,
            hooks: None,
            running: true,
        })
    }
}

//TODO solve issue where RawInput gets stolen from this driver
//GDK input should solve this anyway
pub struct Win32RawInputGenericDriver {
    runtime_interface: RuntimeInterface,
    raw_input_thread: Option<JoinHandle<()>>,
}

impl Win32RawInputGenericDriver {
    pub fn new(runtime_interface: RuntimeInterface) -> Result<Self> {
        Ok(Self {
            runtime_interface,
            raw_input_thread: None,
        })
    }
}

impl SuInputDriver for Win32HookingWindowDriver {
    fn initialize(&mut self) {
        self.hooks = Some(Hooks::new(&self.runtime_interface));
    }

    fn poll(&self) {
        todo!()
    }

    fn get_component_state(&self, _device: usize, _path: SuPath) {
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

impl Drop for Win32HookingWindowDriver {
    fn drop(&mut self) {
        if self.running {
            println!("WARNING Windows Driver was dropped before being destroyed");
            self.destroy();
        }
    }
}

impl SuInputDriver for Win32RawInputGenericDriver {
    fn initialize(&mut self) {
        let runtime_interface = self.runtime_interface.clone();
        self.raw_input_thread = Some(std::thread::spawn(move || {
            raw_input_driver::run(runtime_interface.0.deref())
        }));
    }

    fn poll(&self) {}

    fn get_component_state(&self, _device: usize, _path: SuPath) {
        todo!()
    }

    fn destroy(&mut self) {}
}
