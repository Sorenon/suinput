use std::{collections::HashMap, fs::File, path::Path};

use serde::Deserialize;

pub fn generate(manifest_dir: &Path, workspace_dir: &Path) -> anyhow::Result<()> {
    let bindings_file = File::open(manifest_dir.join("monado/bindings.json")).unwrap();

    let profiles = serde_json::from_reader::<_, Profiles>(bindings_file).unwrap();

    for profile in profiles.profiles.keys() {
        println!("{}", profile)
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
struct Profiles {
    profiles: HashMap<String, InteractionProfile>,
}

#[derive(Debug, Deserialize)]
struct InteractionProfile {
    title: String,
    // r#type: String,
    // monado_device: String,
    subaction_paths: Vec<String>,
    subpaths: HashMap<String, Subpath>,
}

#[derive(Debug, Deserialize)]
struct Subpath {
    r#type: SubpathType,
    localized_name: String,
    components: Vec<Component>,
    // monado_bindings: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Component {
    Click,
    Touch,
    Force,
    Value,
    Position, // x, y
    Twist,
    Pose,

    Haptic,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SubpathType {
    Button,
    Pose,
    Vibration,
    Trackpad,
    Trigger,
    Joystick,
}
