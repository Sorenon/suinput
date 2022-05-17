use runtime_api::SuInputRuntime;

pub fn load_runtime() -> SuInputRuntime {
    SuInputRuntime::new_tmp()
}