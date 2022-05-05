use std::{ffi::OsString, mem::size_of, os::windows::prelude::OsStringExt};

use suinput::{
    event::HIDScanCode, event::MouseButton, event::MouseEvent, event2::DriverManager, Vec2D,
};
use windows_sys::Win32::{
    Devices::HumanInterfaceDevice::MOUSE_MOVE_ABSOLUTE,
    Foundation::{GetLastError, ERROR_INSUFFICIENT_BUFFER, HANDLE, HWND},
    UI::{
        Input::{
            GetRawInputData, GetRawInputDeviceInfoW, GetRawInputDeviceList,
            KeyboardAndMouse::VK_CONTROL, RegisterRawInputDevices, HRAWINPUT, RAWINPUT,
            RAWINPUTDEVICE, RAWINPUTDEVICELIST, RAWINPUTHEADER, RIDEV_DEVNOTIFY, RIDEV_INPUTSINK,
            RIDI_DEVICEINFO, RIDI_DEVICENAME, RID_DEVICE_INFO, RID_INPUT, RIM_TYPEHID,
            RIM_TYPEKEYBOARD, RIM_TYPEMOUSE,
        },
        WindowsAndMessaging::*,
    },
};

use crate::{Error, Result};

/**
 * Returns a list of all raw input devices
 * RAWINPUTDEVICELIST.dwType is mostly useless
 */
