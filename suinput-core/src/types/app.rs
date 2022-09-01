use std::{path::Path, sync::Arc};

use crate::{action_set::ActionSet, instance::BindingLayout};

pub struct ApplicationInfo<'a> {
    pub name: &'a str,
}

pub struct InternalApplicationInstanceCreateInfo<'a> {
    pub application: &'a ApplicationInfo<'a>,
    pub sub_name: Option<&'a str>,
    pub action_sets: &'a [&'a Arc<ActionSet>],
    pub binding_layouts: Vec<Arc<BindingLayout>>,
    // pub rules: &'a [(&'a ApplicationInstanceRules<'a>, AppInstanceRuleResponse)]
}

pub enum AppInstanceRuleResponse {
    Warn,
    DeleteAfter { days: u32 },
}

pub enum ApplicationInstanceRules<'a> {
    FileDoesntContain(&'a Path, &'a FileContainsRule<'a>),
}

pub enum FileContainsRule<'a> {
    UID,
    Text(&'a str),
    Bytes(&'static [u8]),
}
