use std::sync::Arc;
use std::vec::IntoIter;

use nalgebra::Vector2;

use suinput_types::{
    action::ActionStateEnum, binding::SimpleBinding, event::InputEvent, CreateBindingLayoutError,
    SuPath,
};

use crate::action::Action;
use crate::action_set::ActionSet;
use crate::internal::input_component::InternalActionState;
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

use super::processed_binding::{
    Axis, GyroBindingSpace, ProcessedBindingType, ProcessedInputBinding, Sensitivity,
};
use super::WorkingUserInterface;

#[derive(Debug, Clone)]
pub struct ProcessedBindingLayout {
    pub(crate) bindings_index: Vec<ProcessedInputBinding>,
    bindings_for_action: HashMap<u64, Vec<usize>>,
    bindings_for_input: HashMap<(UserPath, InputPath), (Vec<usize>, u32)>,
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

        let mut bindings_index = Vec::<ProcessedInputBinding>::new();
        let mut input_bindings = HashMap::<(UserPath, InputPath), (Vec<usize>, u32)>::new();
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

            if !input_bindings.contains_key(&(user_path, component_path)) {
                input_bindings.insert((user_path, component_path), (Vec::with_capacity(1), 0));
            }

            let action = actions.get((binding.action as usize) - 1).ok_or(
                CreateBindingLayoutError::InvalidActionHandle(binding.action),
            )?;

