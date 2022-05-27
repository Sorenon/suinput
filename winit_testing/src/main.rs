use suinput::{ActionEvent, ActionListener, SimpleBinding, ActionType, SuAction};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::windows::WindowExtWindows,
    window::WindowBuilder,
};

struct Listener {
    action1: SuAction,
    action2: SuAction,
}

impl ActionListener for Listener {
    fn handle_event(&self, event: ActionEvent) {
        if event.action_handle == self.action1.handle() {
            println!("my_first_action -> {:?}", event.data);
        } else if event.action_handle == self.action2.handle() {
            println!("my_second_action -> {:?}", event.data);
        }
    }
}

fn main() -> Result<(), anyhow::Error> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop)?;

    let runtime = suinput::load_runtime();
    runtime.add_driver(windows_driver::Win32DesktopDriver::new)?;
    runtime.set_windows(&[window.hwnd() as _]);

    let instance = runtime.create_instance("Test Instance");

    let action_set = instance.create_action_set("my_first_action_set", 0);
    let action1 = action_set.create_action("my_first_action", ActionType::Boolean);
    let action2 = action_set.create_action("my_second_action", ActionType::Delta2D);

    let mouse_click = instance.get_path("/user/desktop/mouse/input/button_left/click")?;
    let a_key = instance.get_path("/user/desktop/keyboard/input/button_a/click")?;
    let mouse_move = instance.get_path("/user/desktop/mouse/input/move/move2d")?;
    let desktop = instance.get_path("/interaction_profile/standard/desktop")?;

    instance.register_event_listener(Box::new(Listener {
        action1: action1.clone(),
        action2: action2.clone(),
    }));

    let binding_layout = instance.create_binding_layout(
        "default_mouse_and_keyboard",
        desktop,
        &[
            SimpleBinding {
                action: action1.handle(),
                path: mouse_click,
            },
            SimpleBinding {
                action: action1.handle(),
                path: a_key,
            },
            SimpleBinding {
                action: action2.handle(),
                path: mouse_move,
            },
        ],
    );

    instance.set_default_binding_layout(desktop, &binding_layout);

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
