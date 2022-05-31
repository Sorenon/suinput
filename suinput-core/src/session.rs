use std::{
    num::NonZeroUsize,
    sync::{Arc, Weak},
};

use parking_lot::{Mutex, RwLock};
use suinput_types::action::ActionListener;

use crate::{
    instance::Instance, internal::worker_thread::WorkerThreadEvent, runtime::Runtime, user::User,
};

pub struct Session {
    pub runtime_handle: u64,
    pub instance_handle: u64,

    pub(crate) runtime: Weak<Runtime>,
    pub(crate) instance: Weak<Instance>,
    pub(crate) window: Mutex<Option<NonZeroUsize>>,

    pub user: Arc<User>,

    pub(crate) listeners: RwLock<Vec<Box<dyn ActionListener>>>,
}

impl Session {
    pub fn set_window(&self, window: usize) {
        {
            *self.window.lock() = Some(window.try_into().unwrap());
        }

        self.runtime.upgrade().unwrap().refresh_windows();
    }

    pub fn poll(&self) {
        self.runtime
            .upgrade()
            .unwrap()
            .driver2runtime_sender
            .send(WorkerThreadEvent::Poll {
                session: self.runtime_handle,
            })
            .unwrap();
    }

    pub fn register_event_listener(&self, listener: Box<dyn ActionListener>) -> u64 {
        let mut listeners = self.listeners.write();
        listeners.push(listener);
        listeners.len() as u64
    }
}
