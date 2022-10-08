use std::fs::File;
use std::num::NonZeroU128;
use std::path::{Path, PathBuf};
use crate::{Inner, SuSession};

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

    pub fn make_persistent(&self) -> NonZeroU128 {
        todo!()
    }

    pub fn get_persistent_unique_id(&self) -> Option<NonZeroU128> {
        todo!()
    }

    pub fn delete_application_instance(&self) {
        todo!()
    }
}
