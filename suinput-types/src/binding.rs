use crate::SuPath;

#[derive(Debug, Clone, Copy)]
pub struct SimpleBinding {
    pub action: u64,
    pub path: SuPath,
}

#[derive(Debug, Clone)]
pub struct BooleanBinding {
    pub input: SuPath,

    pub activator: activator::Activator,

    pub output: u64,
}

mod activator {
    #[derive(Debug, Clone, Copy)]
    pub enum OverriddenBehavior {
        Block,
        Interrupt,
        None,
    }

    #[derive(Debug, Clone, Copy)]
    pub enum ImpulseOverriddenBehavior {
        Block,
        None,
    }

    #[derive(Debug, Clone, Copy)]
    pub enum Impulse {
        False,
        OnPress,
        OnRelease,
    }

    #[derive(Debug, Clone, Copy)]
    pub enum Activator {
        Hold {
            overridden_behavior: OverriddenBehavior,
            impulse: Impulse,
        },
        QuickTap {
            overridden_behavior: ImpulseOverriddenBehavior,
            blocking: bool,
            max_hold_duration: u32,
        },
        MultiTap {
            overridden_behavior: OverriddenBehavior,
            blocking: bool,
            duration: u32,
            impulse: Impulse,
            taps: u32,
        },
        LongHold {
            min_hold_duration: u32,
            impulse: Impulse,
        },
    }
}

mod gyro {
    ///Defines conversion of angular velocity to delta2d
    ///Y Axis is always controlled by pitch
    pub enum GyroSpace {
        ///Recommended for handheld devices
        LocalSpace {
            x_axis: XAxis,
        },
        ///Recommended for handheld devices
        LocalCombinedYawRoll {
            yaw_factor: f32,
            roll_factor: f32,
        },
        ///Recommended for controllers
        PlayerSpace {
            //default 60° for Yaw
            //default 45° for Roll
            relax_angle: f32,
            x_axis: XAxis,
        },
        WorldSpace {
            x_axis: XAxis,
        },
    }

    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub enum XAxis {
        Yaw,
        Roll,
    }
}

mod chord {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub enum ChordType {
        PressToEnable,
        PressToDisable,
        Toggle,
    }
}
