use std::vec::IntoIter;

use nalgebra::Vector2;
use suinput_types::{
    action::ActionStateEnum, binding::SimpleBinding, event::InputEvent, CreateBindingLayoutError,
    SuPath,
};
use thunderdome::Index;

use crate::internal::input_events::InputEventSources;
use crate::internal::parallel_arena::ParallelArena;
use crate::internal::types::{hash_map::Entry, HashMap};
use crate::types::action_type::{Axis1d, Axis2d, Value};
use crate::{
    action::ActionTypeEnum,
    instance::{BindingLayout, Instance},
    internal::{
        device::DeviceState,
        input_component::InputComponentType,
        interaction_profile::InteractionProfileState,
        paths::{InputPath, InteractionProfilePath, UserPath},
    },
};

use super::processed_binding::{Axis, GyroBindingSpace, ProcessedBinding, Sensitivity};
use super::WorkingUserInterface;

#[derive(Debug, Clone)]
pub struct ProcessedBindingLayout {
    pub(crate) bindings_index: Vec<(ProcessedBinding, ActionStateEnum, u64)>,
    binding_states: Vec<ActionStateEnum>,
    bindings_for_action: HashMap<u64, Vec<usize>>,
    input_bindings: HashMap<UserPath, HashMap<InputPath, Vec<usize>>>,
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
                CreateBindingLayoutError::BadInteractionProfilePath,
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
                        CreateBindingLayoutError::BadUserPath,
                    ))
                }
            };

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
                    if action.data_type == ActionTypeEnum::Boolean {
                        ProcessedBinding::Button2Bool
                    } else if action.data_type == ActionTypeEnum::Value {
                        ProcessedBinding::Button2Value
                    } else {
                        return Err(CreateBindingLayoutError::BadBinding(*binding));
                    }
                }
                Some(InputComponentType::Trigger) => {
                    if action.data_type == ActionTypeEnum::Boolean {
                        ProcessedBinding::Trigger2Bool
                    } else if action.data_type == ActionTypeEnum::Value {
                        ProcessedBinding::Trigger2Value
                    } else {
                        return Err(CreateBindingLayoutError::BadBinding(*binding));
                    }
                }
                Some(InputComponentType::Move2D) => {
                    if action.data_type != ActionTypeEnum::Delta2d {
                        return Err(CreateBindingLayoutError::BadBinding(*binding));
                    }

                    ProcessedBinding::Move2d2Delta2d {
                        sensitivity: (1., 1.),
                    }
                }
                Some(InputComponentType::Cursor) => {
                    if action.data_type != ActionTypeEnum::Cursor {
                        return Err(CreateBindingLayoutError::BadBinding(*binding));
                    }

                    ProcessedBinding::Cursor2Cursor
                }
                Some(InputComponentType::Joystick) => {
                    if action.data_type != ActionTypeEnum::Axis2d {
                        return Err(CreateBindingLayoutError::BadBinding(*binding));
                    }

                    ProcessedBinding::Joystick2Axis2d
                }
                Some(InputComponentType::Gyro(_)) => {
                    if action.data_type != ActionTypeEnum::Delta2d {
                        return Err(CreateBindingLayoutError::BadBinding(*binding));
                    }

                    //TODO default depending on controller type somehow
                    //Handheld -> Local
                    //Controller -> Player
                    ProcessedBinding::Gyro2Delta2d {
                        last_time: None,
                        space: GyroBindingSpace::PlayerSpace {
                            relax_factor: GyroBindingSpace::calc_relax_factor(60.),
                            x_axis: Axis::Yaw,
                        },
                        // cut_off_speed: 0.,
                        // cut_off_recovery: 0.,
                        // smooth_threshold: 0.,
                        // smooth_time: 0.125,
                        sensitivity: Sensitivity::Linear(1.),
                    }
                }
                Some(InputComponentType::Accel) => {
                    return Err(CreateBindingLayoutError::BadBinding(*binding));
                }
                None => {
                    return Err(instance.get_path_string(component_path).map_or(
                        CreateBindingLayoutError::InvalidPathHandle(interaction_profile),
                        CreateBindingLayoutError::BadComponentPath,
                    ))
                }
            };

            let action_state = match action.data_type {
                ActionTypeEnum::Boolean => ActionStateEnum::Boolean(false),
                ActionTypeEnum::Delta2d => ActionStateEnum::Delta2d(Vector2::new(0., 0.).into()),
                ActionTypeEnum::Cursor => ActionStateEnum::Cursor(Vector2::new(0., 0.).into()),
                ActionTypeEnum::Axis1d => ActionStateEnum::Axis1d(0.),
                ActionTypeEnum::Value => ActionStateEnum::Value(0.),
                ActionTypeEnum::Axis2d => ActionStateEnum::Axis2d(Vector2::new(0., 0.).into()),
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
            binding_states: bindings_index.iter().map(|(_, b, _)| *b).collect(),
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

    pub(crate) fn handle_component_event(
        &mut self,
        user_path: SuPath,
        event: &InputEvent,
        interaction_profile: &InteractionProfileState,
        devices: &ParallelArena<(DeviceState, Index)>,
        interface: &mut WorkingUserInterface,
    ) {
        if let Some(component_bindings) = self.input_bindings.get(&user_path) {
            if let Some(bindings) = component_bindings.get(&event.path) {
                let max_priority = bindings.iter().fold(0, |fold, binding_index| {
                    let (_, _, action) = &self.bindings_index[*binding_index];
                    let priority = interface.get_action_priority(*action);
                    priority.max(fold)
                });

                for &binding_index in bindings {
                    let (binding, _, action_handle) = &mut self.bindings_index[binding_index];

                    let priority = interface.get_action_priority(*action_handle);

                    if priority < max_priority {
                        continue;
                    }

                    if let Some(new_binding_state) =
                        binding.on_event(user_path, event, interaction_profile, devices)
                    {
                        let new_binding_state = match new_binding_state {
                            ActionStateEnum::Boolean(new_state) => {
                                *self.binding_states.get_mut(binding_index).unwrap() =
                                    ActionStateEnum::Boolean(new_state);
                                Aggregator {
                                    binding_states: &self.binding_states,
                                    bindings_for_action: &self.bindings_for_action,
                                    interface,
                                }
                                .aggregate::<bool>(*action_handle, new_state, binding_index)
                                .map(|(new_state, _changed)| ActionStateEnum::Boolean(new_state))
                            }
                            ActionStateEnum::Value(new_state) => {
                                *self.binding_states.get_mut(binding_index).unwrap() =
                                    ActionStateEnum::Value(new_state);
                                Aggregator {
                                    binding_states: &self.binding_states,
                                    bindings_for_action: &self.bindings_for_action,
                                    interface,
                                }
                                .aggregate::<Value>(*action_handle, new_state, binding_index)
                                .map(ActionStateEnum::Value)
                            }
                            ActionStateEnum::Axis1d(new_state) => {
                                *self.binding_states.get_mut(binding_index).unwrap() =
                                    ActionStateEnum::Axis1d(new_state);
                                Aggregator {
                                    binding_states: &self.binding_states,
                                    bindings_for_action: &self.bindings_for_action,
                                    interface,
                                }
                                .aggregate::<Axis1d>(*action_handle, new_state, binding_index)
                                .map(ActionStateEnum::Axis1d)
                            }
                            ActionStateEnum::Axis2d(new_state) => {
                                *self.binding_states.get_mut(binding_index).unwrap() =
                                    ActionStateEnum::Axis2d(new_state);
                                Aggregator {
                                    binding_states: &self.binding_states,
                                    bindings_for_action: &self.bindings_for_action,
                                    interface,
                                }
                                .aggregate::<Axis2d>(
                                    *action_handle,
                                    new_state.into(),
                                    binding_index,
                                )
                                .map(|new_state| ActionStateEnum::Axis2d(new_state.into()))
                            }
                            _ => Some(new_binding_state),
                        };

                        if let Some(new_binding_state) = new_binding_state {
                            interface.fire_action_event(*action_handle, new_binding_state)
                        }
                    }
                }
            }
        }
    }
}

struct Aggregator<'a> {
    binding_states: &'a Vec<ActionStateEnum>,
    bindings_for_action: &'a HashMap<u64, Vec<usize>>,
    interface: &'a WorkingUserInterface<'a>,
}

impl<'a> InputEventSources for Aggregator<'a> {
    type Index = u64;
    type SourceIndex = usize;
    type Sources = IntoIter<usize>;

    fn get_state<I: crate::internal::input_events::InputEventType>(
        &self,
        action_handle: Self::Index,
    ) -> Option<I::Value> {
        self.interface
            .get_action_state(action_handle)
            .map(|s| I::from_ase(&s))
    }

    fn get_source_state<I: crate::internal::input_events::InputEventType>(
        &self,
        _: Self::Index,
        source_idx: Self::SourceIndex,
    ) -> Option<I::Value> {
        self.binding_states.get(source_idx).map(I::from_ase)
    }

    fn get_sources<I: crate::internal::input_events::InputEventType>(
        &self,
        idx: Self::Index,
    ) -> Self::Sources {
        self.bindings_for_action
            .get(&idx)
            .unwrap()
            .clone()
            .into_iter()
    }
}
