use dashmap::{mapref::one::Ref, DashMap};
use suinput_types::SuPath;

use super::{
    device_types::DeviceTypes, interaction_profile_type::InteractionProfileType,
    paths::InteractionProfilePath,
};

//TODO load interaction profiles from Drivers and KDL files
pub struct InteractionProfileTypes {
    cache: DashMap<InteractionProfilePath, InteractionProfileType>,
}

impl InteractionProfileTypes {
    pub fn new<F: Fn(&str) -> SuPath>(device_types: &DeviceTypes, get_path: F) -> Self {
        let cache = DashMap::new();

        let desktop = InteractionProfileType::new_desktop_profile(device_types, &get_path);
        cache.insert(desktop.id, desktop);

        let dualsense = InteractionProfileType::new_dualsense_profile(device_types, &get_path);
        cache.insert(dualsense.id, dualsense);

        Self { cache }
    }

    pub fn get(
        &self,
        path: InteractionProfilePath,
    ) -> Option<Ref<'_, InteractionProfilePath, InteractionProfileType>> {
        self.cache.get(&path)
    }
}
