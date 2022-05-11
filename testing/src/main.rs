use bevy::prelude::*;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use runtime::SuInputRuntime;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(initialize_runtime)
        .add_system(check_window_changed)
        .run();
}

fn initialize_runtime(mut commands: Commands) {
    let mut embedded_runtime = SuInputRuntime::new();

    embedded_runtime
        .add_driver(|runtime_interface| {
            windows_driver::Win32DesktopDriver::initialize(runtime_interface, true, true)
        })
        .unwrap();

    commands.insert_resource(embedded_runtime);
}

fn check_window_changed(windows: Res<Windows>, mut runtime: ResMut<SuInputRuntime>) {
    if windows.is_changed() {
        let handles: Vec<usize> = windows
            .iter()
            .filter_map(|window| unsafe {
                match window.raw_window_handle().get_handle().raw_window_handle() {
                    RawWindowHandle::Win32(f) => Some(f.hwnd as usize),
                    _ => None,
                }
            })
            .collect();
        runtime.set_windows(&handles);
    }
}

// pub fn exit_on_window_close_system(
//     mut app_exit_events: EventWriter<AppExit>,
//     mut window_close_requested_events: EventReader<WindowCloseRequested>,
// ) {
//     if window_close_requested_events.iter().next().is_some() {
//         app_exit_events.send(AppExit);
//     }
// }
