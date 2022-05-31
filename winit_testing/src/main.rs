use raw_window_handle::HasRawWindowHandle;
use suinput::{ActionEvent, ActionListener, ActionType, SimpleBinding, SuAction};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

struct Listener {
    action1: SuAction,
    action2: SuAction,
}

impl ActionListener for Listener {
    fn handle_event(&self, event: ActionEvent, user: u64) {
        if event.action_handle == self.action1.handle() {
            println!("my_first_action -> {:?} for user {user}", event.data);
        } else if event.action_handle == self.action2.handle() {
            println!("my_second_action -> {:?} for user {user}", event.data);
        }
    }
}

fn main() -> Result<(), anyhow::Error> {
    let runtime = suinput::load_runtime();
    runtime.add_driver(windows_driver::Win32DesktopDriver::new)?;

    let instance = runtime.create_instance("Test Application");

    let action_set = instance.create_action_set("my_first_action_set", 0);
    let action1 = action_set.create_action("my_first_action", ActionType::Boolean);
    let action2 = action_set.create_action("my_second_action", ActionType::Delta2D);

    let desktop_profile = instance.get_path("/interaction_profiles/standard/desktop")?;

    let binding_layout = instance.create_binding_layout(
        "default_mouse_and_keyboard",
        desktop_profile,
        &[
            SimpleBinding {
                action: action1.handle(),
                path: instance.get_path("/user/desktop/mouse/input/button_left/click")?,
            },
            SimpleBinding {
                action: action1.handle(),
                path: instance.get_path("/user/desktop/keyboard/input/button_a/click")?,
            },
            SimpleBinding {
                action: action2.handle(),
                path: instance.get_path("/user/desktop/mouse/input/move/move2d")?,
            },
        ],
    )?;

    instance.set_default_binding_layout(desktop_profile, &binding_layout);

    let session = instance.create_session();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop)?;

    session.set_window_rwh(window.raw_window_handle());

    session.register_event_listener(Box::new(Listener {
        action1: action1.clone(),
        action2: action2.clone(),
    }));

    session.poll();

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
