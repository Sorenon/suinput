pub mod driver_interface;
pub mod event;
pub mod keyboard;
pub mod action;

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

#[derive(Debug, Clone, Copy)]
pub struct Vec2D {
    pub x: f32,
    pub y: f32,
}


pub enum SuInputResult {

}