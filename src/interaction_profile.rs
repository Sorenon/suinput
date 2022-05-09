use crate::SuPath;

pub(crate) trait InteractionProfile {
    fn get_parent_component(&self, component: SuPath) -> Option<SuPath>;
}