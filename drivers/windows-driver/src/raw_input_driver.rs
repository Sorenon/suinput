use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    os::windows::prelude::OsStrExt,
};

use suinput_types::{
    driver_interface::RuntimeInterfaceTrait,
    event::{InputComponentEvent, InputEvent},
    keyboard::{HIDScanCode, KeyboardPaths},
    Time,
};
use windows_sys::Win32::{
    Devices::HumanInterfaceDevice::MOUSE_MOVE_ABSOLUTE,
    Foundation::{HANDLE, HWND},
    System::{LibraryLoader::GetModuleHandleW, SystemInformation::GetTickCount},
    UI::{
        Input::{RAWINPUT, RIM_TYPEKEYBOARD, RIM_TYPEMOUSE},
        WindowsAndMessaging::*,
    },
};

use crate::{paths::CommonPaths, raw_input, Result};
use crate::{
    raw_input::get_raw_input_data, raw_input::get_ri_device_info, raw_input::RIDeviceInfo, Error,
};

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

        ShowWindow(hwnd, 1);
        return Ok(hwnd);
    }
}

struct RawInputDriver<'a> {
    driver_manager: &'a dyn RuntimeInterfaceTrait,
    paths: CommonPaths,
    keyboard_paths: KeyboardPaths,
    device_ids: HashMap<HANDLE, u64>,
    ri_devices: HashMap<HANDLE, RIDeviceInfo>,
    keyboard_states: HashMap<HANDLE, HashSet<HIDScanCode>>,
    system_mouse_id: u64,
    system_keyboard_id: u64,
}

impl<'a> RawInputDriver<'a> {
    fn device_change(&mut self, raw_input_device: HANDLE, arrival: bool) -> Result<()> {
        if arrival {
            let rid_device_info = get_ri_device_info(raw_input_device)?;

            let device_type = match rid_device_info {
                RIDeviceInfo::Mouse { .. } => self.paths.mouse,
                RIDeviceInfo::Keyboard { .. } => self.paths.keyboard,
                RIDeviceInfo::GenericHID { .. } => todo!(),
            };

            let device_id = self.driver_manager.register_new_device(device_type);

            self.device_ids.insert(raw_input_device, device_id.unwrap());
            self.ri_devices.insert(raw_input_device, rid_device_info);
        } else {
            self.ri_devices.remove(&raw_input_device).unwrap();
            self.driver_manager
                .disconnect_device(self.device_ids.remove(&raw_input_device).unwrap())
                .unwrap();
        }
        Ok(())
    }

    fn input_event(&mut self, raw_input_device: HANDLE, raw_input_data: RAWINPUT) {
        let device_id = self.device_ids.get(&raw_input_device);

        match raw_input_data.header.dwType {
            RIM_TYPEMOUSE => {
                let device_id = *device_id
                    .filter(|_| {
                        matches!(
                            self.ri_devices.get(&raw_input_device),
                            Some(RIDeviceInfo::Mouse { .. })
                        )
                    })
                    .unwrap_or(&self.system_mouse_id);
                process_mouse(raw_input_data, device_id, self.driver_manager, &self.paths);
            }
            RIM_TYPEKEYBOARD => {
                let device_id = *device_id
                    .filter(|_| {
                        matches!(
                            self.ri_devices.get(&raw_input_device),
                            Some(RIDeviceInfo::Keyboard { .. })
                        )
                    })
                    .unwrap_or(&self.system_keyboard_id);

                self.process_keyboard(device_id, raw_input_device, raw_input_data);
            }
            _ => todo!(),
        }
    }

