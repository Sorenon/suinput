use std::{ffi::OsString, os::windows::prelude::OsStringExt, ptr};

use windows_driver::hid_cm::DeviceInterfaceIterator;
use windows_sys::Win32::Devices::{
    DeviceAndDriverInstallation::{
        CM_Get_Device_ID_ListW, CM_Get_Device_ID_List_SizeW, CM_GETIDLIST_FILTER_PRESENT,
        CR_SUCCESS,
    },
    HumanInterfaceDevice::GUID_DEVINTERFACE_MOUSE,
};

fn main() {
    let mut buffer = Vec::new();
    let iter = DeviceInterfaceIterator::new(&mut buffer, &GUID_DEVINTERFACE_MOUSE).unwrap();

    for mut a in iter {
        a.make_ascii_lowercase();
        println!("{}", a.to_string_lossy());
    }

    unsafe {
        iterate_device_ids();
    }
}

unsafe fn iterate_device_ids() {
    let mut size = 0;
    let flags = CM_GETIDLIST_FILTER_PRESENT;

    let result = CM_Get_Device_ID_List_SizeW(&mut size, ptr::null(), flags);
    if result != CR_SUCCESS {
        panic!("{result}");
    }

    let mut buffer = Vec::<u16>::with_capacity(size as usize);

    let result = CM_Get_Device_ID_ListW(ptr::null(), buffer.as_mut_ptr(), size, flags);
    if result != CR_SUCCESS {
        panic!("{result}");
    }

    buffer.set_len(size as usize);

    let mut last_start = 0;
    for (idx, &c) in buffer.iter().enumerate() {
        if c == 0 {
            //Two nulls in a row indicates the end of the set
            if last_start == idx {
                break;
            }

            let device_instance_id = OsString::from_wide(&buffer[last_start..=idx]);
            println!("`{}`", device_instance_id.to_string_lossy());
            last_start = idx + 1;
        }
    }
}
