use nalgebra::Vector2;
use suinput_types::action::ActionStateEnum;

use super::input_component::InputComponentState;

/*
    Needed for

    Device aggregation in interaction profiles
    Binding output aggregation in working user
    Binding Layout aggregation in working user
*/
pub trait InputEventSources {
    type Index: Copy;

    type SourceIndex: Copy + PartialEq;

    //Please GATs soon
    type Sources: Iterator<Item = Self::SourceIndex>;

    fn get_state<I: InputEventType>(&self, idx: Self::Index) -> Option<I::Value>;

    fn get_source_state<I: InputEventType>(
        &self,
        idx: Self::Index,
        source_idx: Self::SourceIndex,
    ) -> Option<I::Value>;

    fn get_sources<I: InputEventType>(&self, idx: Self::Index) -> Self::Sources;

    fn aggregate<I: InputEventType>(
        &self,
        idx: Self::Index,
        event: I::Value,
        source: Self::SourceIndex,
    ) -> Option<I::EventOut> {
        I::aggregate(
            event,
            self.get_state::<I>(idx).unwrap_or_default(),
            self.get_sources::<I>(idx)
                .filter(|source_idx| *source_idx != source)
                .filter_map(|source_idx| self.get_source_state::<I>(idx, source_idx)),
        )
    }
}

pub trait InputEventType: Sized {
    type Value: 'static + Default;
    type EventOut: 'static + Default;

    fn from_ase(ase: &ActionStateEnum) -> Self::Value;
    fn from_ics(ics: &InputComponentState) -> Self::Value;
    fn aggregate<'a>(
        event_state: Self::Value,
        prev_state: Self::Value,
        iter: impl Iterator<Item = Self::Value>,
    ) -> Option<Self::EventOut>;
}

impl InputEventType for bool {
    type Value = Self;
    type EventOut = (Self, Self);

    fn from_ase(ase: &ActionStateEnum) -> bool {
        match ase {
            ActionStateEnum::Boolean(bool) => *bool,
            _ => panic!(),
        }
    }

    fn aggregate<'a>(
        event_state: bool,
        prev_state: bool,
        mut others: impl Iterator<Item = Self::Value>,
    ) -> Option<Self::EventOut> {
        if event_state {
            Some((true, !prev_state))
        } else {
            if others.find(|state| *state).is_some() {
                None
            } else {
                Some((false, prev_state))
            }
        }
    }

    fn from_ics(ics: &InputComponentState) -> Self::Value {
        match ics {
            InputComponentState::Button(bool) => *bool,
            _ => panic!(),
        }
    }
}

impl InputEventType for crate::types::action_type::Value {
    type Value = f32;
    type EventOut = f32;

    fn from_ase(ase: &ActionStateEnum) -> Self::Value {
        match ase {
            ActionStateEnum::Value(value) => *value,
            _ => panic!(),
        }
    }

    fn aggregate<'a>(
        event_state: f32,
        prev_state: Self::Value,
        mut iter: impl Iterator<Item = Self::Value>,
    ) -> Option<Self::EventOut> {
        if event_state > prev_state {
            Some(event_state)
        } else {
            if iter
                .find(|other_state| *other_state >= event_state)
                .is_some()
            {
                None
            } else {
                Some(event_state)
            }
        }
    }

    fn from_ics(ics: &InputComponentState) -> Self::Value {
        match ics {
            InputComponentState::Trigger(val) => *val,
            _ => panic!(),
        }
    }
}

impl InputEventType for crate::types::action_type::Axis1d {
    type Value = f32;
    type EventOut = f32;

    fn from_ase(ase: &ActionStateEnum) -> Self::Value {
        match ase {
            ActionStateEnum::Axis1d(value) => *value,
            _ => panic!(),
        }
    }

    fn aggregate<'a>(
        event_state: f32,
        prev_state: Self::Value,
        mut iter: impl Iterator<Item = Self::Value>,
    ) -> Option<Self::EventOut> {
        let abs = event_state.abs();
        if abs > prev_state.abs() {
            Some(event_state)
        } else {
            if iter.find(|other_state| other_state.abs() >= abs).is_some() {
                None
            } else {
                Some(event_state)
            }
        }
    }

    fn from_ics(_: &InputComponentState) -> Self::Value {
        todo!()
    }
}

impl InputEventType for crate::types::action_type::Axis2d {
    type Value = Vector2<f32>;

    type EventOut = Vector2<f32>;

    fn from_ase(ase: &ActionStateEnum) -> Self::Value {
        match ase {
            ActionStateEnum::Axis2d(state) => (*state).into(),
            _ => panic!(),
        }
    }

    fn from_ics(ics: &InputComponentState) -> Self::Value {
        match ics {
            InputComponentState::Joystick(state) => *state,
            _ => panic!(),
        }
    }

    fn aggregate<'a>(
        event_state: Self::Value,
        prev_state: Self::Value,
        mut iter: impl Iterator<Item = Self::Value>,
    ) -> Option<Self::EventOut> {
        let lensq = event_state.magnitude_squared();
        if lensq > prev_state.magnitude_squared() {
            Some(event_state)
        } else {
            if iter
                .find(|other_state| other_state.magnitude_squared() >= lensq)
                .is_some()
            {
                None
            } else {
                Some(event_state)
            }
        }
    }
}
