use std::ops::Deref;

use dashmap::DashMap;
use parking_lot::RwLock;
use regex::Regex;
use suinput_types::{event::PathFormatError, SuPath};

//TODO investigate storing paths more efficiently
#[derive(Debug)]
pub struct PathManager(DashMap<String, SuPath>, RwLock<Vec<String>>, Regex);

impl PathManager {
    pub fn new() -> Self {
        let regex = Regex::new(r#"^(/(\.*[a-z0-9-_]+\.*)+)+$"#).unwrap();
        Self(DashMap::new(), RwLock::default(), regex)
    }

    pub fn get_path(&self, path_string: &str) -> Result<SuPath, PathFormatError> {
        if let Some(path) = self.0.get(path_string) {
            return Ok(*path.deref());
        }

        if self.2.is_match(path_string) {
            let mut paths = self.1.write();
            //Double check that the path has not been added now that we have exclusive access
            if let Some(path) = self.0.get(path_string) {
                return Ok(*path.deref());
            }

            let path = SuPath(self.0.len().try_into().unwrap());
            self.0.insert(path_string.to_owned(), path);
            paths.push(path_string.to_owned());
            Ok(path)
        } else {
            Err(PathFormatError)
        }
    }

    pub fn get_path_string(&self, path: SuPath) -> Option<String> {
        self.1.read().get(path.0 as usize).cloned()
    }
}

pub struct CommonPaths {
    pub desktop: InteractionProfilePath,
    pub mouse: DevicePath,
    pub keyboard: DevicePath,
    pub system_cursor: DevicePath,
    pub cursor_point: InputPath,
    pub mouse_move: InputPath,
    pub mouse_scroll: InputPath,
    pub mouse_right_click: InputPath,
    pub mouse_left_click: InputPath,
    pub mouse_middle_click: InputPath,
    pub mouse_button4_click: InputPath,
    pub mouse_button5_click: InputPath,
}

impl CommonPaths {
    pub fn new<F: Fn(&str) -> SuPath>(get_path: F) -> Self {
        Self {
            desktop: get_path("/interaction_profiles/standard/desktop"),
            mouse: get_path("/devices/standard/generic_mouse"),
            keyboard: get_path("/devices/standard/hid_keyboard"),
            system_cursor: get_path("/devices/standard/system_cursor"),
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

//TODO start using these everywhere and then migrate them to structs
pub type UserPath = SuPath;
pub type InputPath = SuPath;
pub type InteractionProfilePath = SuPath;
pub type DevicePath = SuPath;
