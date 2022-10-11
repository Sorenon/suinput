use std::fmt::{Display, Formatter};

pub mod action_type;
pub mod app;

pub type Result<T> = core::result::Result<T, Error>;

//TODO come up with better Error propagation
#[derive(thiserror::Error, Debug)]
pub enum Error {
    RuntimeFailure = -0x1,
    ValidationFailure = -0x2,
    IoError = -0x3,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
