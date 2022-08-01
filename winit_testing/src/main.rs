use raw_window_handle::HasRawWindowHandle;
use suinput::{
    action_type::{
        Axis1d, Axis1dActionCreateInfo, Axis2d, Axis2dActionCreateInfo, BooleanActionCreateInfo,
        Cursor, Delta2d,
    },
    ActionEvent, ActionEventEnum, ActionListener, ChildActionType, SimpleBinding, SuAction,
    SuSession,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

struct Listener {
    session: SuSession,
    jump: SuAction<bool>,
    zoom: SuAction<bool>,
    // turn: SuAction,
    cursor: SuAction<Cursor>,
    thrust: SuAction<Axis1d>,
    r#move: SuAction<Axis2d>,
    overridden: SuAction<bool>,
    priority_action: SuAction<bool>,
}

impl ActionListener for Listener {
    fn handle_event(&mut self, event: ActionEvent, user: u64) {
        assert_eq!(user, 0);

        if event.action_handle == self.jump.handle() {
            if let ActionEventEnum::Boolean { state, changed } = event.data {
                if state && changed {
                    println!("jump!");
                    //Unstick zoom if the player jumps so the player has to manually zoom in again
                    self.session.unstick_bool_action(&self.zoom);
                }
            }
        } else if event.action_handle == self.zoom.handle() {
            println!("zoom {:?}", event.data)
        } else if event.action_handle == self.thrust.handle() {
            if let ActionEventEnum::Axis1d { state } = event.data {
                println!("thrust {state}");
            }
        } else if event.action_handle == self.r#move.handle() {
            if let ActionEventEnum::Axis2d { state } = event.data {
                println!("move {state:?}");
            }
        } else if event.action_handle == self.overridden.handle() {
            println!("This should not fire")
        } else if event.action_handle == self.priority_action.handle() {
            println!("Overridden")
        }
    }
}

