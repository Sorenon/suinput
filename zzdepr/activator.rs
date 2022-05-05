use std::{collections::VecDeque, ops::Deref};

use crate::Time;

struct Cycle {
    press_time: Time,
    release_time: Option<Time>,
}

struct ButtonActivatorSystem {
    cycle_buffer: VecDeque<Cycle>,
    activators: Vec<Box<dyn Activator>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ActivatorType {
    Hold,    //can't be blocking
    Press,   //can't be blocking or blockable
    Release, //can't be blocking or blockable
    LongHold,
    SingleTap,
    DoubleTap,
    TripleTap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ActivatorBlockPriority {
    // TripleTap, //canceled if DoubleTap fails or maxHoldDur passes before 2nd release or maxReleasedDur passes before 3rd press
    // DoubleTap, //canceled if SingleTap fails or maxHoldDur passes before 1st release or maxReleasedDur passes before 2nd press
    TripleTap, //canceled if maxTripleTapDur passes before third press
    DoubleTap,  //canceled if maxDoubleTapDur passes before second press
    LongHold,   //canceled if button released before minHoldDur
    SingleTap,  //canceled if button still held after maxHoldDur
    Hold,
}

impl ButtonActivatorSystem {
    fn on_event(&mut self, state: bool, time: Time) {
        if state {
            #[cfg(debug_assertions)]
            if let Some(last_cycle) = self.cycle_buffer.back() {
                debug_assert!(last_cycle.release_time.is_some())
            }
            self.cycle_buffer.push_back(Cycle {
                press_time: time,
                release_time: None,
            })

            //TODO fire press
        } else {
            if let Some(last_cycle) = self.cycle_buffer.back_mut() {
                debug_assert!(last_cycle.release_time.is_none());
                last_cycle.release_time = Some(time);
            }

            //TODO fire release
        }
    }

    fn tick(&mut self, time: Time) {
        //TODO trigger long press
    }
}

enum RunTest {
    MightRun,
    CanRun,
    CannotRun,
}

trait Activator {
    fn block_priority(&self) -> ActivatorBlockPriority;

    fn test_event(&self, cycles: &[Cycle]) -> RunTest;
    fn test_tick(&self, cycles: &[Cycle], time: Time) -> RunTest;

    fn run(&self, cycles: &[Cycle]);
}

struct TripleTapActivator {
    max_duration: Time,
    active: bool,
}

impl Activator for TripleTapActivator {
    fn block_priority(&self) -> ActivatorBlockPriority {
        ActivatorBlockPriority::TripleTap
    }

    fn test_tick(&self, cycles: &[Cycle], time: Time) -> RunTest {
        debug_assert_ne!(cycles.len(), 0);

        if cycles[0].press_time.0 + self.max_duration.0 < time.0 {
            self.test_event(cycles)
        } else {
            RunTest::CannotRun
        }
    }

    fn test_event(&self, cycles: &[Cycle]) -> RunTest {
        debug_assert_ne!(cycles.len(), 0);

        if cycles.len() == 1 {
            return RunTest::MightRun;
        } else if cycles.len() == 2 {
            let last_event = cycles[1].release_time.unwrap_or(cycles[1].press_time);
            if cycles[0].press_time.0 + self.max_duration.0 < last_event.0 {
                return RunTest::MightRun;
            }
        } else {
            if cycles[0].press_time.0 + self.max_duration.0 < cycles[2].press_time.0 {
                return RunTest::CanRun;
            }
        }

        return RunTest::CannotRun;
    }

    fn run(&self, cycles: &[Cycle]) {
        let cycle = &cycles[3];
    }
}
