use raw_window_handle::HasRawWindowHandle;
use suinput::{
    ActionCreateInfo, ActionEvent, ActionEventEnum, ActionListener, ChildActionType, SimpleBinding,
    SuAction, SuSession,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

struct Listener {
    session: SuSession,
    jump: SuAction,
    zoom: SuAction,
    turn: SuAction,
    cursor: SuAction,
}

impl ActionListener for Listener {
    fn handle_event(&self, event: ActionEvent, user: u64) {
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
        } else if event.action_handle == self.turn.handle() {
            if let ActionEventEnum::Delta2D { delta } = event.data {
                println!("turn {delta:?}");
            }
        } else if event.action_handle == self.cursor.handle() {
            if let ActionEventEnum::Cursor {
                normalized_window_coords,
            } = event.data
            {
                let x = normalized_window_coords.0;
                let y = normalized_window_coords.1;
                if x <= 1. && x >= 0. && y <= 1. && y >= 0. {
                    // println!("cursor moved to {normalized_window_coords:?}");
                }
            }
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
    let jump_action = action_set.create_action("jump", ActionCreateInfo::Boolean { sticky: false });
    let zoom_action = action_set.create_action("zoom", ActionCreateInfo::Boolean { sticky: true });
    let turn_action = action_set.create_action("turn", ActionCreateInfo::Delta2D);
    let cursor_action = action_set.create_action("cursor", ActionCreateInfo::Cursor);

    let desktop_profile = instance.get_path("/interaction_profiles/standard/desktop")?;

    let binding_layout = instance.create_binding_layout(
        "default_mouse_and_keyboard",
        desktop_profile,
        &[
            SimpleBinding {
                action: jump_action.handle(),
                path: instance.get_path("/user/desktop/keyboard/input/button_space/click")?,
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
        ],
    )?;

    instance.set_default_binding_layout(desktop_profile, &binding_layout);

    let test = instance.get_path("/interaction_profiles/sony/dualsense")?;

    let binding_layout = instance.create_binding_layout(
        "dualsense_test",
        test,
        &[SimpleBinding {
            action: jump_action.handle(),
            path: instance.get_path("/user/gamepad/input/shoulder_left/click")?,
        }],
    )?;

    instance.set_default_binding_layout(test, &binding_layout);

    let session = instance.create_session(&[&action_set]);

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop)?;

    session.set_window_rwh(window.raw_window_handle());

    session.register_event_listener(Box::new(Listener {
        jump: jump_action.clone(),
        zoom: zoom_action.clone(),
        turn: turn_action.clone(),
        cursor: cursor_action.clone(),
        session: session.clone(),
    }));

    // session.poll();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => {
                *control_flow = ControlFlow::Exit;
                runtime.destroy();
            }
            _ => (),
        }
    });
}
