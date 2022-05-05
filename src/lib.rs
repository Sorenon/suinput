pub mod input_mapper;
pub mod interaction_profile;
pub mod event;
pub mod event2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Path(u32);

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