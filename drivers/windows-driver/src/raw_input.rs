use std::{ffi::OsString, mem::size_of, os::windows::prelude::OsStringExt};

use suinput::{
    keyboard::HIDScanCode,
};
use windows_sys::Win32::{
    Foundation::{GetLastError, ERROR_INSUFFICIENT_BUFFER, HANDLE, HWND},
    UI::{
        Input::{
            GetRawInputData, GetRawInputDeviceInfoW, GetRawInputDeviceList,
            KeyboardAndMouse::VK_CONTROL, RegisterRawInputDevices, HRAWINPUT, RAWINPUT,
            RAWINPUTDEVICE, RAWINPUTDEVICELIST, RAWINPUTHEADER, RIDEV_DEVNOTIFY, RIDEV_INPUTSINK,
            RIDI_DEVICEINFO, RIDI_DEVICENAME, RID_DEVICE_INFO, RID_INPUT, RIM_TYPEHID,
            RIM_TYPEKEYBOARD, RIM_TYPEMOUSE,
        },
    },
};

use crate::{Error, Result};

/**
 * Returns a list of all raw input devices
 * RAWINPUTDEVICELIST.dwType is mostly useless
 */
pub fn get_ri_devices() -> Result<Vec<RAWINPUTDEVICELIST>> {
    //TODO investigate the cause of RAWINPUTDEVICELIST.dwType == 3
    //Is it worth replacing this with DeviceInfoSet
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
            _ => unreachable!("{}", device_info.dwType),
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

pub fn raw_input_to_hid_scancode(
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
            _ => {
                println!("{make_code}");
                return Err(Error::Win32(0));
            }
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
            0x54 => HIDScanCode::PrintScreen, //Technically this should be SysRq but modern keyboards just bind SysRq as Alt+PrintScreen
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
            0x76 => HIDScanCode::F24,
            // 0x76 => HIDScanCode::LANG5 <-- CONFLICT
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
            _ => {
                println!("{make_code} {v_key}");
                return Err(Error::Win32(0));
            }
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
