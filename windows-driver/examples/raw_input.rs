use windows_driver::raw_input::*;

fn main() -> anyhow::Result<()> {
    let devices = get_ri_devices()?;

    for i_device in devices {
        println!("{}, {}", i_device.hDevice, i_device.dwType);
        println!("{:?}", get_ri_device_info(i_device.hDevice)?);
        println!("-{:?}", get_rid_device_interface_name(i_device.hDevice));
    }

    Ok(())
}
