use std::{
    ffi::{OsStr, OsString},
    mem::size_of,
    os::windows::prelude::{OsStrExt, OsStringExt},
    ptr,
};

use windows_sys::{
    core::GUID,
    Win32::Devices::{
        DeviceAndDriverInstallation::{
            CM_Get_Device_Interface_ListW, CM_Get_Device_Interface_List_SizeW,
            CM_Get_Device_Interface_PropertyW, CONFIGRET, CR_BUFFER_SMALL, CR_SUCCESS,
        },
        Properties::DEVPKEY_Device_ContainerId,
    },
};

use crate::{Error, Result};

fn cm_result(ret: CONFIGRET) -> Result<()> {
    if ret == CR_SUCCESS {
        Ok(())
    } else {
        Err(Error::CfgMgr(ret))
    }
}

pub struct DeviceInterfaceIterator<'a> {
    start_idx: usize,
    buffer: &'a mut Vec<u16>,
}

impl<'a> DeviceInterfaceIterator<'a> {
    pub fn new(buffer: &'a mut Vec<u16>, device_interface_class: &GUID) -> Result<Self> {
        unsafe {
            let device_id = ptr::null();
            let flags = 53;

            let mut buffer_len = 0;

            let mut get_result = CR_BUFFER_SMALL;

            while get_result == CR_BUFFER_SMALL {
                cm_result(CM_Get_Device_Interface_List_SizeW(
                    &mut buffer_len,
                    device_interface_class,
                    device_id,
                    flags,
                ))?;

                buffer.set_len(0);
                buffer.reserve(buffer_len as usize);

                get_result = CM_Get_Device_Interface_ListW(
                    device_interface_class,
                    device_id,
                    buffer.as_mut_ptr(),
                    buffer_len,
                    flags,
                );
            }

            cm_result(get_result)?;

            buffer.set_len(buffer_len as usize);
        }

        Ok(Self {
            start_idx: 0,
            buffer,
        })
    }
}

impl<'a> Iterator for DeviceInterfaceIterator<'a> {
    type Item = OsString;

    fn next(&mut self) -> Option<Self::Item> {
        let buffer_slice = &self.buffer[self.start_idx..];

        for (idx, &c) in buffer_slice.iter().enumerate() {
            if c == 0 {
                //Two nulls in a row indicates the end of the set
                if idx == 0 {
                    break;
                }

                let device_instance_id = OsString::from_wide(&buffer_slice[..=idx]);

                self.start_idx += idx + 1;

                return Some(device_instance_id);
            }
        }
        None
    }
}

pub fn get_container_id(cm_device_interface: &OsStr) -> Result<GUID> {
    let mut container_id = unsafe { std::mem::zeroed() };
    cm_result(unsafe {
        CM_Get_Device_Interface_PropertyW(
            cm_device_interface
                .encode_wide()
                .collect::<Vec<u16>>()
                .as_ptr(),
            &DEVPKEY_Device_ContainerId,
            &mut std::mem::zeroed(),
            &mut container_id as *mut GUID as _,
            &mut (size_of::<GUID>() as u32),
            0,
        )
    })?;
    Ok(container_id)
}
