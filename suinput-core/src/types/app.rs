use std::sync::Arc;

use crate::{action_set::ActionSet, instance::BindingLayout};

pub struct ApplicationInfo<'a> {
    pub name: &'a str,
}

pub struct InternalApplicationInstanceCreateInfo<'a> {
    pub application: &'a ApplicationInfo<'a>,
    pub sub_name: Option<&'a str>,
    pub action_sets: &'a [&'a Arc<ActionSet>],
    pub binding_layouts: Vec<Arc<BindingLayout>>,
}
