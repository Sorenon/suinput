use crate::SuPath;

#[derive(Debug, Clone, Copy)]
pub struct SimpleBinding {
    pub action: u64,
    pub path: SuPath,
}
