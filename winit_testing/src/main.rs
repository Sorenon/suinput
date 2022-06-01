use raw_window_handle::HasRawWindowHandle;
use suinput::{ActionEvent, ActionEventEnum, ActionListener, ActionType, SimpleBinding, SuAction};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

struct Listener {
    jump: SuAction,
    turn: SuAction,
    cursor: SuAction,
}

impl ActionListener for Listener {
    fn handle_event(&self, event: ActionEvent, user: u64) {
        if event.action_handle == self.jump.handle() {
            if let ActionEventEnum::Boolean { state, changed } = event.data {
                if state && changed {
                    println!("jump! for user {user}");
                }
            }
            println!("{:?}", event.data)
        } else if event.action_handle == self.turn.handle() {
            if let ActionEventEnum::Delta2D { delta } = event.data {
                println!("turn {delta:?} for user {user}");
            }
        } else if event.action_handle == self.cursor.handle() {
            if let ActionEventEnum::Cursor { normalized_window_coords } = event.data {
                let x = normalized_window_coords.0;
                let y = normalized_window_coords.1;
                if x <= 1. && x >= 0. && y <= 1. && y >= 0. {
                    println!("cursor moved to {normalized_window_coords:?} for user {user}");
                }
            }
        }
    }
}

fn main() -> Result<(), anyhow::Error> {
    let runtime = suinput::load_runtime();
    runtime.add_driver(windows_driver::Win32DesktopDriver::new)?;

    let instance = runtime.create_instance("Test Application");

    let action_set = instance.create_action_set("gameplay", 0);
    let jump_action = action_set.create_action("jump", ActionType::Boolean);
    let turn_action = action_set.create_action("turn", ActionType::Delta2D);
    let cursor_action = action_set.create_action("cursor", ActionType::Cursor);

    let desktop_profile = instance.get_path("/interaction_profiles/standard/desktop")?;

    let binding_layout = instance.create_binding_layout(
        "default_mouse_and_keyboard",
        desktop_profile,
        &[
            SimpleBinding {
                action: jump_action.handle(),
                path: instance.get_path("/user/desktop/mouse/input/button_left/click")?,
            },
            SimpleBinding {
                action: jump_action.handle(),
                path: instance.get_path("/user/desktop/keyboard/input/button_space/click")?,
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

    let session = instance.create_session();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop)?;

    session.set_window_rwh(window.raw_window_handle());

    session.register_event_listener(Box::new(Listener {
        jump: jump_action.clone(),
        turn: turn_action.clone(),
        cursor: cursor_action.clone(),
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
