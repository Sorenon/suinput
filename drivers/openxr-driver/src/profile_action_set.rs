use std::collections::HashMap;

use crate::{
    cvt,
    profile_serial::{self, InteractionProfileContent, OpenXRInteractionProfile, Subpath},
    xr_api::{create_action, create_action_set},
};
use openxr::{
    sys::{Action, ActionSet, ActionSuggestedBinding, InteractionProfileSuggestedBinding},
    Instance, Path,
};

// #[test]
// #[allow(dead_code)]
// fn test() {
//     let entry = unsafe { Entry::load() }.unwrap();

//     let exts = ExtensionSet::default();

//     let instance = entry
//         .create_instance(
//             &ApplicationInfo {
//                 application_name: "SuInput Headless",
//                 application_version: 1,
//                 engine_name: "ye",
//                 engine_version: 1,
//             },
//             &exts,
//             &[],
//         )
//         .unwrap();

//     let _driver = OpenXRDriver::new(instance);
// }

//TODO investigate: collapse into one action set and instead use bindings + xrGetCurrentInteractionProfile
//This could be more efficient and will avoid reaching the action limit

pub(crate) struct ProfileActionSet {
    pub(crate) instance: Instance,
    pub(crate) action_set: ActionSet,
    pub(crate) actions: HashMap<Path, (Action, bool)>,
}

impl ProfileActionSet {
    pub fn new(
        instance: &Instance,
        profile_id: &String,
        profile: &OpenXRInteractionProfile,
    ) -> Option<Self> {
        if profile.extension.is_some() {
            //TODO check if instance supports extension
            return None;
        }

        let (user_paths, sub_paths) = match &profile.content {
            InteractionProfileContent::Some {
                user_paths,
                sub_paths,
            } => (user_paths, sub_paths),
            InteractionProfileContent::Parent { parent: _ } => {
                //TODO create bindings to parent profile
                return None;
            }
        };

        let mut profile_action_set = Self {
            action_set: create_action_set(
                instance,
                &profile_id[22..].replace('/', "-"),
                &profile.localized_name,
            ),
            instance: instance.clone(),
            actions: HashMap::new(),
        };

        profile_action_set.create_profile_actions(user_paths, sub_paths);

        let interaction_profile_path = instance.string_to_path(profile_id).unwrap();
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
                    interaction_profile: interaction_profile_path,
                    count_suggested_bindings: suggested_bindings.len() as u32,
                    suggested_bindings: suggested_bindings.as_ptr(),
                },
            )
        }) {
            eprint!("{:?}", profile_id);
            for binding in &suggested_bindings {
                eprint!("{:?}", instance.path_to_string(binding.binding));
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
                profile_serial::Component::Position | profile_serial::Component::Haptic => {
                    format!("{user_path}{identifier}")
                }
                profile_serial::Component::Pose => {
                    //TODO support pose actions
                    return;
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
