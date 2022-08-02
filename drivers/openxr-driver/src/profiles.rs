use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenXRInteractionProfiles {
    pub profiles: HashMap<String, OpenXRInteractionProfile>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenXRInteractionProfile {
    pub localized_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension: Option<String>,
    #[serde(flatten)]
    pub content: InteractionProfileContent,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Subpath {
    pub localized_name: String,
    pub r#type: SubpathType,
    #[serde(default)]
    pub optional: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_path: Option<String>,
    pub components: Vec<Component>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Component {
    ///Button
    Click,
    ///Button
    Touch,
    ///Value
    Force,
    ///Value
    Value,
    ///Axis2D
    Position,
    ///Axis1D
    Twist,

    ///XrPose
    Pose,

    ///TODO
    Haptic,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum SubpathType {
    Button,
    Pose,
    Vibration,
    Trackpad,
    Trigger,
    Joystick,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum InteractionProfileContent {
    Some {
        user_paths: Vec<String>,
        sub_paths: HashMap<String, Subpath>,
    },
    Parent {
        parent: String,
    }
}

static PROFILES: &str = include_str!("openxr_interaction_profiles.json");

pub fn get_profiles() -> OpenXRInteractionProfiles {
    serde_json::from_str(PROFILES).unwrap()
}
