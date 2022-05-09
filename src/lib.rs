use std::{
    sync::{Arc, RwLock},
    thread::JoinHandle,
};

use event::{GenericDriverEvent, PathManager};
use flume::{Receiver, Sender};

pub mod driver_interface;
pub mod event;
pub mod interaction_profile;
pub mod keyboard;

/**
 * Instead of using static enum parameters, SuInput often uses Path variables
 *
 * Types of Path:
 *
 * /interaction_profiles/<vendor_name>/<type_name>
 *
 * /devices/<vendor_name>/<name>
 *
 * /inputs/<source>[_<position]/<component>
 *
 * /outputs/<source>[_<position>]/<component>
 *
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SuPath(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Time(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActionHandle(u64);

#[derive(Debug, Clone, Copy)]
pub struct Vec2D {
    pub x: f32,
    pub y: f32,
}

pub struct SuInputRuntime {
    pub driver2runtime_sender: Sender<GenericDriverEvent>,
    pub driver2runtime_receiver: Receiver<GenericDriverEvent>,
    pub paths: Arc<RwLock<PathManager>>,
    pub thread: JoinHandle<()>,
}

impl SuInputRuntime {
    pub fn new() -> Self {
        let (driver2runtime_sender, driver2runtime_receiver) =
            flume::bounded::<event::GenericDriverEvent>(100);

        let driver2runtime_receiver_clone = driver2runtime_receiver.clone();
        let thread = std::thread::spawn(move || {
            while let Ok(packet) = driver2runtime_receiver_clone.recv() {
                println!("{:?}", packet)
            }
        });

        Self {
            driver2runtime_sender,
            driver2runtime_receiver,
            paths: Arc::new(RwLock::new(PathManager::default())),
            thread,
        }
    }
}
