use std::{
    sync::{Arc, Weak},
};

use parking_lot::{Mutex, RwLock};

use crate::internal::types::HashMap;
use crate::{
    action::Action,
    action_set::ActionSet,
    instance::{BindingLayout, Instance},
    internal::{inner_session::InnerSession, worker_thread::WorkerThreadEvent},
    runtime::Runtime,
    session::Session,
    user::User,
};

pub struct ApplicationInstance {
    pub(crate) runtime: Weak<Runtime>,
    pub(crate) instance: Weak<Instance>,

    pub(crate) action_sets: HashMap<u64, Arc<ActionSet>>,
    pub(crate) actions: HashMap<u64, Arc<Action>>, //TODO dynamic action sets
    pub(crate) binding_layouts: Vec<Arc<BindingLayout>>,

    pub(crate) session: RwLock<Option<Weak<Session>>>,
}

impl ApplicationInstance {
    pub fn create_session(self: &Arc<Self>) -> Arc<Session> {
        let mut lock = self.session.write();
        assert!(lock.is_none());

        let binding_layouts = self
            .binding_layouts
            .iter()
            .map(|layout| (layout.interaction_profile, layout.processed_cache.clone()))
            .collect();

        let user = User {
            action_states: RwLock::default(),
            new_binding_layouts: Mutex::new(binding_layouts),
        };

        let (driver_events_send, driver_events_rec) = flume::unbounded();

        let runtime = self.runtime.upgrade().unwrap();

        let session = Arc::new(Session {
            runtime: self.runtime.clone(),
            window: Mutex::new(None),
            app_instance: self.clone(),
            user: Arc::new(user),
            listeners: RwLock::default(),
            inner: Mutex::new(InnerSession::new(&runtime, &self.action_sets)),
            driver_events_send,
            driver_events_rec,
            action_events: flume::unbounded(),
        });

        *lock = Some(Arc::downgrade(&session));

        let handle = runtime.sessions.write().insert(session.clone());
        runtime
            .worker_thread_sender
            .send(WorkerThreadEvent::CreateSession { handle })
            .unwrap();

        session
    }
}
