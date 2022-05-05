use std::collections::HashMap;

use crate::{interaction_profile::InteractionProfile, Path, Time, Vec2D};

#[derive(Debug, Clone, Copy)]
pub(crate) enum InputEvent {
    Button { state: bool, changed: bool },
    Analog { state: f32 },
    Joystick { state: Vec2D },
    Cursor { state: Vec2D },
}

pub(crate) enum ActionBindingKey {
    Path(Path),
    Handle(u64),
}

pub(crate) struct InputMapper {
    component_bindings: HashMap<Path, Box<dyn ComponentBindings>>,
    // activators: HashMap<ActionBindingKey, ()>,
    action_bindings: HashMap<ActionBindingKey, ()>,
}

pub(crate) trait ComponentBindings {
    fn handle_event(&self, time: Time, event: &InputEvent);

    fn tick(&self, time: Time);
}

pub(crate) struct ComponentBindingsImpl<T> {
    active_bindings: Vec<Box<dyn ComponentBinding<T>>>,
}

pub(crate) trait ComponentBinding<T> {
    fn base_priority(&self) -> u32;

    fn check_priority(&self, time: Time, event: &T) -> u32;

    fn tick(&self, time: Time, event: &T) -> u32;
}

impl InputMapper {
    pub fn new() -> Self {
        Self {
            component_bindings: HashMap::new(),
            action_bindings: HashMap::new(),
        }
    }

    pub fn tick<I: Iterator<Item = (Time, Path, InputEvent)>>(&self, input_events: I, time: Time) {
        for (time, path, event) in input_events {
            println!("Component:{path:?} had event `{event:?}` at {time:?}");

            if let Some(component_bindings) = self.component_bindings.get(&path) {
                component_bindings.handle_event(time, &event);
            }

            for component_bindings in self.component_bindings.values() {
                component_bindings.tick(time);
            }
        }
    }
}
