use super::*;
use crate::application_instance::SuApplicationInstance;
use crate::{Inner, SuActionSet, SuBindingLayout};

use suinput_core::action_set::ActionSet;
use suinput_core::instance::BindingLayout;
pub use suinput_core::types::app::ApplicationInfo;
use suinput_core::types::app::InternalApplicationInstanceCreateInfo;
pub use suinput_types::binding::SimpleBinding;
pub use suinput_types::CreateBindingLayoutError;

pub struct ApplicationInstanceCreateInfo<'a> {
    pub application_info: &'a ApplicationInfo<'a>,
    pub sub_name: Option<&'a str>,
    pub action_sets: &'a [&'a SuActionSet],
    pub binding_layouts: &'a [&'a SuBindingLayout],
}

#[derive(Clone)]
pub struct SuInstance(pub(crate) Inner<suinput_core::instance::Instance>);

impl SuInstance {
    pub fn get_path(&self, path_string: &str) -> Result<SuPath, PathFormatError> {
        match &self.0 {
            Inner::Embedded(inner) => inner.get_path(path_string),
            Inner::FFI() => todo!(),
        }
    }

    pub fn create_action_set(&self, name: &str, default_priority: u32) -> SuActionSet {
        SuActionSet(match &self.0 {
            Inner::Embedded(inner) => {
                Inner::Embedded(inner.create_action_set(name.into(), default_priority))
            }
            Inner::FFI() => todo!(),
        })
    }

    pub fn create_binding_layout(
        &self,
        name: &str,
        interaction_profile: SuPath,
        bindings: &[SimpleBinding],
    ) -> Result<SuBindingLayout, CreateBindingLayoutError> {
        Ok(SuBindingLayout(match &self.0 {
            Inner::Embedded(inner) => {
                Inner::Embedded(inner.create_binding_layout(name, interaction_profile, bindings)?)
            }
            Inner::FFI() => todo!(),
        }))
    }

    pub fn create_application_instance(
        &self,
        create_info: &ApplicationInstanceCreateInfo,
    ) -> SuApplicationInstance {
        SuApplicationInstance(match &self.0 {
            Inner::Embedded(inner) => {
                let action_sets: Option<Vec<&Arc<ActionSet>>> =
                    create_info.action_sets.iter().map(|i| i.0.get()).collect();

                let binding_layouts: Option<Vec<Arc<BindingLayout>>> =
                    create_info.binding_layouts.iter().map(|i| i.0.get().cloned()).collect();

                let internal_create_info = InternalApplicationInstanceCreateInfo {
                    application: create_info.application_info,
                    sub_name: create_info.sub_name,
                    action_sets: &action_sets.unwrap()[..],
                    binding_layouts: binding_layouts.unwrap(),
                };
                Inner::Embedded(inner.create_application_instance(internal_create_info))
            }
            Inner::FFI() => todo!(),
        })
    }

    pub fn acquire_application_instance(
        &self,
        persistent_unique_id: u128,
    ) -> SuApplicationInstance {
        todo!()
    }
}
