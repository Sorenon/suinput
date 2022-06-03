use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};

use once_cell::sync::OnceCell;
use parking_lot::{Mutex, RwLock};
use suinput_types::{
    binding::SimpleBinding, event::PathFormatError, CreateBindingLayoutError, SuPath,
};

use crate::{
    action::Action,
    internal::binding::binding_engine::ProcessedBindingLayout,
    session::{self, Session},
    user::User,
};

use super::{action_set::ActionSet, runtime::Runtime};

pub struct Instance {
    pub handle: u64,
    pub(crate) runtime: Weak<Runtime>,

    name: String,

    // action_sets: RwLock<Vec<Arc<ActionSet>>>,

    //TODO should this be a generational arena or is that overkill?
    pub(crate) actions: RwLock<Vec<Arc<Action>>>,

    pub(crate) sessions: RwLock<Vec<Arc<Session>>>,

    pub(crate) default_binding_layouts: RwLock<HashMap<SuPath, Arc<BindingLayout>>>,
}

impl Instance {
    pub fn new(
        runtime: &Arc<Runtime>,
        handle: u64,
        name: String,
        persistent_unique_id: (),
    ) -> Self {
        Instance {
            handle,
            runtime: Arc::downgrade(&runtime),
            name,
            actions: RwLock::default(),
            sessions: RwLock::default(),
            default_binding_layouts: RwLock::default(),
            // action_sets: Default::default(),
        }
    }

    pub fn get_path(&self, path_string: &str) -> Result<SuPath, PathFormatError> {
        let runtime = self.runtime.upgrade().unwrap();
        runtime.paths.get_path(path_string)
    }

    pub fn get_path_string(&self, path: SuPath) -> Option<String> {
        let runtime = self.runtime.upgrade().unwrap();
        runtime.paths.get_path_string(path)
    }

    pub fn create_action_set(
        self: &Arc<Self>,
        name: String,
        default_priority: u32,
    ) -> Arc<ActionSet> {
        let action_set = Arc::new(ActionSet {
            handle: 0,
            instance: Arc::downgrade(self),
            name,
            default_priority,
            actions: Default::default(),
            baked_actions: OnceCell::new(),
        });
        // self.action_sets.write().push(action_set.clone());
        action_set
    }

    pub fn create_localization(&self, identifier: String) {
        todo!()
    }

    pub fn create_binding_layout(
        &self,
        name: &str,
        interaction_profile: SuPath,
        bindings: &[SimpleBinding],
    ) -> Result<Arc<BindingLayout>, CreateBindingLayoutError> {
        let bindings = bindings.to_vec();

        ProcessedBindingLayout::new(self, interaction_profile, &bindings)
            .map(|processed| {
                Arc::new(BindingLayout {
                    name: name.into(),
                    interaction_profile,
                    processed,
                    bindings: bindings.to_vec(),
                })
            })
            .map_err(|err| {
                log::error!("SuInput: create_binding_layout failed with {err}");
                err
            })
    }

    pub fn set_default_binding_layout(
        &self,
        interaction_profile: SuPath,
        binding_layout: &Arc<BindingLayout>,
    ) {
        self.default_binding_layouts
            .write()
            .insert(interaction_profile, binding_layout.clone());
    }

    pub fn create_session(
        self: &Arc<Self>,
        action_sets: &[&Arc<ActionSet>],
    ) -> Arc<session::Session> {
        let session = {
            let runtime = self.runtime.upgrade().unwrap();

            //Sessions are owned by both the runtime and their instance
            let mut runtime_sessions = runtime.sessions.write();
            let mut instance_sessions = self.sessions.write();

            let binding_layouts = self
                .default_binding_layouts
                .read()
                .iter()
                .map(|(&profile, layout)| (profile, layout.processed.clone()))
                .collect();

            let user = User {
                action_states: RwLock::default(),
                new_binding_layouts: Mutex::new(binding_layouts),
            };

            let action_sets = action_sets
                .iter()
                .map(|action_set| {
                    if action_set.baked_actions.get().is_none() {
                        let actions = action_set.actions.read();
                        //We don't care if this failed somehow
                        std::mem::forget(action_set.baked_actions.set(actions.clone()));
                    }

                    (*action_set).clone()
                })
                .collect::<Vec<_>>();

            let session = Arc::new(session::Session {
                runtime_handle: runtime_sessions.len() as u64 + 1,
                runtime: self.runtime.clone(),
                instance: Arc::downgrade(self),
                user: Arc::new(user),
                listeners: RwLock::default(),
                window: Mutex::new(None),
                actions: action_sets
                    .iter()
                    .flat_map(|action_set| {
                        action_set
                            .baked_actions
                            .get()
                            .unwrap()
                            .iter()
                            .map(|action| (action.handle, action.clone()))
                    })
                    .collect(),
                action_sets,
            });

            runtime_sessions.push(session.clone());
            instance_sessions.push(session.clone());

            session
        };

        session.poll();

        session
    }

    /**
     * Creates all the needed actions on the xr_instance and attaches them to the xr_session
     * If SuInput does not provide a needed action type users can pass action sets containing that type to be attached
     *
     * Returns true if call was successful
     *
     * Returns false if xrAttachSessionActionSets had already been called for the specified session and the OpenXR layer
     * is not installed. In this case the Instance will have to rely on the developer provided OpenXR fallback driver.
     * This will occur on most pre-existing game engines and will may require altering the engine's OpenXR plugin.
     */
    pub fn bind_openxr(
        &self,
        xr_instance: (),
        xr_session: (),
        extra_xr_action_sets: Option<()>,
    ) -> bool {
        todo!()
    }
}

pub struct BindingLayout {
    pub name: String,
    pub interaction_profile: SuPath,
    pub bindings: Vec<SimpleBinding>,

    pub processed: ProcessedBindingLayout,
}
