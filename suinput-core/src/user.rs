use std::{collections::HashMap, sync::Arc};

use dashmap::DashMap;
use suinput_types::SuPath;

use crate::{instance::BindingLayout, binding_engine::{ProcessedBindingLayout, ActionStateEnum}};

#[derive(Default)]
pub(crate) struct User {
    pub default_binding_layout: HashMap<SuPath, Arc<BindingLayout>>,

    //TODO put this on the input thread
    pub binding_layouts: DashMap<SuPath, ProcessedBindingLayout>,
    pub action_states: DashMap<u64, ActionStateEnum>,
}
