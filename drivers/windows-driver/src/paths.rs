use suinput::driver_interface::RuntimeInterfaceTrait;
use suinput_types::SuPath;

pub struct CommonPaths {
    pub mouse: SuPath,
    pub keyboard: SuPath,
    pub mouse_move: SuPath,
    pub mouse_scroll: SuPath,
    pub mouse_right_click: SuPath,
    pub mouse_left_click: SuPath,
    pub mouse_middle_click: SuPath,
    pub mouse_button4_click: SuPath,
    pub mouse_button5_click: SuPath,
}

impl CommonPaths {
    pub fn new(driver_manager: &dyn RuntimeInterfaceTrait) -> Self {
        Self {
            mouse: driver_manager
                .get_path("/devices/standard/generic_mouse")
                .unwrap(),
            keyboard: driver_manager
                .get_path("/devices/standard/hid_keyboard")
                .unwrap(),
            mouse_move: driver_manager.get_path("/input/move/move2d").unwrap(),
            mouse_scroll: driver_manager.get_path("/input/scroll/move2d").unwrap(),
            mouse_right_click: driver_manager
                .get_path("/input/button_right/click")
                .unwrap(),
            mouse_left_click: driver_manager.get_path("/input/button_left/click").unwrap(),
            mouse_middle_click: driver_manager
                .get_path("/input/button_middle/click")
                .unwrap(),
            mouse_button4_click: driver_manager.get_path("/input/button_4/click").unwrap(),
            mouse_button5_click: driver_manager.get_path("/input/button_5/click").unwrap(),
        }
    }
}