pub fn get_ri_devices() -> Result<Vec<RAWINPUTDEVICELIST>> {
    let mut num = 0;
    unsafe {
        loop {
            let mut raw_input_device_lists = vec![std::mem::zeroed(); num as usize];

            if GetRawInputDeviceList(
                raw_input_device_lists.as_mut_ptr(),
                &mut num,
                size_of::<RAWINPUTDEVICELIST>() as u32,
            ) as i32
                == -1
            {
                if GetLastError() == ERROR_INSUFFICIENT_BUFFER {
                    continue;
                }
                return Err(Error::win32());
            }

            return Ok(raw_input_device_lists);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RIDeviceInfo {
    Mouse {
        id_properties: u32,
        number_of_buttons: u32,
        sample_rate: u32,
        has_horizontal_wheel: bool,
    },
    Keyboard {
        keyboard_type: u32,
        sub_type: u32,
        scan_code_mode: u32,
        number_of_function_keys: u32,
        number_of_indicators: u32,
        number_of_keys: u32,
    },
    GenericHID {
        vendor_id: u32,
        product_id: u32,
        version_number: u32,
        usage_page: u16,
        usage: u16,
    },
}

pub fn get_ri_device_info(raw_input_device: HANDLE) -> Result<RIDeviceInfo> {
    unsafe {
        let mut device_info: RID_DEVICE_INFO = std::mem::zeroed();

        device_info.cbSize = size_of::<RID_DEVICE_INFO>() as u32;

        if GetRawInputDeviceInfoW(
            raw_input_device,
            RIDI_DEVICEINFO,
            &mut device_info as *mut _ as _,
            &mut (size_of::<RID_DEVICE_INFO>() as u32),
        ) as i32
            <= 0
        {
            return Err(Error::win32());
        }

        return Ok(match device_info.dwType {
            RIM_TYPEMOUSE => {
                let info = &device_info.Anonymous.mouse;
                RIDeviceInfo::Mouse {
                    id_properties: info.dwId,
                    number_of_buttons: info.dwNumberOfButtons,
                    sample_rate: info.dwSampleRate,
                    has_horizontal_wheel: info.fHasHorizontalWheel != 0,
                }
            }
            RIM_TYPEKEYBOARD => {
                let info = &device_info.Anonymous.keyboard;
                RIDeviceInfo::Keyboard {
                    keyboard_type: info.dwType,
                    sub_type: info.dwSubType,
                    scan_code_mode: info.dwKeyboardMode,
                    number_of_function_keys: info.dwNumberOfFunctionKeys,
                    number_of_indicators: info.dwNumberOfIndicators,
                    number_of_keys: info.dwNumberOfKeysTotal,
                }
            }
            RIM_TYPEHID => {
                let info = &device_info.Anonymous.hid;
                RIDeviceInfo::GenericHID {
                    vendor_id: info.dwVendorId,
                    product_id: info.dwProductId,
                    version_number: info.dwVersionNumber,
                    usage_page: info.usUsagePage,
                    usage: info.usUsage,
                }
            }
            _ => unreachable!(),
        });
    }
}

pub fn register_raw_input_classes(hwnd: HWND) -> Result<()> {
    unsafe {
        let mut device_classes = [
            RAWINPUTDEVICE {
                usUsagePage: 0x01,
                usUsage: 0x02,
                dwFlags: RIDEV_DEVNOTIFY | RIDEV_INPUTSINK,
                hwndTarget: hwnd,
            },
            RAWINPUTDEVICE {
                usUsagePage: 0x01,
                usUsage: 0x06,
                dwFlags: RIDEV_DEVNOTIFY | RIDEV_INPUTSINK,
                hwndTarget: hwnd,
            },
        ];

        if RegisterRawInputDevices(
            device_classes.as_mut_ptr(),
            device_classes.len() as _,
            size_of::<RAWINPUTDEVICE>() as u32,
        ) == 0
        {
            return Err(Error::win32());
        }
        return Ok(());
    }
}

pub fn get_raw_input_data(raw_input_handle: HRAWINPUT) -> Result<RAWINPUT> {
    unsafe {
        //RAWINPUT is statically sized unless it's from a HID device
        let mut raw_input: RAWINPUT = std::mem::zeroed();
        if GetRawInputData(
            raw_input_handle,
            RID_INPUT,
            &mut raw_input as *mut _ as _,
            &mut (size_of::<RAWINPUT>() as u32),
            size_of::<RAWINPUTHEADER>() as u32,
        ) as i32
            == -1
        {
            return Err(Error::win32());
        }
        return Ok(raw_input);
    }
}

pub fn get_rid_device_interface_name(raw_input_device: HANDLE) -> Result<OsString> {
    unsafe {
        let mut size = 0;

        if GetRawInputDeviceInfoW(
            raw_input_device,
            RIDI_DEVICENAME,
            std::ptr::null_mut(),
            &mut size,
        ) != 0
        {
            return Err(Error::win32());
        }

        let mut buffer = Vec::<u16>::with_capacity(size as usize);

        if (GetRawInputDeviceInfoW(
            raw_input_device,
            RIDI_DEVICENAME,
            buffer.as_mut_ptr() as _,
            &mut size,
        ) as i32)
            < 0
        {
            Err(Error::win32())
        } else {
            buffer.set_len(size as usize);

            Ok(OsString::from_wide(&buffer))
        }
    }
}

pub mod window {
    use std::{
        collections::{HashMap, HashSet},
        ffi::OsStr,
        os::windows::prelude::OsStrExt,
    };

    use suinput::{
        event::{HIDScanCode, KeyboardEvent},
        event2::{DeviceEvent, DriverManager, DummyDriverManager},
    };
    use windows_sys::Win32::{
        Foundation::{HANDLE, HWND},
        System::{LibraryLoader::GetModuleHandleW, SystemInformation::GetTickCount},
        UI::{
            Input::{RIM_TYPEHID, RIM_TYPEKEYBOARD, RIM_TYPEMOUSE},
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageTime, GetMessageW,
                RegisterClassW, GIDC_ARRIVAL, RI_KEY_BREAK, RI_KEY_E0, RI_KEY_E1, WM_INPUT,
                WM_INPUT_DEVICE_CHANGE, WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE,
                WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT, WS_OVERLAPPED,
            },
        },
    };

    use crate::Result;
    use crate::{raw_input::get_ri_device_info, Error};

    use super::{get_raw_input_data, RIDeviceInfo};

    pub fn create_background_window() -> Result<HWND> {
        unsafe {
            let class_name: Vec<_> = OsStr::new("Background RI Window")
                .encode_wide()
                .chain(Some(0).into_iter())
                .collect();

            let title = OsStr::new("Background RI Window")
                .encode_wide()
                .chain(Some(0).into_iter())
                .collect::<Vec<_>>();

            if RegisterClassW(&WNDCLASSW {
                lpfnWndProc: Some(DefWindowProcW),
                hInstance: GetModuleHandleW(std::ptr::null()),
                lpszClassName: class_name.as_ptr(),
                ..std::mem::zeroed()
            }) == 0
            {
                return Err(Error::win32());
            }

            let hwnd = CreateWindowExW(
                WS_EX_NOACTIVATE | WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_TOOLWINDOW,
                class_name.as_ptr(),
                title.as_ptr(),
                WS_OVERLAPPED,
                0,
                0,
                0,
                0,
                0,
                0,
                GetModuleHandleW(std::ptr::null()),
                std::ptr::null(),
            );
            if hwnd == 0 {
                return Err(Error::win32());
            }

            return Ok(hwnd);
        }
    }

    pub fn run() {
        let window = create_background_window().unwrap();
        super::register_raw_input_classes(window).unwrap();
        unsafe {
            // let mut containers = HashMap::<OsString, Vec<HANDLE>>::new();
            let mut ri_devices = HashMap::<HANDLE, RIDeviceInfo>::new();
            let mut keyboard_states = HashMap::<HANDLE, HashSet<HIDScanCode>>::new();

            let driver_manager = DummyDriverManager;

            driver_manager.send_device_event(DeviceEvent::DeviceActivated {
                id: 0,
                ty: "standard_mouse/mouse".to_string(),
            });

            driver_manager.send_device_event(DeviceEvent::DeviceActivated {
                id: 0,
                ty: "standard_keyboard/keyboard".to_string(),
            });

            let mut msg = std::mem::zeroed();
            while GetMessageW(&mut msg, 0, 0, 0) > 0 {
                //TODO brush up on my overflow maths and figure this out
                //https://devblogs.microsoft.com/oldnewthing/20140122-00/?p=2013
                let _time = GetTickCount();
                let _message_time = GetMessageTime() as u32;

                match msg.message {
                    WM_INPUT_DEVICE_CHANGE => {
                        let raw_input_device = msg.lParam as HANDLE;
                        if msg.wParam == GIDC_ARRIVAL as usize {
                            let rid_device_info = get_ri_device_info(raw_input_device).unwrap();

                            let device_type = match rid_device_info {
                                RIDeviceInfo::Mouse { .. } => "standard_mouse/mouse",
                                RIDeviceInfo::Keyboard { .. } => "standard_keyboard/keyboard",
                                RIDeviceInfo::GenericHID { .. } => todo!(),
                            };

                            ri_devices.insert(raw_input_device, rid_device_info);

                            driver_manager.send_device_event(DeviceEvent::DeviceActivated {
                                id: raw_input_device as _,
                                ty: device_type.to_string(),
                            });
                        } else {
                            ri_devices.remove(&raw_input_device).unwrap();
                            driver_manager.send_device_event(DeviceEvent::DeviceDeactivated {
                                id: raw_input_device as _,
                            })
                        }
                    }
                    WM_INPUT => {
                        let raw_input_data = get_raw_input_data(msg.lParam as _).unwrap();

                        let raw_input_device = raw_input_data.header.hDevice;

                        //TODO check if this is true
                        //If the application initialized raw input before we do we may miss WM_INPUT_DEVICE_CHANGE
                        if raw_input_device != 0 && !ri_devices.contains_key(&raw_input_device) {
                            let rid_device_info = get_ri_device_info(raw_input_device).unwrap();

                            let device_type = match rid_device_info {
                                RIDeviceInfo::Mouse { .. } => "standard_mouse/mouse",
                                RIDeviceInfo::Keyboard { .. } => "standard_keyboard/keyboard",
                                RIDeviceInfo::GenericHID { .. } => todo!(),
                            };

                            ri_devices.insert(raw_input_device, rid_device_info);

                            driver_manager.send_device_event(DeviceEvent::DeviceActivated {
                                id: raw_input_device as _,
                                ty: device_type.to_string(),
                            });
                        }

                        match raw_input_data.header.dwType {
                            RIM_TYPEMOUSE => super::process_mouse(raw_input_data, &driver_manager),
                            RIM_TYPEKEYBOARD => {
                                let keyboard = raw_input_data.data.keyboard;
                                let flags = keyboard.Flags as u32;
                                let mode = if flags & RI_KEY_BREAK == 0 {
                                    "Down"
                                } else {
                                    "Up"
                                };

                                let message = {
                                    use windows_sys::Win32::UI::WindowsAndMessaging::*;
                                    match keyboard.Message {
                                        WM_ACTIVATE => "WM_ACTIVATE",
                                        WM_APPCOMMAND => "WM_APPCOMMAND",
                                        WM_CHAR => "WM_CHAR",
                                        WM_DEADCHAR => "WM_CHAR",
                                        WM_HOTKEY => "WM_CHAR",
                                        WM_KEYDOWN => "WM_CHAR",
                                        WM_KEYUP => "WM_CHAR",
                                        WM_KILLFOCUS => "WM_CHAR",
                                        WM_SETFOCUS => "WM_CHAR",
                                        WM_SYSDEADCHAR => "WM_CHAR",
                                        WM_SYSKEYDOWN => "WM_CHAR",
                                        WM_SYSKEYUP => "WM_CHAR",
                                        WM_UNICHAR => "WM_CHAR",
                                        _ => "Unknown",
                                    }
                                };

                                let e0 = flags & RI_KEY_E0 != 0;
                                let e1 = flags & RI_KEY_E1 != 0;

                                let hid_scan_code = super::raw_input_to_hid_scancode(
                                    keyboard.MakeCode,
                                    e0,
                                    keyboard.VKey,
                                    e1,
                                );

                                println!(
                                    "Keyboard({}): {:?} {} {:X} {:X} {} e0:{} e1:{}",
                                    raw_input_device as usize,
                                    hid_scan_code,
                                    message,
                                    keyboard.MakeCode,
                                    keyboard.VKey,
                                    mode,
                                    e0,
                                    e1
                                );

                                //https://github.com/rust-lang/rust/issues/87335 pls
                                let hid_scan_code =
                                    if let Some(hid_scan_code) = hid_scan_code.ok().flatten() {
                                        hid_scan_code
                                    } else {
                                        return;
                                    };

                                let keyboard_state = if let Some(keyboard_state) =
                                    keyboard_states.get_mut(&raw_input_device)
                                {
                                    keyboard_state
                                } else {
                                    keyboard_states.insert(raw_input_device, HashSet::new());
                                    keyboard_states.get_mut(&raw_input_device).unwrap()
                                };

                                if flags & RI_KEY_BREAK == 0 {
                                    //Key pressed
                                    if !keyboard_state.contains(&hid_scan_code) {
                                        keyboard_state.insert(hid_scan_code);
                                        let event = KeyboardEvent::Press(hid_scan_code);
                                    }
                                } else {
                                    //Key released
                                    let event = KeyboardEvent::Release(hid_scan_code);
                                };
                            }
                            RIM_TYPEHID => {
                                unimplemented!();
                            }
                            _ => unimplemented!(),
                        }
                    }
                    _ => (),
                }

                DispatchMessageW(&msg);
            }
        }
    }
}

fn process_mouse(raw_input_data: RAWINPUT, driver_manager: &dyn DriverManager) {
    let mouse = unsafe { raw_input_data.data.mouse };

    if mouse.usFlags & (MOUSE_MOVE_ABSOLUTE as u16) == 0 {
        if mouse.lLastX != 0 || mouse.lLastY != 0 {
            let event = MouseEvent::Move(Vec2D {
                x: mouse.lLastX as f32,
                y: mouse.lLastY as f32,
            });
            println!("Mouse({}) {:?}", raw_input_data.header.hDevice, event);
        }
    } else {
        unimplemented!(
            "Mouse({}) Absolute {} {}",
            raw_input_data.header.hDevice,
            mouse.lLastX,
            mouse.lLastY
        );
    }

    let data = unsafe { &mouse.Anonymous.Anonymous };

    if data.usButtonFlags != 0 {
        let flags = data.usButtonFlags as u32;
        if flags & RI_MOUSE_BUTTON_1_DOWN != 0 {
            let event = MouseEvent::Press(MouseButton::Left);
            println!("Mouse({}) {:?}", raw_input_data.header.hDevice, event);
        } else if flags & RI_MOUSE_BUTTON_1_UP != 0 {
            let event = MouseEvent::Release(MouseButton::Left);
            println!("Mouse({}) {:?}", raw_input_data.header.hDevice, event);
        }
        if flags & RI_MOUSE_BUTTON_2_DOWN != 0 {
            let event = MouseEvent::Press(MouseButton::Right);
            println!("Mouse({}) {:?}", raw_input_data.header.hDevice, event);
        } else if flags & RI_MOUSE_BUTTON_2_UP != 0 {
            let event = MouseEvent::Release(MouseButton::Right);
            println!("Mouse({}) {:?}", raw_input_data.header.hDevice, event);
        }
        if flags & RI_MOUSE_BUTTON_3_DOWN != 0 {
            let event = MouseEvent::Press(MouseButton::Middle);
            println!("Mouse({}) {:?}", raw_input_data.header.hDevice, event);
        } else if flags & RI_MOUSE_BUTTON_3_UP != 0 {
            let event = MouseEvent::Release(MouseButton::Middle);
            println!("Mouse({}) {:?}", raw_input_data.header.hDevice, event);
        }
        if flags & RI_MOUSE_BUTTON_4_DOWN != 0 {
            let event = MouseEvent::Press(MouseButton::Button4);
            println!("Mouse({}) {:?}", raw_input_data.header.hDevice, event);
        } else if flags & RI_MOUSE_BUTTON_4_UP != 0 {
            let event = MouseEvent::Release(MouseButton::Button4);
            println!("Mouse({}) {:?}", raw_input_data.header.hDevice, event);
        }
        if flags & RI_MOUSE_BUTTON_5_DOWN != 0 {
            let event = MouseEvent::Press(MouseButton::Button5);
            println!("Mouse({}) {:?}", raw_input_data.header.hDevice, event);
        } else if flags & RI_MOUSE_BUTTON_5_UP != 0 {
            let event = MouseEvent::Release(MouseButton::Button5);
            println!("Mouse({}) {:?}", raw_input_data.header.hDevice, event);
        }
        if flags & RI_MOUSE_WHEEL != 0 {
            let event = MouseEvent::Scroll(Vec2D {
                x: 0.,
                y: (data.usButtonData as i16) as f32 / WHEEL_DELTA as f32,
            });
            println!("Mouse({}) {:?}", raw_input_data.header.hDevice, event);
        }
        if flags & RI_MOUSE_HWHEEL != 0 {
            let event = MouseEvent::Scroll(Vec2D {
                x: (data.usButtonData as i16) as f32 / WHEEL_DELTA as f32,
                y: 0.,
            });
            println!("Mouse({}) {:?}", raw_input_data.header.hDevice, event);
        }
    }
}

fn raw_input_to_hid_scancode(
    make_code: u16,
    e0: bool,
    v_key: u16,
    e1: bool,
) -> Result<Option<HIDScanCode>> {
    //https://www.win.tue.nl/~aeb/linux/kbd/scancodes-1.html
    //https://download.microsoft.com/download/1/6/1/161ba512-40e2-4cc9-843a-923143f3456c/translate.pdf
    //https://www.scs.stanford.edu/10wi-cs140/pintos/specs/kbd/scancodes-9.html
    //https://superuser.com/questions/550679/where-to-find-windows-keyboard-scancode-registry-information

    if make_code == 0 {
        //TODO sort these out
        println!("Received fake key {v_key}");
        return Ok(None);
    }

    let scancode = if e0 {
        match make_code {
            0x1C => HIDScanCode::KeypadEnter,  //
            0x1D => HIDScanCode::RightControl, //
            0x2A => return Ok(None),           //fake LShift
            0x35 => HIDScanCode::KeypadDivide, //
            0x36 => return Ok(None),           //fake RShift
            0x37 => HIDScanCode::PrintScreen,  //
            0x38 => HIDScanCode::RightAlt,     //
            0x46 => HIDScanCode::Pause,
            0x47 => HIDScanCode::Home,     //
            0x48 => HIDScanCode::Up,       //
            0x49 => HIDScanCode::PageUp,   //
            0x4B => HIDScanCode::Left,     //
            0x4D => HIDScanCode::Right,    //
            0x4F => HIDScanCode::End,      //
            0x50 => HIDScanCode::Down,     //
            0x51 => HIDScanCode::PageDown, //
            0x52 => HIDScanCode::Insert,   //
            0x53 => HIDScanCode::Delete,   //
            0x5B => HIDScanCode::LeftGui,
            0x5C => HIDScanCode::RightGui,
            0x5D => HIDScanCode::App,
            // 0x5E => keyboard power
            // 0x5F => sleep,
            // 0x63 => wake,
            _ => panic!("{make_code}"),
        }
    } else {
        match make_code {
            0x01 => HIDScanCode::Escape,         //
            0x02 => HIDScanCode::Key1,           //
            0x03 => HIDScanCode::Key2,           //
            0x04 => HIDScanCode::Key3,           //
            0x05 => HIDScanCode::Key4,           //
            0x06 => HIDScanCode::Key5,           //
            0x07 => HIDScanCode::Key6,           //
            0x08 => HIDScanCode::Key7,           //
            0x09 => HIDScanCode::Key8,           //
            0x0A => HIDScanCode::Key9,           //
            0x0B => HIDScanCode::Key0,           //
            0x0C => HIDScanCode::Minus,          //
            0x0D => HIDScanCode::Equals,         //
            0x0E => HIDScanCode::Backspace,      //
            0x0F => HIDScanCode::Tab,            //
            0x10 => HIDScanCode::Q,              //
            0x11 => HIDScanCode::W,              //
            0x12 => HIDScanCode::E,              //
            0x13 => HIDScanCode::R,              //
            0x14 => HIDScanCode::T,              //
            0x15 => HIDScanCode::Y,              //
            0x16 => HIDScanCode::U,              //
            0x17 => HIDScanCode::I,              //
            0x18 => HIDScanCode::O,              //
            0x19 => HIDScanCode::P,              //
            0x1A => HIDScanCode::LeftBracket,    //
            0x1B => HIDScanCode::RightBracket,   //
            0x1C => HIDScanCode::Enter,          //
            0x1D => HIDScanCode::LeftControl,    //
            0x1E => HIDScanCode::A,              //
            0x1F => HIDScanCode::S,              //
            0x20 => HIDScanCode::D,              //
            0x21 => HIDScanCode::F,              //
            0x22 => HIDScanCode::G,              //
            0x23 => HIDScanCode::H,              //
            0x24 => HIDScanCode::J,              //
            0x25 => HIDScanCode::K,              //
            0x26 => HIDScanCode::L,              //
            0x27 => HIDScanCode::Semicolon,      //
            0x28 => HIDScanCode::Apostrophe,     //
            0x29 => HIDScanCode::Grave,          //
            0x2A => HIDScanCode::LeftShift,      //
            0x2B => HIDScanCode::Backslash,      //
            0x2C => HIDScanCode::Z,              //
            0x2D => HIDScanCode::X,              //
            0x2E => HIDScanCode::C,              //
            0x2F => HIDScanCode::V,              //
            0x30 => HIDScanCode::B,              //
            0x31 => HIDScanCode::N,              //
            0x32 => HIDScanCode::M,              //
            0x33 => HIDScanCode::Comma,          //
            0x34 => HIDScanCode::Period,         //
            0x35 => HIDScanCode::ForwardSlash,   //
            0x36 => HIDScanCode::RightShift,     //
            0x37 => HIDScanCode::KeypadMultiply, //
            0x38 => HIDScanCode::LeftAlt,        //
            0x39 => HIDScanCode::Space,          //
            0x3A => HIDScanCode::CapsLock,       //
            0x3B => HIDScanCode::F1,             //
            0x3C => HIDScanCode::F2,             //
            0x3D => HIDScanCode::F3,             //
            0x3E => HIDScanCode::F4,             //
            0x3F => HIDScanCode::F5,             //
            0x40 => HIDScanCode::F6,             //
            0x41 => HIDScanCode::F7,             //
            0x42 => HIDScanCode::F8,             //
            0x43 => HIDScanCode::F9,             //
            0x44 => HIDScanCode::F10,            //
            0x45 => HIDScanCode::KeypadNumLock,  //
            0x46 => HIDScanCode::ScrollLock,     //
            0x47 => HIDScanCode::Keypad7,        //
            0x48 => HIDScanCode::Keypad8,        //
            0x49 => HIDScanCode::Keypad9,        //
            0x4A => HIDScanCode::KeypadMinus,    //
            0x4B => HIDScanCode::Keypad4,        //
            0x4C => HIDScanCode::Keypad5,        //
            0x4D => HIDScanCode::Keypad6,        //
            0x4E => HIDScanCode::KeypadPlus,     //
            0x4F => HIDScanCode::Keypad1,        //
            0x50 => HIDScanCode::Keypad2,        //
            0x51 => HIDScanCode::Keypad3,        //
            0x52 => HIDScanCode::Keypad0,        //
            0x53 => HIDScanCode::KeypadDecimal,  //
            0x54 => HIDScanCode::PrintScreen, //Technically this should be SysRq but modern keyboards just bind SysRq to Alt+PrintScreen
            // 0x55 =>
            0x56 => HIDScanCode::NonUSHash,
            0x57 => HIDScanCode::F11, //
            0x58 => HIDScanCode::F12, //
            0x59 => HIDScanCode::KeypadEquals,
            // 0x5A
            // 0x5B
            0x5C => HIDScanCode::International6,
            // 0x5D
            // 0x5E power?
            // 0x5F sleep?
            // 0x60
            // 0x61 zoom?
            0x63 => HIDScanCode::Help,
            0x64 => HIDScanCode::F13,
            0x65 => HIDScanCode::F14,
            0x66 => HIDScanCode::F15,
            0x67 => HIDScanCode::F16,
            0x68 => HIDScanCode::F17,
            0x69 => HIDScanCode::F18,
            0x6A => HIDScanCode::F19,
            0x6B => HIDScanCode::F20,
            0x6C => HIDScanCode::F21,
            0x6D => HIDScanCode::F22,
            0x6E => HIDScanCode::F23,
            // 0x6F
            0x70 => HIDScanCode::International2,
            0x71 => HIDScanCode::LANG2, //?
            0x72 => HIDScanCode::LANG1, //?
            0x73 => HIDScanCode::International1,
            //0x74
            //0x75
            0x76 => HIDScanCode::LANG5,
            // 0x76 => HIDScanCode::F24, CONFLICT!
            0x77 => HIDScanCode::LANG4,
            0x78 => HIDScanCode::LANG3,
            0x79 => HIDScanCode::International4,
            //0x7A
            0x7B => HIDScanCode::International5,
            0x7C => HIDScanCode::Tab, //?
            0x7D => HIDScanCode::International3,
            0x7E => HIDScanCode::KeypadComma,
            //0x7F
            0xF1 => HIDScanCode::LANG2, //?
            0xF2 => HIDScanCode::LANG1, //?

            // 0x7E => Self::NumPadComma, //https://kbdlayout.info/KBDBR/virtualkeys
            // 0x73 => Self::International1, //https://kbdlayout.info/KBDBR/virtualkeys and https://kbdlayout.info/kbd106/virtualkeys

            // 0x70 => Self::KatakanaHiragana,
            // 0x77 => Self::KatakanaHiragana, //Hiragana
            // 0x78 => Self::KatakanaHiragana, //Katakana
            // 0x79 => Self::Henkan,
            // 0x7B => Self::Muhenkan,
            // 0x7D => Self::Yen,

            // 0xF2 => Self::HanguelEnglish,
            // 0xF1 => Self::Hanja,
            _ => panic!("{make_code} {v_key}"),
        }
    };

    //Alt Gr sends two keys, one real (v_key == VK_MENU) and one simulated (v_key == VK_CONTROL)
    if scancode == HIDScanCode::RightAlt && v_key == VK_CONTROL {
        return Ok(None);
    }

    //Pause|Break fixes
    if scancode == HIDScanCode::LeftControl && e1 {
        return Ok(Some(HIDScanCode::Pause));
    } else if scancode == HIDScanCode::KeypadNumLock && v_key == 0xFF {
        return Ok(None);
    }

    return Ok(Some(scancode));
}
