use std::collections::HashMap;

use crate::{SuPath, Time};

#[derive(Debug, Default)]
pub struct PathManager(HashMap<String, SuPath>, HashMap<SuPath, String>);

#[derive(Debug)]
pub enum PathFormatError {
    MissingLeadingSlash,
    IllegalEndingSlash,
    IllegalCharacter,
    DoubleSlash,
    PeriodOnlyWord,
}

impl PathManager {
    pub fn get_path(&mut self, path_string: &str) -> Result<SuPath, PathFormatError> {
        if let Some(&path) = self.0.get(path_string) {
            return Ok(path);
        }

        let mut words = path_string.split('/');
        if !words.next().unwrap().is_empty() {
            return Err(PathFormatError::MissingLeadingSlash);
        }

        for word in &mut words {
            if word.is_empty() {
                if words.next().is_none() {
                    return Err(PathFormatError::IllegalEndingSlash);
                }
                return Err(PathFormatError::DoubleSlash);
            }
            if word.contains(|c| {
                if let 'a'..='z' | '0'..='9' | '.' | '-' | '_' = c {
                    false
                } else {
                    true
                }
            }) {
                return Err(PathFormatError::IllegalCharacter);
            }
            if !word.contains(|c| c != '.') {
                return Err(PathFormatError::PeriodOnlyWord);
            }
        }

        let path = SuPath(self.0.len() as u32);
        self.0.insert(path_string.to_owned(), path);
        self.1.insert(path, path_string.to_owned());
        Ok(path)
    }

    pub fn get_path_string(&self, path: SuPath) -> Option<String> {
        self.1.get(&path).map(|inner| inner.clone())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct InputEvent {
    pub device: u64,
    pub path: SuPath,
    pub time: Time,
    pub data: InputComponentEvent,
}

#[derive(Debug, Clone, Copy)]
pub enum InputComponentEvent {
    Button(ButtonEvent),
    Move2D(Move2D),
    Cursor(Cursor)
}

#[derive(Debug, Clone, Copy)]
pub enum ButtonEvent {
    Press,
    Release,
}

#[derive(Debug, Clone, Copy)]
pub struct Move2D {
    pub value: (f64, f64),
}

#[derive(Debug, Clone, Copy)]
pub struct Cursor {
    pub normalized_screen_coords: (f64, f64),
}
