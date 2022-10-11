use std::fs::File;
use std::num::NonZeroU128;
use std::path::Path;
use std::sync::{Arc, Weak};

use parking_lot::{Mutex, RwLock};
use slotmap::DefaultKey;

use crate::internal::serial;
use crate::internal::types::HashMap;
use crate::user::OutActionStateEnum;
use crate::{
    action::Action,
    action_set::ActionSet,
    instance::{BindingLayout, Instance},
    internal::{inner_session::InnerSession, worker_thread::WorkerThreadEvent},
    runtime::Runtime,
    session::Session,
    types::{Error, Result},
    user::User,
};

pub struct ApplicationInstance {
    pub(crate) runtime: Weak<Runtime>,
    pub(crate) instance: Weak<Instance>,
    pub(crate) index: DefaultKey,

    pub(crate) application_name: String,
    pub(crate) sub_name: Option<String>,

    pub(crate) action_sets: HashMap<u64, Arc<ActionSet>>,
    pub(crate) actions: HashMap<u64, Arc<Action>>,
    //TODO dynamic action sets
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
            action_states: RwLock::new(
                self.actions
                    .values()
                    .map(|action| {
                        (
                            action.handle,
                            OutActionStateEnum::from_type(action.data_type),
                        )
                    })
                    .collect(),
            ),
            new_binding_layouts: Mutex::new(binding_layouts),
        };

        let (driver_events_send, driver_events_rec) = flume::unbounded();

        let runtime = self.runtime.upgrade().unwrap();

        let session = Arc::new(Session {
            runtime: self.runtime.clone(),
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

    pub fn make_persistent(&self, file_path: &Path) -> Result<()> {
        std::fs::create_dir_all(file_path.parent().ok_or(Error::ValidationFailure)?)
            .map_err(|_| Error::IoError)?;

        //TODO migrate to KDL
        serde_json::to_writer_pretty(
            File::create(file_path).map_err(|_| Error::IoError)?,
            &self.serialize(),
        )
        .map_err(|_| Error::RuntimeFailure)
    }

    //This is a terrible way of doing this
    pub fn serialize(&self) -> serial::ApplicationInstance {
        serial::ApplicationInstance {
            name: &self.application_name,
            sub_name: self.sub_name.as_deref(),
            unique_id: NonZeroU128::new(1).unwrap(),
            action_sets: self
                .action_sets
                .iter()
                .map(|set| set.1.serialize())
                .collect(),
            dynamic_action_sets: vec![],
            binding_layouts: vec![],
        }
    }
}
