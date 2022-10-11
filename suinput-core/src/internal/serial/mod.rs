use std::num::NonZeroU128;

use serde::{Deserialize, Serialize};

use crate::action::ActionTypeEnum;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ParentActionType {
    StickyBool,
    Axis1d,
    Axis2d,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Action<'a> {
    pub name: &'a str,
    pub data_type: ActionTypeEnum,
    pub parent_type: Option<ParentActionType>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionSet<'a> {
    pub name: &'a str,
    pub default_priority: u32,
    pub parent: Option<&'a str>,
    pub actions: Vec<Action<'a>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BindingLayout<'a> {
    pub name: &'a str,
    pub interaction_profile: &'a str,
    pub bindings: Vec<Binding>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Binding {
    Simple {
        parent_action: Option<String>,
        action: String,
        input_component: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationInstance<'a> {
    pub name: &'a str,
    pub sub_name: Option<&'a str>,
    pub unique_id: NonZeroU128,
    pub action_sets: Vec<ActionSet<'a>>,
    pub dynamic_action_sets: Vec<ActionSet<'a>>,
    pub binding_layouts: Vec<BindingLayout<'a>>,
}
