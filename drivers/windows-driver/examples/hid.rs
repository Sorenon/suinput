use std::os::windows::prelude::OsStrExt;

use windows_driver::{hid::DeviceInfoSet, hid_cm::get_container_id};
use windows_sys::{Win32::Devices::HumanInterfaceDevice::{
    GUID_DEVINTERFACE_KEYBOARD, GUID_DEVINTERFACE_MOUSE,
}, core::GUID};

fn main() {
    let mice_device_info_set = DeviceInfoSet::new(Some(&GUID_DEVINTERFACE_MOUSE)).unwrap();
    let kbd_device_info_set = DeviceInfoSet::new(Some(&GUID_DEVINTERFACE_KEYBOARD)).unwrap();

    for (device_interface_name, _) in mice_device_info_set.iter_device_interfaces(GUID_DEVINTERFACE_MOUSE) {
        println!("Mouse:{}", device_interface_name.to_string_lossy());
        print_guid(&get_container_id(&device_interface_name).unwrap());
    }

    for (device_interface_name, device) in kbd_device_info_set.iter_device_interfaces(GUID_DEVINTERFACE_KEYBOARD) {
        println!("Keyboard:{}", device_interface_name.to_string_lossy());
        print_guid(&kbd_device_info_set.get_container_id(&device).unwrap());
    }
}

fn print_guid(guid: &GUID) {
    println!(
        "{:x?}-{:x?}-{:x?}-{:x?}-{:x?}",
        guid.data1,
        guid.data2,
        guid.data3,
        u16::from_be_bytes([guid.data4[0], guid.data4[1]]),
        u64::from_be_bytes({
            let mut final_bit = [0; 8];
            final_bit[2..].copy_from_slice(&guid.data4[2..]);
            final_bit
        })
    );
}
