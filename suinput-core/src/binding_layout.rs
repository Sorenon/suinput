use std::collections::HashMap;

use suinput_types::SuPath;

pub struct BindingLayoutState {
    direct_bindings: HashMap<(SuPath, SuPath), u64>
}