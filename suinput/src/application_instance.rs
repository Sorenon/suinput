use crate::{Inner, SuSession};

#[derive(Clone)]
pub struct SuApplicationInstance(
    pub(crate) Inner<suinput_core::application_instance::ApplicationInstance>,
);

impl SuApplicationInstance {
    pub fn get_persistent_unique_id(&self) -> u128 {
        todo!()
    }

    pub fn try_begin_session(&self) -> SuSession {
        SuSession(match &self.0 {
            Inner::Embedded(inner) => Inner::Embedded(inner.create_session()),
            Inner::FFI() => todo!(),
        })
    }
}
