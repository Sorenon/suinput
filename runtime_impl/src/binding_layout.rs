use std::collections::HashMap;

use suinput::SuPath;

pub struct BindingLayoutState {
    direct_bindings: HashMap<(SuPath, SuPath), u64>
}