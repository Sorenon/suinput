use std::collections::HashMap;

use crate::{Path, Time};

#[derive(Debug, Default)]
pub struct DummyDriverManager(HashMap<String, Path>, HashMap<Path, String>);

pub trait DriverManager {
    fn send_device_event(&self, device_event: DeviceEvent);
    fn send_component_event(&self, component_event: InputComponentEvent);
    fn get_path(&mut self, path_string: &str) -> Path;
    fn get_path_string(&self, path: Path) -> Option<String>;
}

impl DriverManager for DummyDriverManager {
    fn send_device_event(&self, device_event: DeviceEvent) {
        println!("{:?}", device_event);
    }

    fn send_component_event(&self, component_event: InputComponentEvent) {
        println!("Input Event {{ device: {:?}, path: '{}', data: {:?} }}",
            component_event.device,
            self.get_path_string(component_event.path).unwrap(),
            component_event.data,
        );
    }

    fn get_path(&mut self, path_string: &str) -> Path {
        if let Some(&path) = self.0.get(path_string) {
            path
        } else {
            let path = Path(self.0.len() as u32);
            self.0.insert(path_string.to_owned(), path);
            self.1.insert(path, path_string.to_owned());
            path
        }
    }

    fn get_path_string(&self, path: Path) -> Option<String> {
        self.1.get(&path).map(|inner| inner.clone())
    }
}

#[derive(Debug, Clone)]
pub enum DeviceEvent {
    DeviceActivated { id: usize, ty: Path },
    DeviceDeactivated { id: usize },
}

#[derive(Debug, Clone)]
pub struct InputComponentEvent {
    pub device: usize,
    pub path: Path,
    pub time: Time,
    pub data: EventType,
}

#[derive(Debug, Clone, Copy)]
pub enum EventType {
    Button(ButtonEvent),
    Move2D(Move2D),
}

#[derive(Debug, Clone, Copy)]
pub enum ButtonEvent {
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