    fn process_keyboard(
        &mut self,
        device_id: u64,
        raw_input_device: HANDLE,
        raw_input_data: RAWINPUT,
    ) {
        let keyboard = unsafe { raw_input_data.data.keyboard };
        let flags = keyboard.Flags as u32;

        // WM_CHAR
        // WM_KEYDOWN
        // assert_eq!(keyboard.Message, WM_CHAR);

        let e0 = flags & RI_KEY_E0 != 0;
        let e1 = flags & RI_KEY_E1 != 0;

        let hid_scan_code =
            raw_input::raw_input_to_hid_scancode(keyboard.MakeCode, e0, keyboard.VKey, e1);

        //https://github.com/rust-lang/rust/issues/87335 pls
        let hid_scan_code = if let Some(hid_scan_code) = hid_scan_code.ok().flatten() {
            hid_scan_code
        } else {
            return;
        };

        let keyboard_state =
            if let Some(keyboard_state) = self.keyboard_states.get_mut(&raw_input_device) {
                keyboard_state
            } else {
                self.keyboard_states
                    .insert(raw_input_device, HashSet::new());
                self.keyboard_states.get_mut(&raw_input_device).unwrap()
            };

        if flags & RI_KEY_BREAK == 0 {
            //Key pressed
            if !keyboard_state.contains(&hid_scan_code) {
                keyboard_state.insert(hid_scan_code);
                let event = InputEvent {
                    device: device_id,
                    path: self.keyboard_paths.get(hid_scan_code),
                    time: suinput_types::Time(0),
                    data: InputComponentEvent::Button(true),
                };
                self.driver_manager.send_component_event(event).unwrap();
            }
        } else {
            //Key released
            if keyboard_state.contains(&hid_scan_code) {
                keyboard_state.remove(&hid_scan_code);
                let event = InputEvent {
                    device: device_id,
                    path: self.keyboard_paths.get(hid_scan_code),
                    time: suinput_types::Time(0),
                    data: InputComponentEvent::Button(false),
                };
                self.driver_manager.send_component_event(event).unwrap();
            }
        };
    }
}

pub fn run(driver_manager: &dyn RuntimeInterfaceTrait) {
    let window = create_background_window().unwrap();
    //TODO check if we loose raw input priority
    raw_input::register_raw_input_classes(window).unwrap();

    let paths = CommonPaths::new(driver_manager);
    let keyboard_paths =
        KeyboardPaths::new(|path_string| driver_manager.get_path(path_string).unwrap());

    let mut raw_input_driver = RawInputDriver {
        device_ids: HashMap::new(),
        ri_devices: HashMap::new(),
        keyboard_states: HashMap::new(),
        system_mouse_id: driver_manager.register_new_device(paths.mouse).unwrap(),
        system_keyboard_id: driver_manager.register_new_device(paths.keyboard).unwrap(),
        paths,
        keyboard_paths,
        driver_manager,
    };

    unsafe {
        let mut msg = std::mem::zeroed();
        while GetMessageW(&mut msg, window, 0, 0) > 0 {
            //TODO brush up on my overflow maths and figure this out
            //https://devblogs.microsoft.com/oldnewthing/20140122-00/?p=2013
            let _time = GetTickCount();
            let _message_time = GetMessageTime() as u32;

            match msg.message {
                WM_INPUT_DEVICE_CHANGE => {
                    raw_input_driver
                        .device_change(msg.lParam as HANDLE, msg.wParam == GIDC_ARRIVAL as usize)
                        .unwrap();
                }
                WM_INPUT => {
                    let raw_input_data = get_raw_input_data(msg.lParam as _).unwrap();

                    let raw_input_device = raw_input_data.header.hDevice;

                    raw_input_driver.input_event(raw_input_device, raw_input_data);
                }
                _ => (),
            }

            DispatchMessageW(&msg);
        }
    }
}

