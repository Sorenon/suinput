use std::{
    collections::{HashMap, HashSet},
    os::raw::c_char,
};

use openxr::{
    sys::{
        Action, ActionCreateInfo, ActionSet, ActionSetCreateInfo, ActionSuggestedBinding,
        InteractionProfileSuggestedBinding, Session, SessionActionSetsAttachInfo,
        MAX_ACTION_NAME_SIZE, MAX_ACTION_SET_NAME_SIZE, MAX_LOCALIZED_ACTION_NAME_SIZE,
        MAX_LOCALIZED_ACTION_SET_NAME_SIZE,
    },
    ActionType, ApplicationInfo, Entry, ExtensionSet, Instance, Path,
};
use parking_lot::RwLock;
use profiles::{OpenXRInteractionProfile, Subpath};

mod profiles;

//TODO XR_EXT_palm_pose
pub struct OpenXRDriver {
    instance: openxr::Instance,

    profile_action_sets: HashMap<String, ProfileActionSet>,

    sessions: RwLock<Vec<Session>>,
}

impl OpenXRDriver {
    pub fn new(instance: Instance) -> Self {
        Self {
            profile_action_sets: profiles::get_profiles()
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

    pub fn add_session(&self, session: Session) {
        let mut sessions = self.sessions.write();
        if sessions.contains(&session) {
            return;
        }
        sessions.push(session);
        std::mem::drop(sessions);

        let action_sets = self
            .profile_action_sets
            .values()
            .map(|set| set.action_set)
            .collect::<Vec<_>>();

        cvt(unsafe {
            (self.instance.fp().attach_session_action_sets)(
                session,
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

#[test]
#[allow(dead_code)]
fn test() {
    let entry = unsafe {
        Entry::load()
    }.unwrap();

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

    let _driver = OpenXRDriver::new(instance);
}

struct ProfileActionSet {
    instance: Instance,
    action_set: ActionSet,
    actions: HashMap<Path, (Action, bool)>,
}

impl ProfileActionSet {
    fn new(
        instance: &Instance,
        profile_id: &String,
        profile: &OpenXRInteractionProfile,
    ) -> Option<Self> {
        if profile.extension.is_some() {
            return None;
        }

        let mut profile_action_set = Self {
            action_set: create_action_set(
                instance,
                &profile_id[22..].replace('/', "-"),
                &profile.localized_name,
            ),
            instance: instance.clone(),
            actions: HashMap::new(),
        };

        match &profile.content {
            profiles::InteractionProfileContent::Some {
                user_paths,
                sub_paths,
            } => {
                profile_action_set.create_profile_actions(user_paths, sub_paths);
            }
            profiles::InteractionProfileContent::Parent { parent: _ } => {
                return None;
            }
        }

        let interaction_profile = instance.string_to_path(profile_id).unwrap();
        let suggested_bindings = profile_action_set
            .actions
            .iter()
            .filter_map(|(&binding, &(action, optional))| {
                if optional {
                    None
                } else {
                    Some(ActionSuggestedBinding { action, binding })
                }
            })
            .collect::<Vec<_>>();

        if let Err(err) = cvt(unsafe {
            (instance.fp().suggest_interaction_profile_bindings)(
                instance.as_raw(),
                &InteractionProfileSuggestedBinding {
                    ty: InteractionProfileSuggestedBinding::TYPE,
                    next: std::ptr::null(),
                    interaction_profile,
                    count_suggested_bindings: suggested_bindings.len() as u32,
                    suggested_bindings: suggested_bindings.as_ptr(),
                },
            )
        }) {
            println!("{:?}", profile_id);
            for binding in &suggested_bindings {
                println!("{:?}", instance.path_to_string(binding.binding));
            }
            panic!("suggest_interaction_profile_bindings failed with {:?}", err)
        }

        Some(profile_action_set)
    }

    fn create_profile_actions(
        &mut self,
        user_paths: &Vec<String>,
        sub_paths: &HashMap<String, Subpath>,
    ) {
        for (identifier, sub_path) in sub_paths {
            match &sub_path.user_path {
                Some(user_path) => self.create_sub_path_actions(identifier, user_path, sub_path),
                None => {
                    for user_path in user_paths {
                        self.create_sub_path_actions(identifier, user_path, sub_path);
                    }
                }
            }
        }
    }

    fn create_sub_path_actions(&mut self, identifier: &str, user_path: &str, sub_path: &Subpath) {
        for component in &sub_path.components {
            let name = match component {
                profiles::Component::Position | profiles::Component::Haptic => {
                    format!("{user_path}{identifier}")
                }
                _ => format!("{user_path}{identifier}/{}", component.as_str()),
            };

            let path = self.instance.string_to_path(&name).unwrap();

            let action = create_action(
                &self.instance,
                self.action_set,
                &name[6..].replace('/', "-"),
                &name,
                component.ty(),
            );

            self.actions.insert(path, (action, sub_path.optional));
        }
    }
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

fn create_action(
    instance: &Instance,
    action_set: ActionSet,
    name: &str,
    localized_name: &str,
    action_type: ActionType,
) -> Action {
    let mut action = Action::NULL;
    cvt(unsafe {
        let mut action_name: [c_char; MAX_ACTION_NAME_SIZE] = std::mem::zeroed();
        let mut localized_action_name: [c_char; MAX_LOCALIZED_ACTION_NAME_SIZE] =
            std::mem::zeroed();
        place_cstr(&mut action_name, name);
        place_cstr(&mut localized_action_name, localized_name);

        (instance.fp().create_action)(
            action_set,
            &ActionCreateInfo {
                ty: ActionCreateInfo::TYPE,
                next: std::ptr::null(),
                action_name,
                action_type,
                count_subaction_paths: 0,
                subaction_paths: std::ptr::null(),
                localized_action_name,
            },
            &mut action,
        )
    })
    .unwrap();
    action
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
