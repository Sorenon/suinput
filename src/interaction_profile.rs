use crate::Path;

pub(crate) trait InteractionProfile {
    fn get_parent_component(&self, component: Path) -> Option<Path>;
}