pub fn process_mouse(
    raw_input_data: RAWINPUT,
    device_id: u64,
    driver_manager: &dyn RuntimeInterfaceTrait,
    paths: &CommonPaths,
) {
    let mouse = unsafe { raw_input_data.data.mouse };

    if mouse.usFlags & (MOUSE_MOVE_ABSOLUTE as u16) == 0 {
        if mouse.lLastX != 0 || mouse.lLastY != 0 {
            driver_manager
                .send_component_event(InputEvent {
                    device: device_id,
                    path: paths.mouse_move,
                    time: Time(0),
                    data: InputComponentEvent::Move2D(
                        [mouse.lLastX as f64, mouse.lLastY as f64 * -1.].into(),
                    ),
                })
                .unwrap();
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
            driver_manager
                .send_component_event(InputEvent {
                    device: device_id,
                    path: paths.mouse_left_click,
                    time: Time(0),
                    data: InputComponentEvent::Button(true),
                })
                .unwrap();
        } else if flags & RI_MOUSE_BUTTON_1_UP != 0 {
            driver_manager
                .send_component_event(InputEvent {
                    device: device_id,
                    path: paths.mouse_left_click,
                    time: Time(0),
                    data: InputComponentEvent::Button(false),
                })
                .unwrap();
        }
        if flags & RI_MOUSE_BUTTON_2_DOWN != 0 {
            driver_manager
                .send_component_event(InputEvent {
                    device: device_id,
                    path: paths.mouse_right_click,
                    time: Time(0),
                    data: InputComponentEvent::Button(true),
                })
                .unwrap();
        } else if flags & RI_MOUSE_BUTTON_2_UP != 0 {
            driver_manager
                .send_component_event(InputEvent {
                    device: device_id,
                    path: paths.mouse_right_click,
                    time: Time(0),
                    data: InputComponentEvent::Button(false),
                })
                .unwrap();
        }
        if flags & RI_MOUSE_BUTTON_3_DOWN != 0 {
            driver_manager
                .send_component_event(InputEvent {
                    device: device_id,
                    path: paths.mouse_middle_click,
                    time: Time(0),
                    data: InputComponentEvent::Button(true),
                })
                .unwrap();
        } else if flags & RI_MOUSE_BUTTON_3_UP != 0 {
            driver_manager
                .send_component_event(InputEvent {
                    device: device_id,
                    path: paths.mouse_middle_click,
                    time: Time(0),
                    data: InputComponentEvent::Button(false),
                })
                .unwrap();
        }
        if flags & RI_MOUSE_BUTTON_4_DOWN != 0 {
            driver_manager
                .send_component_event(InputEvent {
                    device: device_id,
                    path: paths.mouse_button4_click,
                    time: Time(0),
                    data: InputComponentEvent::Button(true),
                })
                .unwrap();
        } else if flags & RI_MOUSE_BUTTON_4_UP != 0 {
            driver_manager
                .send_component_event(InputEvent {
                    device: device_id,
                    path: paths.mouse_button4_click,
                    time: Time(0),
                    data: InputComponentEvent::Button(false),
                })
                .unwrap();
        }
        if flags & RI_MOUSE_BUTTON_5_DOWN != 0 {
            driver_manager
                .send_component_event(InputEvent {
                    device: device_id,
                    path: paths.mouse_button5_click,
                    time: Time(0),
                    data: InputComponentEvent::Button(true),
                })
                .unwrap();
        } else if flags & RI_MOUSE_BUTTON_5_UP != 0 {
            driver_manager
                .send_component_event(InputEvent {
                    device: device_id,
                    path: paths.mouse_button5_click,
                    time: Time(0),
                    data: InputComponentEvent::Button(false),
                })
                .unwrap();
        }
        if flags & RI_MOUSE_WHEEL != 0 {
            driver_manager
                .send_component_event(InputEvent {
                    device: device_id,
                    path: paths.mouse_scroll,
                    time: Time(0),
                    data: InputComponentEvent::Move2D(
                        [0., (data.usButtonData as i16) as f64 / WHEEL_DELTA as f64].into(),
                    ),
                })
                .unwrap();
        }
        if flags & RI_MOUSE_HWHEEL != 0 {
            driver_manager
                .send_component_event(InputEvent {
                    device: device_id,
                    path: paths.mouse_scroll,
                    time: Time(0),
                    data: InputComponentEvent::Move2D(
                        [(data.usButtonData as i16) as f64 / WHEEL_DELTA as f64, 0.].into(),
                    ),
                })
                .unwrap();
        }
    }
}
