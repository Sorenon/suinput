use std::{
    collections::{hash_map::Entry, HashMap},
    time::Instant,
};

use parking_lot::RwLock;
use suinput_types::{
    action::ActionType,
    event::{InputComponentEvent, InputEvent},
    SuPath,
};

use crate::{
    device::InputComponentType,
    instance::{BindingLayout, Instance},
    interaction_profile::InteractionProfileType,
};

pub(crate) struct ProcessedBindingLayout {
    //TODO replace RwLock with Cell
    bindings_index: RwLock<Vec<(ProcessedBinding, ActionStateEnum, u64)>>,
    input_bindings: HashMap<SuPath, HashMap<SuPath, Vec<usize>>>,
    bindings_for_action: HashMap<u64, Vec<usize>>,
}

impl ProcessedBindingLayout {
    pub fn new(
        instance: &Instance,
        interaction_profile: SuPath,
        binding_layout: &BindingLayout,
    ) -> Self {
        let runtime = instance.runtime.upgrade().unwrap();
        let actions = instance.actions.read();

        if interaction_profile != binding_layout.interaction_profile {
            todo!("Binding Layout conversion not yet implemented")
        }

        //TODO store interaction profile types in runtime
        let interaction_profile_type =
            InteractionProfileType::new_desktop_profile(|str| instance.get_path(str).unwrap());
        assert_eq!(interaction_profile, interaction_profile_type.id);

        let mut bindings_index = Vec::<(ProcessedBinding, ActionStateEnum, u64)>::new();
        let mut input_bindings = HashMap::<SuPath, HashMap<SuPath, Vec<usize>>>::new();
        let mut bindings_for_action = HashMap::<u64, Vec<usize>>::new();

        for binding in &binding_layout.bindings {
            let path_string = instance.get_path_string(binding.path).unwrap();

            let split_idx = path_string.find("/input").expect("Invalid path string");
            let (user_str, component_str) = path_string.split_at(split_idx);

            let user_path = instance.get_path(user_str).unwrap();
            let device = interaction_profile_type
                .user2device
                .get(&user_path)
                .unwrap();
            let device = runtime.device_types.get(device).unwrap();

            let component_path = instance.get_path(component_str).unwrap();

            if !input_bindings.contains_key(&user_path) {
                input_bindings.insert(user_path, HashMap::new());
            }

            let component_paths = input_bindings.get_mut(&user_path).unwrap();

            if !component_paths.contains_key(&component_path) {
                component_paths.insert(component_path, Vec::with_capacity(1));
            }

            let action = actions.get((binding.action as usize) - 1).unwrap();

            let processed_binding = match device.input_components.get(&component_path) {
                Some(InputComponentType::Button) => {
                    assert_eq!(action.action_type, ActionType::Boolean);
                    ProcessedBinding::Button2Bool
                }
                Some(InputComponentType::Move2D) => {
                    assert_eq!(action.action_type, ActionType::Delta2D);
                    ProcessedBinding::Move2d2Delta2d {
                        sensitivity: (1., 1.),
                    }
                }
                Some(InputComponentType::Cursor) => {
                    assert_eq!(action.action_type, ActionType::Cursor);
                    ProcessedBinding::Cursor2Cursor
                }
                _ => todo!(),
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

        Self {
            bindings_index: RwLock::new(bindings_index),
            input_bindings,
            bindings_for_action,
        }
    }

    pub fn on_event(&self, user_path: SuPath, event: &InputEvent, instance: &Instance) {
        let mut bindings_index = self.bindings_index.write();

        if let Some(component_bindings) = self.input_bindings.get(&user_path) {
            if let Some(bindings) = component_bindings.get(&event.path) {
                for binding_index in bindings {
                    if let Some((event, action_handle)) =
                        execute_binding(*binding_index, &mut bindings_index, event)
                    {
                        // let event = match event {
                        //     ActionEventEnum::Boolean { state, changed } => {
                        //         let none_other_true = self
                        //             .bindings_for_action
                        //             .get(&action_handle)
                        //             .unwrap()
                        //             .iter()
                        //             .filter(|idx| *idx != binding_index)
                        //             .find(|idx| {
                        //                 let (_, state, _) = &bindings_index[**idx];
                        //                 match state {
                        //                     ActionEventEnum::Boolean { state, .. } => *state,
                        //                     _ => unreachable!(),
                        //                 }
                        //             })
                        //             .is_none();

                        //         //TODO just do 'changed' by comparing against the cached action state I don't think there is any point in calculating it ourselves
                        //         if state {
                        //             //If the binding is true we always fire an event
                        //             //Changed is sent if all other bindings are false and this binding changed
                        //             Some(ActionEventEnum::Boolean {
                        //                 state: true,
                        //                 changed: none_other_true && changed,
                        //             })
                        //         } else {
                        //             //If the binding is false we only fire an event if all other bindings are false and this binding changed
                        //             if changed && none_other_true {
                        //                 Some(ActionEventEnum::Boolean {
                        //                     state: false,
                        //                     changed: true,
                        //                 })
                        //             } else {
                        //                 None
                        //             }
                        //         }
                        //     }
                        //     ActionEventEnum::Cursor {
                        //         normalized_screen_coords: _,
                        //     } => todo!(),
                        //     //We always fire Move2d events
                        //     _ => Some(event),
                        // };

                        // if let Some(event) = event {
                        //     for listener in instance.listeners.read().iter() {
                        //         listener.handle_event(ActionEvent {
                        //             action_handle,
                        //             time: Instant::now(),
                        //             data: event,
                        //         })
                        //     }
                        // }
                    }
                }
            }
        }
    }
}

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

#[derive(Debug, Clone, Copy)]
pub(crate)enum ActionStateEnum {
    Boolean(bool),
    Delta2D((f64, f64)),
    Cursor((f64, f64)),
}
