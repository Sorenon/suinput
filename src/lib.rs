pub mod event;
pub mod input_mapper;
pub mod interaction_profile;
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
 * /outputs/<{source>[_<position>]/<component>
 *
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Path(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Time(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActionHandle(u64);

#[derive(Debug, Clone, Copy)]
pub struct Vec2D {
    pub x: f32,
    pub y: f32,
}

fn main() {
    let mapper = input_mapper::InputMapper::new();

    mapper.tick(
        [
            (
                Time(0),
                Path(124),
                input_mapper::InputEvent::Button {
                    state: true,
                    changed: true,
                },
            ),
            (
                Time(1000),
                Path(124),
                input_mapper::InputEvent::Button {
                    state: false,
                    changed: true,
                },
            ),
        ]
        .into_iter(),
        Time(1100),
    );
}
