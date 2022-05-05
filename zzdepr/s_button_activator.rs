use std::{collections::VecDeque, ops::Deref};

use crate::Time;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SButtonActivatorType {
    Hold,    //can't be blocking
    Press,   //can't be blocking or blockable
    Release, //can't be blocking or blockable
    LongHold,
    SingleTap,
    DoubleTap,
    TripleTap,
}

pub(crate) struct LongHoldActivator {
    min_hold_dir: Time,
    last_press_time: Option<Time>,
    active: bool,
}

impl LongHoldActivator {
    fn on_event(&mut self, pressed: bool, time: Time) {
        if pressed {
            self.last_press_time = Some(time);
        } else {
            self.last_press_time = None;
            self.active = false;
        }
    }

    fn tick(&mut self, time: Time) {
        if !self.active {
            if let Some(last_press_time) = self.last_press_time {
                if last_press_time.0 + self.min_hold_dir.0 > time.0 {
                    self.active = true;
                }
            }
        }
    }
}
