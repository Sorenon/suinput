use std::num::NonZeroUsize;

use binding::SimpleBinding;
use thiserror::Error;

pub mod action;
pub mod binding;
pub mod controller_paths;
pub mod driver_interface;
pub mod event;
pub mod keyboard;

/**
 * Instead of using static enum parameters, SuInput often uses Path variables
 *
 * Types of Path:
 *
 * /interaction_profiles/<vendor_name>/<type_name>
 *
 * /devices/<vendor_name>/<name>
 *
 * /inputs/<source>[_<position]/<component>
 *
 * /outputs/<source>[_<position>]/<component>
 *
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SuPath(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Time(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActionHandle(u64);

#[derive(Debug, Clone, Error)]
pub enum CreateBindingLayoutError {
    #[error("Invalid Path Handle `{0:X?}`")]
    InvalidPathHandle(SuPath),
    #[error("Invalid Action Handle `{0:X?}`")]
    InvalidActionHandle(u64),
    #[error("Bad Interaction Profile Path `{0}`")]
    BadInteractionProfilePath(String),
    #[error("Bad Component Path `{0}`")]
    BadComponentPath(String),
    #[error("Bad User Path `{0}`")]
    BadUserPath(String),
    #[error("Bad Binding `{0:?}`")]
    BadBinding(SimpleBinding),
}

pub type WindowHandle = NonZeroUsize;