            let processed_binding = match device.input_components.get(&component_path) {
                Some(InputComponentType::Button) => {
                    if action.data_type == ActionTypeEnum::Boolean {
                        ProcessedBindingType::Button2Bool
                    } else if action.data_type == ActionTypeEnum::Value {
                        ProcessedBindingType::Button2Value
                    } else {
                        return Err(CreateBindingLayoutError::BadBinding(*binding));
                    }
                }
                Some(InputComponentType::Trigger) => {
                    if action.data_type == ActionTypeEnum::Boolean {
                        ProcessedBindingType::Trigger2Bool
                    } else if action.data_type == ActionTypeEnum::Value {
                        ProcessedBindingType::Trigger2Value
                    } else {
                        return Err(CreateBindingLayoutError::BadBinding(*binding));
                    }
                }
                Some(InputComponentType::Move2D) => {
                    if action.data_type != ActionTypeEnum::Delta2d {
                        return Err(CreateBindingLayoutError::BadBinding(*binding));
                    }

                    ProcessedBindingType::Move2d2Delta2d {
                        sensitivity: (1., 1.),
                    }
                }
                Some(InputComponentType::Joystick) => {
                    if action.data_type != ActionTypeEnum::Axis2d {
                        return Err(CreateBindingLayoutError::BadBinding(*binding));
                    }

                    ProcessedBindingType::Joystick2Axis2d
                }
                Some(InputComponentType::Gyro(_)) => {
                    if action.data_type != ActionTypeEnum::Delta2d {
                        return Err(CreateBindingLayoutError::BadBinding(*binding));
                    }

                    //TODO default depending on controller type somehow
                    //Handheld -> Local
                    //Controller -> Player
                    ProcessedBindingType::Gyro2Delta2d {
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
                ActionTypeEnum::Boolean => InternalActionState::Boolean(false),
                ActionTypeEnum::Axis1d => InternalActionState::Axis1d(0.),
                ActionTypeEnum::Value => InternalActionState::Value(0.),
                ActionTypeEnum::Axis2d => InternalActionState::Axis2d(Vector2::new(0., 0.)),
                _ => InternalActionState::NonApplicable,
            };

            bindings_index.push(ProcessedInputBinding {
                ty: processed_binding,
                state: action_state,
                action: action.handle,
                input_component: (user_path, component_path),
            });
            input_bindings
                .get_mut(&(user_path, component_path))
                .unwrap()
                .0
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
            bindings_for_input: input_bindings,
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
        binding_layout.processed_cache.clone()
    }

    pub(crate) fn handle_component_event(
        &mut self,
        user_path: SuPath,
        event: &InputEvent,
        interaction_profile: &InteractionProfileState,
        devices: &ParallelArena<(DeviceState, InteractionProfilePath)>,
        interface: &mut WorkingUserInterface,
    ) {
        if let Some((bindings, max_priority)) =
            self.bindings_for_input.get(&(user_path, event.path))
        {
            for &binding_index in bindings {
                let binding = &mut self.bindings_index[binding_index];

                if !interface.is_action_active(binding.action) {
                    continue;
                }

                let priority = interface.get_action_priority(binding.action);

                if priority < *max_priority {
                    continue;
                }

                let action = binding.action;

                if let Some(new_binding_state) =
                    binding
                        .ty
                        .on_event(user_path, event, interaction_profile, devices)
                {
                    binding.save_state(&new_binding_state);
                    let new_binding_state = Self::aggregate(
                        interface,
                        new_binding_state,
                        binding.action,
                        &self.bindings_index,
                        &self.bindings_for_action,
                        binding_index,
                    );

                    if let Some(new_binding_state) = new_binding_state {
                        interface.fire_action_event(action, new_binding_state)
                    }
                }
            }
        }
    }

    fn aggregate(
        interface: &WorkingUserInterface,
        new_binding_state: ActionStateEnum,
        action: u64,
        bindings_index: &Vec<ProcessedInputBinding>,
        bindings_for_action: &HashMap<u64, Vec<usize>>,
        binding_idx: usize,
    ) -> Option<ActionStateEnum> {
        match new_binding_state {
            ActionStateEnum::Boolean(new_state) => Aggregator {
                bindings: bindings_index,
                bindings_for_action,
                interface,
            }
            .aggregate::<bool>(action, new_state, binding_idx)
            .map(|(new_state, _changed)| ActionStateEnum::Boolean(new_state)),
            ActionStateEnum::Value(new_state) => Aggregator {
                bindings: bindings_index,
                bindings_for_action,
                interface,
            }
            .aggregate::<Value>(action, new_state, binding_idx)
            .map(ActionStateEnum::Value),
            ActionStateEnum::Axis1d(new_state) => Aggregator {
                bindings: bindings_index,
                bindings_for_action,
                interface,
            }
            .aggregate::<Axis1d>(action, new_state, binding_idx)
            .map(ActionStateEnum::Axis1d),
            ActionStateEnum::Axis2d(new_state) => Aggregator {
                bindings: bindings_index,
                bindings_for_action,
                interface,
            }
            .aggregate::<Axis2d>(action, new_state.into(), binding_idx)
            .map(|new_state| ActionStateEnum::Axis2d(new_state.into())),
            _ => Some(new_binding_state),
        }
    }

    pub(crate) fn change_active_action_sets(
        &mut self,
        interaction_profile: &InteractionProfileState,
        interface: &mut WorkingUserInterface,
        //Sorted min priority to max
        disabling: &[&Arc<ActionSet>],
        //Sorted max priority to min
        enabling: &[&Arc<ActionSet>],
    ) {
        for action_set in enabling {
            for action in action_set.baked_actions.get().unwrap() {
                self.handle_action_enable(action_set, action, interface, interaction_profile);
            }
        }

        for action_set in disabling {
            for action in action_set.baked_actions.get().unwrap() {
                self.handle_action_disable(action_set, action, interface, interaction_profile);
            }
        }
    }

    pub(crate) fn handle_action_enable(
        &mut self,
        action_set: &Arc<ActionSet>,
        action: &Arc<Action>,
        interface: &mut WorkingUserInterface,
        interaction_profile: &InteractionProfileState,
    ) {
        if let Some(action_bindings) = self.bindings_for_action.get(&action.handle) {
            //For each binding to the action
            for action_binding_idx in action_bindings {
                //Get the input component
                let input_component = self
                    .bindings_index
                    .get(*action_binding_idx)
                    .unwrap()
                    .input_component;
                //Get the bindings to and max priority of said input component
                let (bindings, old_max_priority) =
                    self.bindings_for_input.get(&input_component).unwrap();

                //If the old max priority is less than the new action then interrupt the old bindings and trigger the action's bindings
                if *old_max_priority < action_set.default_priority {
                    for other_binding_idx in bindings {
                        let binding = self.bindings_index.get_mut(*other_binding_idx).unwrap();

                        if !interface.is_action_active(binding.action) {
                            continue;
                        }

                        if interface.get_action_priority(binding.action) == *old_max_priority {
                            if let Some(event) = binding.ty.interrupt(&binding.state) {
                                binding.save_state(&event);
                                let binding_action = binding.action;
                                if let Some(event) = Self::aggregate(
                                    interface,
                                    event,
                                    binding_action,
                                    &self.bindings_index,
                                    &self.bindings_for_action,
                                    *other_binding_idx,
                                ) {
                                    interface.fire_action_event(binding_action, event);
                                }
                            }
                        }
                    }

                    //If we have a cached component state then 'activate' the action binding
                    if let Some(component_state) = interaction_profile
                        .get_input_component_state(input_component.0, input_component.1)
                    {
                        let binding = self.bindings_index.get_mut(*action_binding_idx).unwrap();
                        if let Some(event) = binding.ty.activate(component_state) {
                            binding.save_state(&event);
                            if let Some(event) = Self::aggregate(
                                interface,
                                event,
                                action.handle,
                                &self.bindings_index,
                                &self.bindings_for_action,
                                *action_binding_idx,
                            ) {
                                interface.fire_action_event(action.handle, event);
                            }
                        }
                    }

                    //Update the max priority
                    self.bindings_for_input.get_mut(&input_component).unwrap().1 =
                        action_set.default_priority;
                }
            }
        }
    }

    fn handle_action_disable(
        &mut self,
        action_set: &Arc<ActionSet>,
        action: &Arc<Action>,
        interface: &mut WorkingUserInterface,
        interaction_profile: &InteractionProfileState,
    ) {
        if let Some(action_bindings) = self.bindings_for_action.get(&action.handle) {
            //For each binding to the action
            for action_binding_idx in action_bindings {
                //Get the input component
                let action_binding = self.bindings_index.get_mut(*action_binding_idx).unwrap();
                let input_component = action_binding.input_component;
                //Get the bindings to and max priority of said input component
                let (bindings, old_max_priority) =
                    self.bindings_for_input.get(&input_component).unwrap();

                //If the old max priority is the same as the action's priority
                if *old_max_priority == action_set.default_priority {
                    //Interrupt the action's binding
                    if let Some(event) = action_binding.ty.interrupt(&action_binding.state) {
                        action_binding.save_state(&event);
                        if let Some(event) = Self::aggregate(
                            interface,
                            event,
                            action.handle,
                            &self.bindings_index,
                            &self.bindings_for_action,
                            *action_binding_idx,
                        ) {
                            interface.fire_action_event(action.handle, event);
                        }
                    }

                    let mut new_max_priority = 0;

                    //Find the max priority of the other bindings
                    for binding_idx in bindings {
                        let binding = self.bindings_index.get_mut(*binding_idx).unwrap();
                        if interface.is_action_active(binding.action) {
                            new_max_priority =
                                new_max_priority.max(interface.get_action_priority(binding.action))
                        }
                    }

                    //If the new max priority is lower then 'activate' the new max priority bindings
                    if new_max_priority < *old_max_priority {
                        if let Some(component_state) = interaction_profile
                            .get_input_component_state(input_component.0, input_component.1)
                        {
                            for binding_idx in bindings {
                                let binding = self.bindings_index.get_mut(*binding_idx).unwrap();
                                if interface.is_action_active(binding.action)
                                    && new_max_priority
                                        == interface.get_action_priority(binding.action)
                                {
                                    if let Some(event) = binding.ty.activate(component_state) {
                                        binding.save_state(&event);
                                        let binding_action = binding.action;
                                        if let Some(event) = Self::aggregate(
                                            interface,
                                            event,
                                            binding_action,
                                            &self.bindings_index,
                                            &self.bindings_for_action,
                                            *action_binding_idx,
                                        ) {
                                            interface.fire_action_event(binding_action, event);
                                        }
                                    }
                                }
                            }
                        }

                        self.bindings_for_input.get_mut(&input_component).unwrap().1 =
                            new_max_priority;
                    }
                }
            }
        }
    }
}

struct Aggregator<'a> {
    bindings: &'a Vec<ProcessedInputBinding>,
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
        self.bindings
            .get(source_idx)
            .map(|binding| I::from_ias(&binding.state))
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
