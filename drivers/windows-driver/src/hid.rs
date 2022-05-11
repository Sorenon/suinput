use std::{ffi::OsString, mem::size_of, os::windows::prelude::OsStringExt};

use windows_sys::{
    core::GUID,
    Win32::{
        Devices::{
            DeviceAndDriverInstallation::{
                SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInterfaces, SetupDiGetClassDevsW,
                SetupDiGetDeviceInterfaceDetailW, SetupDiGetDevicePropertyW, DIGCF_ALLCLASSES,
                DIGCF_DEVICEINTERFACE, DIGCF_PRESENT, SP_DEVICE_INTERFACE_DATA,
                SP_DEVICE_INTERFACE_DETAIL_DATA_W, SP_DEVINFO_DATA,
            },
            Properties::DEVPKEY_Device_ContainerId,
        },
        Foundation::{
            GetLastError, ERROR_INSUFFICIENT_BUFFER, ERROR_NO_MORE_ITEMS, HANDLE,
            INVALID_HANDLE_VALUE,
        },
    },
};

use crate::{Error, Result};

/*
    Device Interface:
    An interface exposed by a driver to allow UserSpace application I/O

    Device Interface Class:
    The type of device interface e.g. mouse or mountable

    Device Information Set:
    A list of Device Information Elements

    Device Information Element:
    A devnode and the list of Device Interfaces it has
*/

pub struct DeviceInfoSet(HANDLE);

impl DeviceInfoSet {
    pub fn new(class: Option<&GUID>) -> Result<Self> {
        let device_info_set = if let Some(class) = class {
            unsafe {
                SetupDiGetClassDevsW(
                    class,
                    std::ptr::null(),
                    0,
                    DIGCF_DEVICEINTERFACE | DIGCF_PRESENT,
                )
            }
        } else {
            unsafe {
                SetupDiGetClassDevsW(
                    std::ptr::null(),
                    std::ptr::null(),
                    0,
                    DIGCF_ALLCLASSES | DIGCF_DEVICEINTERFACE | DIGCF_PRESENT,
                )
            }
        };

        if device_info_set == INVALID_HANDLE_VALUE {
            Err(Error::win32())
        } else {
            Ok(DeviceInfoSet(device_info_set))
        }
    }

    pub fn iter_device_interfaces(&self, class: GUID) -> DeviceInterfaceIterator {
        DeviceInterfaceIterator {
            idx: 0,
            class,
            device_info_set: self,
        }
    }

    pub fn get_container_id(&self, device_info_data: &SP_DEVINFO_DATA) -> Result<GUID> {
        let mut container_id = unsafe { std::mem::zeroed() };
        if unsafe {
            SetupDiGetDevicePropertyW(
                self.0,
                device_info_data,
                &DEVPKEY_Device_ContainerId,
                &mut 0,
                &mut container_id as *mut GUID as _,
                size_of::<GUID>() as u32,
                std::ptr::null_mut(),
                0,
            )
        } == 0
        {
            Err(Error::win32())
        } else {
            Ok(container_id)
        }
    }
}

impl Drop for DeviceInfoSet {
    fn drop(&mut self) {
        if unsafe { SetupDiDestroyDeviceInfoList(self.0) } == 0 {
            panic!("{}", Error::win32());
        }
    }
}

pub struct DeviceInterfaceIterator<'a> {
    idx: u32,
    class: GUID,
    device_info_set: &'a DeviceInfoSet,
}

impl Iterator for DeviceInterfaceIterator<'_> {
    type Item = (OsString, SP_DEVINFO_DATA);

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let mut device_interface_data: SP_DEVICE_INTERFACE_DATA = std::mem::zeroed();
            device_interface_data.cbSize = size_of::<SP_DEVICE_INTERFACE_DATA>() as u32;

            if SetupDiEnumDeviceInterfaces(
                self.device_info_set.0,
                std::ptr::null(),
                &self.class,
                self.idx,
                &mut device_interface_data,
            ) == 0
            {
                if GetLastError() == ERROR_NO_MORE_ITEMS {
                    return None;
                } else {
                    panic!("{}", Error::win32());
                }
            }

            self.idx += 1;

            let mut size = 0;

            let mut device_info_data = SP_DEVINFO_DATA {
                cbSize: size_of::<SP_DEVINFO_DATA>() as u32,
                ..std::mem::zeroed()
            };

            if SetupDiGetDeviceInterfaceDetailW(
                self.device_info_set.0,
                &device_interface_data,
                std::ptr::null_mut(),
                0,
                &mut size,
                &mut device_info_data,
            ) == 0
                && GetLastError() != ERROR_INSUFFICIENT_BUFFER
            {
                panic!("{}", Error::win32());
            }

            let mut buffer = Vec::<u8>::with_capacity(size as usize);
            let detail = buffer.as_mut_ptr() as *mut SP_DEVICE_INTERFACE_DETAIL_DATA_W;
            (*detail).cbSize = size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>() as u32;

            if SetupDiGetDeviceInterfaceDetailW(
                self.device_info_set.0,
                &device_interface_data,
                buffer.as_mut_ptr() as _,
                size,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            ) == 0
            {
                panic!("{}", Error::win32());
            }

            let device_interface_name = OsString::from_wide(std::slice::from_raw_parts(
                (*detail).DevicePath.as_ptr(),
                (size as usize - size_of::<u32>()) / 2,
            ));

            Some((device_interface_name, device_info_data))
        }
    }
}
