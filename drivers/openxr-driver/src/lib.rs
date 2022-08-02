use std::{os::raw::c_char, ffi::CStr};

use openxr::{
    sys::{
        ActionSet, ActionSetCreateInfo, SessionCreateInfo, MAX_ACTION_SET_NAME_SIZE,
        MAX_LOCALIZED_ACTION_SET_NAME_SIZE,
    },
    ApplicationInfo, Entry, ExtensionSet, FormFactor, Instance, SessionCreateFlags, Version,
};

mod profiles;

#[test]
fn test() {
    // OpenXRDriver::new_headless();
}

//TODO XR_EXT_palm_pose
struct OpenXRDriver {
    entry: Entry,

    instance: openxr::Instance,
    session: openxr::sys::Session,
}

impl OpenXRDriver {
    pub fn new(entry: Entry, instance: Instance, session: openxr::sys::Session) {
        let profiles = profiles::get_profiles();

        for (profile_id, profile) in profiles.profiles {
            let action_set = create_action_set(&instance, &profile_id, &profile.localized_name);
        }
    }

    fn create_actions(instance: Instance, session: openxr::sys::Session) {}
}

fn create_action_set(instance: &Instance, name: &str, localized_name: &str) -> ActionSet {
    let mut action_set = ActionSet::NULL;
    cvt(unsafe {
        let mut action_set_name: [c_char; MAX_ACTION_SET_NAME_SIZE] = std::mem::zeroed();
        let mut localized_action_set_name: [c_char; MAX_LOCALIZED_ACTION_SET_NAME_SIZE] =
            std::mem::zeroed();
        place_cstr(&mut action_set_name, name);
        place_cstr(&mut localized_action_set_name, localized_name);

        (instance.fp().create_action_set)(
            instance.as_raw(),
            &ActionSetCreateInfo {
                ty: ActionSetCreateInfo::TYPE,
                next: std::ptr::null(),
                action_set_name,
                localized_action_set_name,
                priority: 0,
            },
            &mut action_set,
        )
    })
    .unwrap();
    action_set
}

// FFI helpers
fn cvt(x: openxr::sys::Result) -> openxr::Result<openxr::sys::Result> {
    if x.into_raw() >= 0 {
        Ok(x)
    } else {
        Err(x)
    }
}

fn place_cstr(out: &mut [c_char], s: &str) {
    if s.len() + 1 > out.len() {
        panic!(
            "string requires {} > {} bytes (including trailing null)",
            s.len(),
            out.len()
        );
    }
    for (i, o) in s.bytes().zip(out.iter_mut()) {
        *o = i as c_char;
    }
    out[s.len()] = 0;
}

#[test]
fn do_the_thing() {
    let entry = Entry::load().unwrap();

    let exts = ExtensionSet::default();

    let instance = entry
        .create_instance(
            &ApplicationInfo {
                application_name: "SuInput Headless",
                application_version: 1,
                engine_name: "ye",
                engine_version: 1,
            },
            &exts,
            &[],
        )
        .unwrap();

    let profiles = profiles::get_profiles();

    for (profile_id, profile) in profiles.profiles {
        if let Some(ext) = profile.extension {
            entry.enumerate_extensions().unwrap().
        }

        let action_set = create_action_set(&instance, &profile_id.replace('/', "-"), &profile.localized_name);
    }
}
