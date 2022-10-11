use crate::{session::SuSession, Inner};
use std::fmt::{Debug, Formatter};
use std::num::NonZeroU128;
use std::path::Path;
use suinput_core::application_instance::ApplicationInstance;

#[derive(Clone)]
pub struct SuApplicationInstance(
    pub(crate) Inner<suinput_core::application_instance::ApplicationInstance>,
);

impl SuApplicationInstance {
    pub fn try_begin_session(&self) -> SuSession {
        SuSession(match &self.0 {
            Inner::Embedded(inner) => Inner::Embedded(inner.create_session()),
            Inner::FFI() => todo!(),
        })
    }

    /// `storage_path` should be a path to an empty / nonexistent file
    pub fn make_persistent(&self, file_path: &Path) -> crate::Result<()> {
        match &self.0 {
            Inner::Embedded(inner) => inner.make_persistent(file_path),
            Inner::FFI() => todo!(),
        }
    }

    pub fn get_persistent_unique_id(&self) -> Option<NonZeroU128> {
        todo!()
    }

    pub fn delete_application_instance(&self) {
        todo!()
    }
}
