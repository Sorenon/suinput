use std::{
    sync::{Arc, Weak},
};

use parking_lot::RwLock;
use suinput_types::{event::PathFormatError, SuPath, action::ActionListener};

use crate::{action::Action, internal::{user::User, binding::binding_engine::ProcessedBindingLayout}};

use super::{action_set::ActionSet, runtime::Runtime};

pub struct Instance {
    pub(crate) runtime: Weak<Runtime>,

    name: String,

    // action_sets: RwLock<Vec<Arc<ActionSet>>>,
    pub(crate) user: RwLock<User>,
    pub(crate) listeners: RwLock<Vec<Box<dyn ActionListener>>>,

    //TODO should this be a generational arena or is that overkill?
    pub(crate) actions: RwLock<Vec<Arc<Action>>>,
}

impl Instance {
    pub fn new(runtime: &Arc<Runtime>, name: String, persistent_unique_id: ()) -> Self {
        Instance {
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

        self.runtime
            .upgrade()
            .unwrap()
            .driver2runtime_sender
            .send(crate::internal::worker_thread::WorkerThreadEvent::Poll)
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

// #[derive(Default)]
// pub(crate) struct Player {
//     pub default_binding_layout: HashMap<SuPath, Arc<BindingLayout>>,
//     pub active_binding_layout: HashMap<SuPath, ProcessedBindingLayout>,
// }

pub struct BindingLayout {
    pub name: String,
    pub interaction_profile: SuPath,
    pub bindings: Vec<SimpleBinding>,
}

#[derive(Debug, Clone, Copy)]
pub struct SimpleBinding {
    pub action: u64,
    pub path: SuPath,
}

// pub(crate) struct ProcessedBindingLayout {
//     pub bindings: HashMap<SuPath, HashMap<SuPath, Vec<u64>>>,
// }

// impl ProcessedBindingLayout {
//     fn new(
//         instance: &Instance,
//         interaction_profile: SuPath,
//         binding_layout: &BindingLayout,
//     ) -> Self {
//         if interaction_profile != binding_layout.interaction_profile {
//             todo!("Binding Layout conversion not yet implemented")
//         }

//         let mut bindings = HashMap::<SuPath, HashMap<SuPath, Vec<u64>>>::new();

//         for binding in &binding_layout.bindings {
//             let path_string = instance.get_path_string(binding.path).unwrap();

//             let split_idx = path_string.find("/input").expect("Invalid path string");
//             let (user_str, component_str) = path_string.split_at(split_idx);

//             let user_path = instance.get_path(user_str).unwrap();
//             let component_path = instance.get_path(component_str).unwrap();

//             if !bindings.contains_key(&user_path) {
//                 bindings.insert(user_path, HashMap::new());
//             }

//             let component_paths = bindings.get_mut(&user_path).unwrap();

//             if !component_paths.contains_key(&component_path) {
//                 component_paths.insert(component_path, Vec::with_capacity(1));
//             }

//             component_paths
//                 .get_mut(&component_path)
//                 .unwrap()
//                 .push(binding.action)
//         }

//         Self { bindings }
//     }
// }
