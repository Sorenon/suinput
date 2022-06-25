use crate::{internal::{input_events::InputEventType, types::HashMap}, types::action_type::{Value, Axis1d, Axis2d}};

use nalgebra::Vector2;
use suinput_types::{
    action::ActionEventEnum,
};

use super::working_user::ActionStates;

pub enum ParentActionState {
    StickyBool {
        combined_state: bool,
        stuck: bool,

        press: u64,
        release: u64,
        toggle: u64,
    },
    Axis1d {
        combined_state: f32,

        positive: u64,
        negative: u64,
    },
    Axis2d {
        combined_state: Vector2<f32>,

        up: u64,
        down: u64,
        left: u64,
        right: u64,
        vertical: u64,
        horizontal: u64,
    },
}

//TODO make this truly event based to allow for repeat presses
pub fn handle_sticky_bool_event(
    parent: u64,
    parent_action_states: &mut HashMap<u64, ParentActionState>,
    action_states: &ActionStates,
) -> Option<ActionEventEnum> {
    if let Some(ParentActionState::StickyBool {
        combined_state,
        stuck,
        press,
        release,
        toggle,
    }) = parent_action_states.get_mut(&parent)
    {
        let parent = action_states.get_bool(parent).unwrap_or_default();
        let toggle = action_states.get_bool(*toggle).unwrap_or_default();
        let release = action_states.get_bool(*release).unwrap_or_default();
        let press = action_states.get_bool(*press).unwrap_or_default();

        if toggle {
            *stuck = !*stuck;
        }

        if *stuck && release || press {
            *stuck = press;
        }

        let last_state = *combined_state;

        *combined_state = parent || *stuck;

        if last_state != *combined_state {
            Some(ActionEventEnum::Boolean {
                state: *combined_state,
                changed: true,
            })
        } else {
            None
        }
    } else {
        panic!()
    }
}

pub fn handle_axis1d_event(
    parent: u64,
    parent_action_states: &mut HashMap<u64, ParentActionState>,
    action_states: &ActionStates,
) -> Option<ActionEventEnum> {
    if let Some(ParentActionState::Axis1d {
        combined_state,
        positive,
        negative,
    }) = parent_action_states.get_mut(&parent)
    {
        let states = &action_states.states;
        let positive = states
            .get(&positive)
            .map(|v| Value::from_ase(v))
            .unwrap_or_default();
        let negative = states
            .get(&negative)
            .map(|v| Value::from_ase(v))
            .unwrap_or_default();
        let parent = states
            .get(&parent)
            .map(|v| Axis1d::from_ase(v))
            .unwrap_or_default();

        let new_combined_state = (positive - negative + parent).clamp(-1., 1.);
        if new_combined_state.abs() != combined_state.abs() {
            *combined_state = new_combined_state;

            Some(ActionEventEnum::Axis1d {
                state: new_combined_state,
            })
        } else {
            None
        }
    } else {
        panic!()
    }
}

pub fn handle_axis2d_event(
    parent: u64,
    parent_action_states: &mut HashMap<u64, ParentActionState>,
    action_states: &ActionStates,
) -> Option<ActionEventEnum> {
    if let Some(ParentActionState::Axis2d {
        combined_state,
        up,
        down,
        left,
        right,
        vertical,
        horizontal,
    }) = parent_action_states.get_mut(&parent)
    {
        let states = &action_states.states;
        let parent = states
            .get(&parent)
            .map(|v| Axis2d::from_ase(v))
            .unwrap_or_default();

        let up = states
            .get(&up)
            .map(|v| Value::from_ase(v))
            .unwrap_or_default();
        let down = states
            .get(&down)
            .map(|v| Value::from_ase(v))
            .unwrap_or_default();
        let left = states
            .get(&left)
            .map(|v| Value::from_ase(v))
            .unwrap_or_default();
        let right = states
            .get(&right)
            .map(|v| Value::from_ase(v))
            .unwrap_or_default();

        let horizontal = states
            .get(&horizontal)
            .map(|v| Axis1d::from_ase(v))
            .unwrap_or_default();
        let vertical = states
            .get(&vertical)
            .map(|v| Axis1d::from_ase(v))
            .unwrap_or_default();

        let new_combined_state = Vector2::new(
            (right - left + horizontal + parent.x).clamp(-1., 1.),
            (up - down + vertical + parent.y).clamp(-1., 1.),
        );

        if new_combined_state != *combined_state {
            *combined_state = new_combined_state;

            Some(ActionEventEnum::Axis2d {
                state: new_combined_state.into(),
            })
        } else {
            None
        }
    } else {
        panic!()
    }
}
