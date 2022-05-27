use std::{collections::HashMap, sync::Arc};

use suinput_types::SuPath;

use crate::{
    binding_engine::{ActionStateEnum, ProcessedBindingLayout},
    instance::BindingLayout,
};

#[derive(Default)]
pub(crate) struct User {
    pub default_binding_layout: HashMap<SuPath, Arc<BindingLayout>>,

    //should there also be a way to remove binding layouts?
    pub new_binding_layouts: HashMap<SuPath, ProcessedBindingLayout>,
    pub action_states: HashMap<u64, ActionStateEnum>,
}
