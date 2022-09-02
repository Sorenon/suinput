use std::{collections::HashMap, os::raw::c_char};

use openxr::{
    sys::{Session, SessionActionSetsAttachInfo},
    Instance,
};
use parking_lot::RwLock;
use profile_action_set::ProfileActionSet;
use suinput::{SuSession, SuInputRuntime};

mod profile_action_set;
mod profile_serial;
mod xr_api;

//TODO XR_EXT_palm_pose
pub struct OpenXRDriver {
    instance: openxr::Instance,

    profile_action_sets: HashMap<String, ProfileActionSet>,

    sessions: RwLock<Vec<Session>>,
}

// /**
//  * Creates all the needed actions on the xr_instance and attaches them to the xr_session
//  * If SuInput does not provide a needed action type users can pass action sets containing that type to be attached
//  *
//  * Returns true if call was successful
//  *
//  * Returns false if xrAttachSessionActionSets had already been called for the specified session and the OpenXR layer
//  * is not installed. In this case the Instance will have to rely on the developer provided OpenXR fallback driver.
//  * This will occur on most pre-existing game engines and will may require altering the engine's OpenXR plugin.
//  */
// pub fn bind_openxr(
//     &self,
//     _xr_instance: (),
//     _xr_session: (),
//     _extra_xr_action_sets: Option<()>,
// ) -> bool {
//     todo!()
// }

impl OpenXRDriver {
    pub fn new(instance: Instance) -> Self {
        Self {
            profile_action_sets: profile_serial::get_profiles()
                .profiles
                .iter()
                .filter_map(|(profile_name, profile)| {
                    ProfileActionSet::new(&instance, profile_name, profile)
                        .map(|set| (profile_name.clone(), set))
                })
                .collect(),
            instance,
            sessions: RwLock::new(Vec::new()),
        }
    }

    pub fn bind_session(
        &self,
        session: &SuSession,
        xr_session: Session,
        extra_action_sets: &[openxr::sys::ActionSet],
    ) {
        self.add_session(xr_session, extra_action_sets);
    }

    fn add_session(&self, xr_session: Session, extra_action_sets: &[openxr::sys::ActionSet]) {
        let mut sessions = self.sessions.write();
        if sessions.contains(&xr_session) {
            return;
        }
        sessions.push(xr_session);
        std::mem::drop(sessions);

        let action_sets = self
            .profile_action_sets
            .values()
            .map(|set| set.action_set)
            .chain(extra_action_sets.iter().copied())
            .collect::<Vec<_>>();

        cvt(unsafe {
            (self.instance.fp().attach_session_action_sets)(
                xr_session,
                &SessionActionSetsAttachInfo {
                    ty: SessionActionSetsAttachInfo::TYPE,
                    next: std::ptr::null(),
                    count_action_sets: action_sets.len() as u32,
                    action_sets: action_sets.as_ptr(),
                },
            )
        })
        .unwrap();
    }
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
