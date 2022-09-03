use std::{
    num::NonZeroUsize,
    sync::{Arc, Weak},
};

use flume::{Receiver, Sender};
use parking_lot::{Mutex, RwLock};
use suinput_types::action::ActionListener;

use crate::types::action_type::ActionType;
use crate::{
    action::Action,
    action_set::ActionSet,
    application_instance::ApplicationInstance,
    internal::inner_session::{InnerSession, Runtime2SessionEvent, SessionActionEvent},
    runtime::Runtime,
    user::User,
};

pub struct Session {
    pub(crate) runtime: Weak<Runtime>,

    pub(crate) window: Mutex<Option<NonZeroUsize>>,

    pub(crate) app_instance: Arc<ApplicationInstance>,

    pub user: Arc<User>,

    pub(crate) listeners: RwLock<Vec<Box<dyn ActionListener>>>,

    pub(crate) action_events: (Sender<SessionActionEvent>, Receiver<SessionActionEvent>),
    pub(crate) driver_events_send: Sender<Runtime2SessionEvent>,
    pub(crate) driver_events_rec: Receiver<Runtime2SessionEvent>,
    pub(crate) inner: Mutex<InnerSession>,
}

impl Session {
    pub fn set_window(&self, window: usize) {
        {
            *self.window.lock() = Some(window.try_into().unwrap());
        }

        self.runtime.upgrade().unwrap().refresh_windows();
    }

    pub fn sync<'a>(&self, action_sets: impl Iterator<Item = &'a Arc<ActionSet>>) {
        let mut inner = self.inner.lock();
        inner.sync(
            self.runtime.upgrade().unwrap(),
            action_sets,
            &self.app_instance.action_sets,
            &self.action_events.1,
            &self.driver_events_rec,
            &self.user,
            &self.app_instance.actions,
            &mut self.listeners.write(),
        );
    }

    pub fn register_event_listener(&self, listener: Box<dyn ActionListener>) -> u64 {
        let mut listeners = self.listeners.write();
        listeners.push(listener);
        listeners.len() as u64
    }

    pub fn unstick_bool_action(&self, action: &Action) {
        self.action_events
            .0
            .send(SessionActionEvent::Unstick {
                action: action.handle,
            })
            .unwrap();
    }

    pub fn get_action_state<T: ActionType>(&self, action: &Action) -> Result<T::State, ()> {
        let action_states = self.user.action_states.read();

        action_states
            .get(&action.handle)
            .and_then(|state| T::pick_state(state).copied())
            .ok_or(())
    }
}
