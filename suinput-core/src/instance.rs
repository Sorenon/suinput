use std::sync::{Arc, Weak};

use parking_lot::RwLock;
use suinput_types::{
    action::ActionListener, binding::SimpleBinding, event::PathFormatError, SuPath,
};

use crate::{
    action::Action,
    internal::{binding::binding_engine::ProcessedBindingLayout, worker_thread::WorkerThreadEvent},
    user::User,
};

use super::{action_set::ActionSet, runtime::Runtime};

pub struct Instance {
    pub handle: u64,
    pub(crate) runtime: Weak<Runtime>,

    name: String,

    // action_sets: RwLock<Vec<Arc<ActionSet>>>,
    pub(crate) user: RwLock<User>,
    pub(crate) listeners: RwLock<Vec<Box<dyn ActionListener>>>,

    //TODO should this be a generational arena or is that overkill?
    pub(crate) actions: RwLock<Vec<Arc<Action>>>,
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
            user: RwLock::default(),
            listeners: RwLock::default(),
            actions: RwLock::default(),
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
    ) -> Arc<BindingLayout> {
        Arc::new(BindingLayout {
            name: name.into(),
            interaction_profile,
            bindings: bindings.to_vec(),
        })
    }

    pub fn set_default_binding_layout(
        &self,
        interaction_profile: SuPath,
        binding_layout: &Arc<BindingLayout>,
    ) {
        let mut player = self.user.write();
        player
            .default_binding_layout
            .insert(interaction_profile, binding_layout.clone());

        player.new_binding_layouts.insert(
            interaction_profile,
            ProcessedBindingLayout::new(self, interaction_profile, binding_layout),
        );
    }

    pub fn poll(&self) {
        self.runtime
            .upgrade()
            .unwrap()
            .driver2runtime_sender
            .send(WorkerThreadEvent::Poll {
                instance: self.handle,
            })
            .unwrap();
    }

    pub fn register_event_listener(&self, listener: Box<dyn ActionListener>) -> u64 {
        let mut listeners = self.listeners.write();
        listeners.push(listener);
        listeners.len() as u64
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
}
