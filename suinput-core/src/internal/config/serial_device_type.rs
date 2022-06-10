use std::collections::HashMap;

use suinput_types::{event::PathFormatError, SuPath};

use crate::{
    internal::{device_type::DeviceType, input_component::InputComponentType, paths::PathManager},
    runtime::Runtime,
};

#[derive(Debug, knuffel::Decode)]
pub struct Device {
    #[knuffel(property)]
    pub vendor: String,
    #[knuffel(property)]
    pub name: String,

    #[knuffel(children(name = "identifier"))]
    pub identifiers: Vec<Identifier>,
}

#[derive(Debug, knuffel::Decode)]
pub struct Identifier {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(children)]
    pub components: Vec<Component>,
}

#[derive(Debug, knuffel::Decode)]
pub enum Component {
    Button(#[knuffel(argument)] String),
    Trigger(#[knuffel(argument)] String),
    Joystick(#[knuffel(argument)] String),
    Cursor(#[knuffel(argument)] String),
    Move2d(#[knuffel(argument)] String),
    Gyro(
        #[knuffel(argument)] String,
        #[knuffel(property(name = "calibrated"))] bool,
    ),
    Accel(#[knuffel(argument)] String),
    AdaptiveTrigger(#[knuffel(argument)] String),
    Touchpad(
        #[knuffel(argument)] String,
        #[knuffel(property(name = "max_points"))] u32,
        #[knuffel(property(name = "pressure"))] bool,
    ),
    Led(#[knuffel(argument)] String),
    PlayerNumber(
        #[knuffel(argument)] String,
        #[knuffel(property(name = "min"))] u32,
        #[knuffel(property(name = "max"))] u32,
    ),
    HdRumble(#[knuffel(argument)] String),
    Rumble(#[knuffel(argument)] String),
}

pub static DEVICES: &str = include_str!("devices.kdl"); 

#[rustfmt::skip]
pub fn deserialize(paths: &PathManager) -> Vec<DeviceType> {
    let devices = knuffel::parse::<Vec<Device>>(".kdl", DEVICES).unwrap();
    
    devices.iter().map(|device| {
        let id = paths.get_path(&format!("/devices/{}/{}", device.vendor, device.name)).unwrap();

        let input_components = device.identifiers.iter().flat_map(|identifier| {
            identifier.components.iter().filter_map(|component| {
                Some(match component {
                    Component::Button(name) => (InputComponentType::Button, name),
                    Component::Trigger(_) => return None,
                    Component::Joystick(_) => return None,
                    Component::Cursor(name) => (InputComponentType::Cursor, name),
                    Component::Move2d(name) => (InputComponentType::Move2D, name),
                    Component::Gyro(_, _) => return None,
                    Component::Accel(_) => return None,
                    Component::AdaptiveTrigger(_) => return None,
                    Component::Touchpad(_, _, _) => return None,
                    Component::Led(_) => return None,
                    Component::PlayerNumber(_, _, _) => return None,
                    Component::HdRumble(_) => return None,
                    Component::Rumble(_) => return None,
                })
            })
            .map(|(ty, name)| {
                Ok((paths.get_path(&format!("/input/{}/{}", identifier.name, name))?, ty))
            })
        }
    ).collect::<Result<HashMap<_, _>, PathFormatError>>().unwrap();

    DeviceType {
        id,
        input_components,
    }
    }).collect()
}
