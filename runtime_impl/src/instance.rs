use std::sync::{Arc, Weak};

use parking_lot::RwLock;

use super::{action_set::ActionSet, runtime::Runtime};

pub struct Instance {
    runtime: Weak<Runtime>,

    name: String,
    // action_sets: RwLock<Vec<Arc<ActionSet>>>,
}

impl Instance {
    pub fn new(runtime: &Arc<Runtime>, name: String, persistent_unique_id: ()) -> Self {
        Instance {
            runtime: Arc::downgrade(&runtime),
            name,
            // action_sets: Default::default(),
        }
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

    pub fn create_binding_layout(&self, name: String, layout: ()) {}

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
