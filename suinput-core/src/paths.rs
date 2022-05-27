use suinput_types::SuPath;

pub struct CommonPaths {
    pub mouse: SuPath,
    pub keyboard: SuPath,
    pub system_cursor: SuPath,
    pub cursor_point: SuPath,
    pub mouse_move: SuPath,
    pub mouse_scroll: SuPath,
    pub mouse_right_click: SuPath,
    pub mouse_left_click: SuPath,
    pub mouse_middle_click: SuPath,
    pub mouse_button4_click: SuPath,
    pub mouse_button5_click: SuPath,
}

impl CommonPaths {
    pub fn new<F: Fn(&str) -> SuPath>(get_path: F) -> Self {
        Self {
            mouse: get_path("/device/standard/generic_mouse"),
            keyboard: get_path("/device/standard/hid_keyboard"),
            system_cursor: get_path("/device/standard/system_cursor"),
            cursor_point: get_path("/input/cursor/point"),
            mouse_move: get_path("/input/move/move2d"),
            mouse_scroll: get_path("/input/scroll/move2d"),
            mouse_right_click: get_path("/input/button_right/click"),
            mouse_left_click: get_path("/input/button_left/click"),
            mouse_middle_click: get_path("/input/button_middle/click"),
            mouse_button4_click: get_path("/input/button_4/click"),
            mouse_button5_click: get_path("/input/button_5/click"),
        }
    }
}
