use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    os::windows::prelude::OsStrExt,
};

use suinput::{
    driver_interface::DriverRuntimeInterfaceTrait,
    event::{ButtonEvent, DeviceEvent, InputComponentEvent, InputEvent},
    keyboard::{self, HIDScanCode},
};
use windows_sys::Win32::{
    Foundation::{HANDLE, HWND},
    System::{LibraryLoader::GetModuleHandleW, SystemInformation::GetTickCount},
    UI::{
        Input::{RIM_TYPEHID, RIM_TYPEKEYBOARD, RIM_TYPEMOUSE},
        WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageTime, GetMessageW,
            RegisterClassW, ShowWindow, GIDC_ARRIVAL, RI_KEY_BREAK, RI_KEY_E0, RI_KEY_E1, WM_INPUT,
            WM_INPUT_DEVICE_CHANGE, WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
            WS_EX_TRANSPARENT, WS_OVERLAPPED,
        },
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

//TODO investigate using the application window for this with hooks
//^ This may improve the stability of the Unity plugin
pub fn run(driver_manager: &dyn DriverRuntimeInterfaceTrait) {
    let window = create_background_window().unwrap();
    raw_input::register_raw_input_classes(window).unwrap();
    unsafe {
        // let mut containers = HashMap::<OsString, Vec<HANDLE>>::new();
        let mut ri_devices = HashMap::<HANDLE, RIDeviceInfo>::new();
        let mut keyboard_states = HashMap::<HANDLE, HashSet<HIDScanCode>>::new();

        let paths = CommonPaths::new(driver_manager);
        let keyboard_paths = keyboard::KeyboardPaths::new(driver_manager);

        driver_manager.send_device_event(DeviceEvent::DeviceActivated {
            id: 0,
            ty: paths.mouse,
        });

        driver_manager.send_device_event(DeviceEvent::DeviceActivated {
            id: 0,
            ty: paths.keyboard,
        });

        let mut msg = std::mem::zeroed();
        while GetMessageW(&mut msg, window, 0, 0) > 0 {
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
                            RIDeviceInfo::Mouse { .. } => paths.mouse,
                            RIDeviceInfo::Keyboard { .. } => paths.keyboard,
                            RIDeviceInfo::GenericHID { .. } => todo!(),
                        };

                        ri_devices.insert(raw_input_device, rid_device_info);

                        driver_manager.send_device_event(DeviceEvent::DeviceActivated {
                            id: raw_input_device as _,
                            ty: device_type,
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
                            RIDeviceInfo::Mouse { .. } => paths.mouse,
                            RIDeviceInfo::Keyboard { .. } => paths.keyboard,
                            RIDeviceInfo::GenericHID { .. } => {
                                //Precision touch pads sometimes send a ctrl key
                                // panic!("`{raw_input_device}`_`{rid_device_info:?}`");
                                //TODO improve this
                                paths.keyboard
                            }
                        };

                        ri_devices.insert(raw_input_device, rid_device_info);

                        driver_manager.send_device_event(DeviceEvent::DeviceActivated {
                            id: raw_input_device as _,
                            ty: device_type,
                        });
                    }

                    match raw_input_data.header.dwType {
                        RIM_TYPEMOUSE => {
                            raw_input::process_mouse(raw_input_data, driver_manager, &paths)
                        }
                        RIM_TYPEKEYBOARD => {
                            let keyboard = raw_input_data.data.keyboard;
                            let flags = keyboard.Flags as u32;

                            // WM_CHAR
                            // WM_KEYDOWN
                            // assert_eq!(keyboard.Message, WM_CHAR);

                            let e0 = flags & RI_KEY_E0 != 0;
                            let e1 = flags & RI_KEY_E1 != 0;

                            let hid_scan_code = raw_input::raw_input_to_hid_scancode(
                                keyboard.MakeCode,
                                e0,
                                keyboard.VKey,
                                e1,
                            );

                            //https://github.com/rust-lang/rust/issues/87335 pls
                            let hid_scan_code =
                                if let Some(hid_scan_code) = hid_scan_code.ok().flatten() {
                                    hid_scan_code
                                } else {
                                    continue;
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
                                    let event = InputEvent {
                                        device: raw_input_device as usize,
                                        path: keyboard_paths.get(hid_scan_code),
                                        time: suinput::Time(0),
                                        data: InputComponentEvent::Button(ButtonEvent::Press),
                                    };
                                    driver_manager.send_component_event(event);
                                }
                            } else {
                                //Key released
                                if keyboard_state.contains(&hid_scan_code) {
                                    keyboard_state.remove(&hid_scan_code);
                                    let event = InputEvent {
                                        device: raw_input_device as usize,
                                        path: keyboard_paths.get(hid_scan_code),
                                        time: suinput::Time(0),
                                        data: InputComponentEvent::Button(ButtonEvent::Release),
                                    };
                                    driver_manager.send_component_event(event);
                                }
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
