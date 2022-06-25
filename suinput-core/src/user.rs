use parking_lot::{Mutex, RwLock};
use suinput_types::{action::ActionStateEnum, SuPath};

use crate::internal::binding::binding_engine::ProcessedBindingLayout;
use crate::internal::types::HashMap;

#[derive(Default)]
pub struct User {
    pub action_states: RwLock<HashMap<u64, ActionStateEnum>>,
    //should there also be a way to remove binding layouts?
    pub new_binding_layouts: Mutex<HashMap<SuPath, ProcessedBindingLayout>>,
}
