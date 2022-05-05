use crate::{Path, Time};

pub struct DummyDriverManager;

pub trait DriverManager {
    fn send_device_event(&self, device_event: DeviceEvent);
    fn send_component_event(&self, component_event: InputComponentEvent);
}

impl DriverManager for DummyDriverManager {
    fn send_device_event(&self, device_event: DeviceEvent) {
        println!("{:?}", device_event);
    }

    fn send_component_event(&self, component_event: InputComponentEvent) {
        println!("{:?}", component_event);
    }
}

#[derive(Debug, Clone)]
pub enum DeviceEvent {
    DeviceActivated { id: usize, ty: String },
    DeviceDeactivated { id: usize },
}

#[derive(Debug, Clone, Copy)]
pub struct InputComponentEvent {
    pub device: usize,
    pub path: Path,
    pub time: Time,
    pub data: EventType,
}

#[derive(Debug, Clone, Copy)]
pub enum EventType {
    Button(Button),
    Move2D(Move2D),
}

#[derive(Debug, Clone, Copy)]
pub enum Button {
    Press,
    Release,
}

#[derive(Debug, Clone, Copy)]
pub struct Move2D {
    pub value: (f64, f64),
}

#[derive(Debug, Clone, Copy)]
pub struct Cursor {
    pub normalized_screen_coords: (f64, f64),
}
