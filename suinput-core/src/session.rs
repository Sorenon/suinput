use std::{
    num::NonZeroUsize,
    sync::{atomic::AtomicBool, Arc, Weak},
};

use flume::{Receiver, Sender};
use parking_lot::{Mutex, RwLock};
use suinput_types::action::ActionListener;
use thunderdome::Index;

use crate::{
    action::Action,
    action_set::ActionSet,
    instance::Instance,
    internal::inner_session::{InnerSession, Runtime2SessionEvent},
    runtime::{Driver2RuntimeEvent, Runtime},
    user::User,
};
use crate::{internal::types::HashMap, types::action_type::ActionType};

pub struct Session {
    pub(crate) runtime: Weak<Runtime>,
    pub(crate) instance: Weak<Instance>,
    pub(crate) window: Mutex<Option<NonZeroUsize>>,

    pub user: Arc<User>,

    pub(crate) listeners: RwLock<Vec<Box<dyn ActionListener>>>,

    pub(crate) action_sets: Vec<Arc<ActionSet>>,
    pub(crate) actions: HashMap<u64, Arc<Action>>, //TODO dynamic action sets

    pub(crate) driver_events_send: Sender<Runtime2SessionEvent>,
    pub(crate) driver_events_rec: Receiver<Runtime2SessionEvent>,
    // pub(crate) action_events: RingBuffer<Driver2RuntimeEvent>,
    pub(crate) inner: Mutex<InnerSession>,
}

impl Session {
    pub fn set_window(&self, window: usize) {
        {
            *self.window.lock() = Some(window.try_into().unwrap());
        }

        self.runtime.upgrade().unwrap().refresh_windows();
    }

    pub fn poll(&self) {
        let mut inner = self.inner.lock();
        inner.sync(
            self.runtime.upgrade().unwrap(),
            &self.driver_events_rec,
            &self.user,
            &self.actions,
            &mut self.listeners.write(),
        );
    }

    pub fn register_event_listener(&self, listener: Box<dyn ActionListener>) -> u64 {
        let mut listeners = self.listeners.write();
        listeners.push(listener);
        listeners.len() as u64
    }

    pub fn unstick_bool_action(&self, action: &Action) {
        let runtime = self.runtime.upgrade().unwrap();

        // runtime
        //     .worker_thread_sender
        //     .send(WorkerThreadEvent::Output(OutputEvent::ReleaseStickyBool {
        //         session: self.runtime_handle,
        //         action: action.handle,
        //     }))
        //     .unwrap();

        todo!()
    }

    pub fn get_action_state<T: ActionType>(&self, action: &Action) -> Result<T::Value, ()> {
        let action_states = self.user.action_states.read();
        action_states
            .get(&action.handle)
            .map(|state| T::from_ase(state))
            .flatten()
            .ok_or(())
    }
}
