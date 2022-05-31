use std::collections::{hash_map::Entry, HashMap};

use suinput_types::{
    action::{ActionStateEnum, ActionType},
    binding::SimpleBinding,
    event::{InputComponentEvent, InputEvent},
    CreateBindingLayoutError, SuPath,
};

use crate::{
    instance::{BindingLayout, Instance},
    internal::{
        input_component::InputComponentType,
        paths::{InputPath, InteractionProfilePath, UserPath},
    },
};

#[derive(Debug, Clone)]
pub struct ProcessedBindingLayout {
    pub(crate) bindings_index: Vec<(ProcessedBinding, ActionStateEnum, u64)>,

    input_bindings: HashMap<UserPath, HashMap<InputPath, Vec<usize>>>,
    pub(crate) bindings_for_action: HashMap<u64, Vec<usize>>,
}

impl ProcessedBindingLayout {
    pub fn new(
        instance: &Instance,
        interaction_profile: InteractionProfilePath,
        bindings: &Vec<SimpleBinding>,
    ) -> Result<Self, CreateBindingLayoutError> {
        let runtime = instance.runtime.upgrade().unwrap();
        let actions = instance.actions.read();

        let interaction_profile_type = runtime
            .interaction_profile_types
            .get(interaction_profile)
            .ok_or_else(|| {
            instance.get_path_string(interaction_profile).map_or(
                CreateBindingLayoutError::InvalidPathHandle(interaction_profile),
                |path| CreateBindingLayoutError::BadInteractionProfilePath(path),
            )
        })?;

        let mut bindings_index = Vec::<(ProcessedBinding, ActionStateEnum, u64)>::new();
        let mut input_bindings = HashMap::<SuPath, HashMap<SuPath, Vec<usize>>>::new();
        let mut bindings_for_action = HashMap::<u64, Vec<usize>>::new();

        for binding in bindings {
            let path_string = instance
                .get_path_string(binding.path)
                .ok_or(CreateBindingLayoutError::InvalidPathHandle(binding.path))?;

            let split_idx = path_string.find("/input").expect("Invalid path string");
            let (user_str, component_str) = path_string.split_at(split_idx);

            let user_path = instance.get_path(user_str).unwrap();

            // todo!("we need to do this through interaction_profile");
            let device = match interaction_profile_type.user2device.get(&user_path) {
                Some(device) => device,
                None => {
                    return Err(instance.get_path_string(user_path).map_or(
                        CreateBindingLayoutError::InvalidPathHandle(interaction_profile),
                        |path| CreateBindingLayoutError::BadUserPath(path),
                    ))
                }
            };

            let device = runtime.device_types.get(*device).unwrap();

            let component_path = instance.get_path(component_str).unwrap();

            if !input_bindings.contains_key(&user_path) {
                input_bindings.insert(user_path, HashMap::new());
            }

            let component_paths = input_bindings.get_mut(&user_path).unwrap();

            if !component_paths.contains_key(&component_path) {
                component_paths.insert(component_path, Vec::with_capacity(1));
            }

            let action = actions.get((binding.action as usize) - 1).ok_or(
                CreateBindingLayoutError::InvalidActionHandle(binding.action),
            )?;

            let processed_binding = match device.input_components.get(&component_path) {
                Some(InputComponentType::Button) => {
                    if action.action_type != ActionType::Boolean {
                        return Err(CreateBindingLayoutError::BadBinding(*binding));
                    }

                    ProcessedBinding::Button2Bool
                }
                Some(InputComponentType::Move2D) => {
                    if action.action_type != ActionType::Delta2D {
                        return Err(CreateBindingLayoutError::BadBinding(*binding));
                    }

                    ProcessedBinding::Move2d2Delta2d {
                        sensitivity: (1., 1.),
                    }
                }
                Some(InputComponentType::Cursor) => {
                    if action.action_type != ActionType::Cursor {
                        return Err(CreateBindingLayoutError::BadBinding(*binding));
                    }

                    ProcessedBinding::Cursor2Cursor
                }
                None => {
                    return Err(instance.get_path_string(component_path).map_or(
                        CreateBindingLayoutError::InvalidPathHandle(interaction_profile),
                        |path| CreateBindingLayoutError::BadComponentPath(path),
                    ))
                }
            };

            let action_state = match action.action_type {
                ActionType::Boolean => ActionStateEnum::Boolean(false),
                ActionType::Delta2D => ActionStateEnum::Delta2D((0., 0.)),
                ActionType::Cursor => ActionStateEnum::Cursor((0., 0.)),
            };

            bindings_index.push((processed_binding, action_state, action.handle));
            component_paths
                .get_mut(&component_path)
                .unwrap()
                .push(bindings_index.len() - 1);
            match bindings_for_action.entry(action.handle) {
                Entry::Occupied(mut vec) => vec.get_mut().push(bindings_index.len() - 1),
                Entry::Vacant(empty) => {
                    empty.insert(vec![bindings_index.len() - 1]);
                }
            }
        }

        Ok(Self {
            bindings_index,
            input_bindings,
            bindings_for_action,
        })
    }

    pub fn convert_from_binding_layout(
        _instance: &Instance,
        interaction_profile: InteractionProfilePath,
        binding_layout: &BindingLayout,
    ) -> Self {
        if interaction_profile != binding_layout.interaction_profile {
            todo!("Binding Layout conversion not yet implemented")
        }

        // Self::new(instance, interaction_profile, &binding_layout.bindings)
        binding_layout.processed.clone()
    }

    pub fn on_event<F>(&mut self, user_path: SuPath, event: &InputEvent, mut on_action_event: F)
    where
        F: FnMut(u64, usize, &ActionStateEnum, &ProcessedBindingLayout),
    {
        if let Some(component_bindings) = self.input_bindings.get(&user_path) {
            if let Some(bindings) = component_bindings.get(&event.path) {
                for &binding_index in bindings {
                    if let Some((new_binding_state, action_handle)) =
                        execute_binding(binding_index, &mut self.bindings_index, event)
                    {
                        on_action_event(action_handle, binding_index, &new_binding_state, &self)
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ProcessedBinding {
    Button2Bool,
    Move2d2Delta2d { sensitivity: (f64, f64) },
    Cursor2Cursor,
}

pub(crate) fn execute_binding(
    binding_index: usize,
    bindings_index: &mut Vec<(ProcessedBinding, ActionStateEnum, u64)>,
    event: &InputEvent,
) -> Option<(ActionStateEnum, u64)> {
    let (binding, binding_action_state, action_handle) = &mut bindings_index[binding_index];

    binding.on_event(event).map(|some| {
        *binding_action_state = some;
        (some, *action_handle)
    })
}

impl ProcessedBinding {
    /// Returns None if the action state should not be changed / an even should not fire
    pub(crate) fn on_event(&mut self, event: &InputEvent) -> Option<ActionStateEnum> {
        match (self, event.data) {
            (ProcessedBinding::Button2Bool, InputComponentEvent::Button(state)) => {
                Some(ActionStateEnum::Boolean(state))
            }
            (
                ProcessedBinding::Move2d2Delta2d { sensitivity },
                InputComponentEvent::Move2D(delta),
            ) => Some(ActionStateEnum::Delta2D((
                delta.value.0 * sensitivity.0,
                delta.value.1 * sensitivity.1,
            ))),
            (ProcessedBinding::Cursor2Cursor, InputComponentEvent::Cursor(cursor)) => {
                Some(ActionStateEnum::Cursor(cursor.normalized_screen_coords))
            }
            _ => None,
        }
    }
}