fn main() -> Result<(), anyhow::Error> {
    let runtime = suinput::load_runtime();
    runtime.set_window_driver(windows_driver::Win32HookingWindowDriver::new)?;
    runtime
        .add_generic_driver(|interface| {
            sdl2_driver::SDLGameControllerGenericDriver::new(false, interface)
        })
        .unwrap();
    runtime.add_generic_driver(windows_driver::Win32RawInputGenericDriver::new)?;

    let instance = runtime.create_instance("Test Application");

    let action_set = instance.create_action_set("gameplay", 0);
    let priority_action_set = instance.create_action_set("higher_priority", 1);
    let jump_action = action_set.create_action("jump", BooleanActionCreateInfo::default());
    let zoom_action = action_set.create_action("zoom", BooleanActionCreateInfo { sticky: true });
    let turn_action = action_set.create_action("turn", ());
    let cursor_action = action_set.create_action("cursor", ());
    let thrust_action = action_set.create_action(
        "thrust",
        Axis1dActionCreateInfo {
            positive: Some("forward_thrust".into()),
            negative: Some("backward_thrust".into()),
        },
    );
    let move_action = action_set.create_action(
        "move",
        Axis2dActionCreateInfo {
            up: Some("move_forward".into()),
            down: Some("move_back".into()),
            left: Some("move_left".into()),
            right: Some("move_right".into()),
            vertical: Some("move_forward_and_back".into()),
            horizontal: Some("move_sideways".into()),
        },
    );

    let overridden = action_set.create_action("overriden", BooleanActionCreateInfo::default());
    let priority_action =
        priority_action_set.create_action("priority_action", BooleanActionCreateInfo::default());

    let desktop_profile = instance.get_path("/interaction_profiles/standard/desktop")?;

    let binding_layout = instance.create_binding_layout(
        "default_mouse_and_keyboard",
        desktop_profile,
        &[
            SimpleBinding {
                action: jump_action.handle(),
                path: instance.get_path("/user/desktop/keyboard/input/button_space/click")?,
            },
            SimpleBinding {
                action: jump_action.handle(),
                path: instance.get_path("/user/desktop/keyboard/input/button_v/click")?,
            },
            //Zoom in if the user holds right click
            SimpleBinding {
                action: zoom_action.handle(),
                path: instance.get_path("/user/desktop/mouse/input/button_right/click")?,
            },
            //Toggle zoom in if the user middle clicks
            SimpleBinding {
                action: zoom_action.get_child_action(ChildActionType::StickyToggle),
                path: instance.get_path("/user/desktop/mouse/input/button_middle/click")?,
            },
            //End toggle if the user right clicks
            SimpleBinding {
                action: zoom_action.get_child_action(ChildActionType::StickyRelease),
                path: instance.get_path("/user/desktop/mouse/input/button_right/click")?,
            },
            SimpleBinding {
                action: turn_action.handle(),
                path: instance.get_path("/user/desktop/mouse/input/move/move2d")?,
            },
            SimpleBinding {
                action: cursor_action.handle(),
                path: instance.get_path("/user/desktop/cursor/input/cursor/point")?,
            },
            SimpleBinding {
                action: thrust_action.get_child_action(ChildActionType::Positive),
                path: instance.get_path("/user/desktop/keyboard/input/button_up/click")?,
            },
            SimpleBinding {
                action: thrust_action.get_child_action(ChildActionType::Negative),
                path: instance.get_path("/user/desktop/keyboard/input/button_down/click")?,
            },
            SimpleBinding {
                action: move_action.get_child_action(ChildActionType::Up),
                path: instance.get_path("/user/desktop/keyboard/input/button_w/click")?,
            },
            SimpleBinding {
                action: move_action.get_child_action(ChildActionType::Right),
                path: instance.get_path("/user/desktop/keyboard/input/button_d/click")?,
            },
            SimpleBinding {
                action: move_action.get_child_action(ChildActionType::Down),
                path: instance.get_path("/user/desktop/keyboard/input/button_s/click")?,
            },
            SimpleBinding {
                action: move_action.get_child_action(ChildActionType::Left),
                path: instance.get_path("/user/desktop/keyboard/input/button_a/click")?,
            },
            SimpleBinding {
                action: overridden.handle(),
                path: instance.get_path("/user/desktop/keyboard/input/button_t/click")?,
            },
            SimpleBinding {
                action: priority_action.handle(),
                path: instance.get_path("/user/desktop/keyboard/input/button_t/click")?,
            },
        ],
    )?;

    instance.set_default_binding_layout(desktop_profile, &binding_layout);

    let dualsense_profile = instance.get_path("/interaction_profiles/sony/dualsense")?;

    let binding_layout = instance.create_binding_layout(
        "default_dualsense",
        dualsense_profile,
        &[
            SimpleBinding {
                action: jump_action.handle(),
                path: instance.get_path("/user/gamepad/input/diamond_down/click")?,
            },
            SimpleBinding {
                action: zoom_action.handle(),
                path: instance.get_path("/user/gamepad/input/trigger_left/value")?,
            },
            SimpleBinding {
                action: thrust_action.get_child_action(ChildActionType::Positive),
                path: instance.get_path("/user/gamepad/input/trigger_right/value")?,
            },
            SimpleBinding {
                action: thrust_action.get_child_action(ChildActionType::Negative),
                path: instance.get_path("/user/gamepad/input/shoulder_right/click")?,
            },
            SimpleBinding {
                action: move_action.handle(),
                path: instance.get_path("/user/gamepad/input/joystick_left/position")?,
            },
            SimpleBinding {
                action: turn_action.handle(),
                path: instance.get_path("/user/gamepad/input/motion/gyro")?,
            },
        ],
    )?;

    instance.set_default_binding_layout(dualsense_profile, &binding_layout);

    let session = instance.create_session(&[&action_set, &priority_action_set]);

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop)?;

    session.set_window_rwh(window.raw_window_handle());

    session.register_event_listener(Box::new(Listener {
        jump: jump_action,
        zoom: zoom_action,
        // turn: turn_action.clone(),
        cursor: cursor_action,
        thrust: thrust_action,
        session: session.clone(),
        r#move: move_action,
        overridden,
        priority_action,
    }));

    let mut yaw = 0.;
    let mut pitch = 0.;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => {
                *control_flow = ControlFlow::Exit;
                runtime.destroy();
            }
            Event::MainEventsCleared => {
                std::thread::sleep(std::time::Duration::from_millis(16));

                session.poll();

                let delta = session.get_action_state::<Delta2d>(&turn_action).unwrap();
                if delta.x != 0. || delta.y != 0. {
                    yaw += delta.x;
                    pitch += delta.y;
                    println!("{yaw} {pitch}")
                }
            }
            _ => (),
        }
    });
}
