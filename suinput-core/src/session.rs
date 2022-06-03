use std::{
    collections::HashMap,
    num::NonZeroUsize,
    sync::{Arc, Weak},
};

use parking_lot::{Mutex, RwLock};
use suinput_types::action::ActionListener;

use crate::{
    action::Action,
    action_set::ActionSet,
    instance::Instance,
    internal::worker_thread::{OutputEvent, WorkerThreadEvent},
    runtime::Runtime,
    user::User,
};

pub struct Session {
    pub runtime_handle: u64,

    pub(crate) runtime: Weak<Runtime>,
    pub(crate) instance: Weak<Instance>,
    pub(crate) window: Mutex<Option<NonZeroUsize>>,

    pub user: Arc<User>,

    pub(crate) listeners: RwLock<Vec<Box<dyn ActionListener>>>,

    pub(crate) action_sets: Vec<Arc<ActionSet>>,
    pub(crate) actions: HashMap<u64, Arc<Action>>, //TODO dynamic action sets
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
            .worker_thread_sender
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

    pub fn unstick_bool_action(&self, action: &Action) {
        let runtime = self.runtime.upgrade().unwrap();

        runtime
            .worker_thread_sender
            .send(WorkerThreadEvent::Output(OutputEvent::ReleaseStickyBool {
                session: self.runtime_handle,
                action: action.handle,
            }))
            .unwrap();
    }
}
