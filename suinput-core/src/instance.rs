use std::{
    path::{Path, PathBuf},
    sync::{Arc, Weak},
};

use crate::{
    application_instance::ApplicationInstance,
    internal::{paths::InteractionProfilePath, types::HashMap},
    types::app::InternalApplicationInstanceCreateInfo,
};

use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use slotmap::{DefaultKey, SlotMap};
use suinput_types::{
    binding::SimpleBinding, event::PathFormatError, CreateBindingLayoutError, SuPath,
};

use crate::{
    action::Action,
    internal::binding::binding_engine::processed_binding_layout::ProcessedBindingLayout,
    session::Session,
};

use super::{action_set::ActionSet, runtime::Runtime};

pub struct Instance {
    pub handle: u64,
    pub(crate) runtime: Weak<Runtime>,

    pub(crate) storage_path: Option<PathBuf>,

    //TODO replace these with generational arenas
    pub(crate) action_sets: RwLock<Vec<Arc<ActionSet>>>,
    pub(crate) actions: RwLock<Vec<Arc<Action>>>,

    pub(crate) application_instances: RwLock<SlotMap<DefaultKey, Arc<ApplicationInstance>>>,

    pub(crate) sessions: RwLock<Vec<Arc<Session>>>,

    pub(crate) default_binding_layouts: RwLock<HashMap<SuPath, Arc<BindingLayout>>>,
}

impl Instance {
    pub fn new(runtime: &Arc<Runtime>, handle: u64, storage_path: Option<&Path>) -> Self {
        Instance {
            handle,
            runtime: Arc::downgrade(runtime),
            storage_path: storage_path.map(|path| path.to_owned()),
            actions: RwLock::default(),
            application_instances: Default::default(),
            sessions: RwLock::default(),
            default_binding_layouts: RwLock::default(),
            action_sets: RwLock::default(),
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
        let mut action_sets = self.action_sets.write();

        let arc = Arc::new(ActionSet {
            handle: action_sets.len() as u64,
            instance: Arc::downgrade(self),
            name,
            default_priority,
            actions: Default::default(),
            baked_actions: OnceCell::new(),
        });

        action_sets.push(arc.clone());

        arc
    }

    pub fn create_localization(&self, _identifier: String) {
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
                    processed_cache: processed,
                    bindings: bindings.to_vec(),
                })
            })
            .map_err(|err| {
                log::error!("SuInput: create_binding_layout failed with {err}");
                err
            })
    }

    pub fn create_application_instance(
        self: &Arc<Self>,
        create_info: InternalApplicationInstanceCreateInfo,
    ) -> Arc<ApplicationInstance> {
        let action_sets = create_info
            .action_sets
            .iter()
            .map(|action_set| {
                action_set
                    .baked_actions
                    .get_or_init(|| action_set.actions.read().clone());
                (action_set.handle, (*action_set).clone())
            })
            .collect::<HashMap<_, _>>();

        let actions = action_sets
            .values()
            .flat_map(|action_set| {
                action_set
                    .baked_actions
                    .get()
                    .unwrap()
                    .iter()
                    .map(|action| (action.handle, action.clone()))
            })
            .collect();

        let mut application_instances = self.application_instances.write();

        let index = application_instances.insert_with_key(|index| {
            Arc::new(ApplicationInstance {
                runtime: self.runtime.clone(),
                instance: Arc::downgrade(self),
                index,
                application_name: create_info.application.name.to_string(),
                sub_name: create_info.sub_name.map(|s| s.to_string()),
                action_sets,
                actions,
                binding_layouts: create_info.binding_layouts,
                session: RwLock::new(None),
            })
        });

        application_instances.get(index).unwrap().clone()
    }
}

pub struct BindingLayout {
    pub name: String,
    pub interaction_profile: InteractionProfilePath,
    pub bindings: Vec<SimpleBinding>,

    pub processed_cache: ProcessedBindingLayout,
}
