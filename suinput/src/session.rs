use crate::{Inner, SuAction, SuActionSet, SuUser};
use suinput_core::types::action_type::ActionType;
use suinput_types::action::ActionListener;

#[derive(Clone)]
pub struct SuSession(pub(crate) Inner<suinput_core::session::Session>);

impl SuSession {
    pub fn register_event_listener(&self, listener: Box<dyn ActionListener>) -> u64 {
        match &self.0 {
            Inner::Embedded(inner) => inner.register_event_listener(listener),
            Inner::FFI() => todo!(),
        }
    }

    pub fn get_main_user(&self) -> SuUser {
        SuUser(match &self.0 {
            Inner::Embedded(inner) => Inner::Embedded(inner.user.clone()),
            Inner::FFI() => todo!(),
        })
    }

    pub fn sync(&self, action_sets: &[&SuActionSet]) {
        match &self.0 {
            Inner::Embedded(inner) => inner.sync(action_sets.iter().map(|set| match &set.0 {
                Inner::Embedded(action_set) => action_set,
                Inner::FFI() => todo!(),
            })),
            Inner::FFI() => todo!(),
        }
    }

    pub fn get_action_state<T: ActionType>(&self, action: &SuAction<T>) -> Result<T::State, ()> {
        match (&self.0, &action.0) {
            (Inner::Embedded(inner), Inner::Embedded(action)) => {
                inner.get_action_state::<T>(action)
            }
            (Inner::FFI(), Inner::FFI()) => todo!(),
            _ => panic!(),
        }
    }

    pub fn unstick_bool_action(&self, action: &SuAction<bool>) {
        match (&self.0, &action.0) {
            (Inner::Embedded(inner), Inner::Embedded(action)) => inner.unstick_bool_action(action),
            (Inner::FFI(), Inner::FFI()) => todo!(),
            _ => panic!(),
        }
    }

    // pub fn create_action_space(&self, action: &SuAction<Pose>, pose_in_space: Pose) {
    //     todo!()
    // }
}
