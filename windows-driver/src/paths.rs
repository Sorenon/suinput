use suinput::{event::DriverManager, Path};

pub struct Paths {
    pub mouse: Path,
    pub keyboard: Path,
    pub mouse_move: Path,
    pub mouse_scroll: Path,
    pub mouse_right_click: Path,
    pub mouse_left_click: Path,
    pub mouse_middle_click: Path,
    pub mouse_button4_click: Path,
    pub mouse_button5_click: Path,
}

impl Paths {
    pub fn new(driver_manager: &mut dyn DriverManager) -> Self {
        //TODO create well defined Path system
        Self {
            mouse: driver_manager.get_path("/device/standard/generic_mouse"),
            keyboard: driver_manager.get_path("/device/standard/hid_keyboard"),
            mouse_move: driver_manager.get_path("/input/move/move2d"),
            mouse_scroll: driver_manager.get_path("/input/scroll/move2d"),
            mouse_right_click: driver_manager.get_path("/input/button_right/click"),
            mouse_left_click: driver_manager.get_path("/input/button_left/click"),
            mouse_middle_click: driver_manager.get_path("/input/button_middle/click"),
            mouse_button4_click: driver_manager.get_path("/input/button_4/click"),
            mouse_button5_click: driver_manager.get_path("/input/button_5/click"),
        }
    }
}
