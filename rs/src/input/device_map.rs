use std::collections::HashMap;

use godot::{
    classes::{
        Input, InputEvent, InputEventKey, InputEventMouse, InputMap,
        class_macros::private::virtuals::ZipReader::{StringName, Vector2, real},
        resource::DeepDuplicateMode,
    },
    meta::ClassId,
    obj::Singleton as _,
    prelude::{Base, GodotClass, INode, Node, godot_api},
};

use crate::input::{Action, MovementFrame};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputAction {
    AvatarLeft,
    AvatarRight,
    AvatarUp,
    AvatarDown,

    AttackLight,
    AttackHeavy,
    Dash,
    Jump,
}

impl InputAction {
    pub const ACTION_NAMES: &[(Self, &str)] = &[
        (Self::AvatarLeft, Self::AvatarLeft.get_default_name()),
        (Self::AvatarRight, Self::AvatarRight.get_default_name()),
        (Self::AvatarUp, Self::AvatarUp.get_default_name()),
        (Self::AvatarDown, Self::AvatarDown.get_default_name()),
        (Self::Jump, Self::Jump.get_default_name()),
        (Self::Dash, Self::Dash.get_default_name()),
        (Self::AttackLight, Self::AttackLight.get_default_name()),
        (Self::AttackHeavy, Self::AttackHeavy.get_default_name()),
    ];

    pub const fn get_default_name(self) -> &'static str {
        match self {
            Self::AvatarLeft => "avatar_move_left",
            Self::AvatarRight => "avatar_move_right",
            Self::AvatarUp => "avatar_move_up",
            Self::AvatarDown => "avatar_move_down",
            Self::AttackLight => "avatar_attack_light",
            Self::AttackHeavy => "avatar_attack_heavy",
            Self::Dash => "avatar_dash",
            Self::Jump => "avatar_jump",
        }
    }

    pub fn build_map(index: usize) -> HashMap<Self, StringName> {
        HashMap::from_iter(
            Self::ACTION_NAMES
                .iter()
                .map(|(action, name)| (*action, StringName::from(&format!("{name}_{index}")))),
        )
    }
}

pub struct InputChannel {
    index: usize,
    action_names: HashMap<InputAction, StringName>,
}

impl InputChannel {
    pub fn new(index: usize) -> Self {
        let res = Self {
            index,
            action_names: InputAction::build_map(index),
        };
        res.register();
        res
    }

    fn register(&self) {
        tracing::trace!(index = self.index, "register input mixer");
        let mut map = InputMap::singleton();
        for (action, name) in &self.action_names {
            tracing::trace!(%name, "register action");
            let dz = map.action_get_deadzone(action.get_default_name());
            map.add_action_ex(name).deadzone(dz).done();
        }
    }

    pub fn register_device(&self, device: i32) {
        const JOYPAD_UNIVERSAL: i32 = -1;
        const fn device_is_joypad(device: i32) -> bool {
            device != InputEvent::DEVICE_ID_KEYBOARD && device != InputEvent::DEVICE_ID_MOUSE
        }
        const fn device_matches_event(event: i32, device: i32) -> bool {
            event == device || (device_is_joypad(device) && event == JOYPAD_UNIVERSAL)
        }

        tracing::trace!(index = self.index, device, "register input mixer device");

        let mut map = InputMap::singleton();
        for (action, name) in &self.action_names {
            let events = map.action_get_events(action.get_default_name());
            for event in events.iter_shared() {
                let mut evdev = event.get_device();
                if event.is_dynamic_class(InputEventKey::class_id()) {
                    evdev = InputEvent::DEVICE_ID_KEYBOARD;
                } else if event.is_dynamic_class(InputEventMouse::class_id()) {
                    evdev = InputEvent::DEVICE_ID_MOUSE;
                }

                if !device_matches_event(evdev, device) {
                    continue;
                }

                let mut event = event
                    .duplicate_resource_ex()
                    .deep(DeepDuplicateMode::ALL)
                    .done();
                event.set_device(device);

                tracing::trace!(action = %name, device, %evdev, event = %event.as_text(), "register event");

                map.action_add_event(name, &event);
            }
        }
    }

    pub fn is_action_pressed(&self, action: InputAction) -> bool {
        Input::singleton().is_action_pressed(&self.action_names[&action])
    }

    pub fn is_action_just_pressed(&self, action: InputAction) -> bool {
        Input::singleton().is_action_just_pressed(&self.action_names[&action])
    }

    pub fn get_action_strength(&self, action: InputAction) -> real {
        Input::singleton().get_action_strength(&self.action_names[&action])
    }

    pub fn get_vector(
        &self,
        deadzone: real,
        negative_x: InputAction,
        positive_x: InputAction,
        negative_y: InputAction,
        positive_y: InputAction,
    ) -> Vector2 {
        Input::singleton()
            .get_vector_ex(
                &self.action_names[&negative_x],
                &self.action_names[&positive_x],
                &self.action_names[&negative_y],
                &self.action_names[&positive_y],
            )
            .deadzone(deadzone)
            .done()
    }

    pub fn get_movement_frame(
        &self,
        deadzone: real,
        negative_x: InputAction,
        positive_x: InputAction,
        negative_y: InputAction,
        positive_y: InputAction,
    ) -> Option<MovementFrame> {
        MovementFrame::from_vector(
            deadzone,
            self.get_vector(deadzone, negative_x, positive_x, negative_y, positive_y),
        )
    }

    pub fn get_dominant_action(&self) -> Option<Action> {
        if self.is_action_just_pressed(InputAction::Dash) {
            Some(Action::Dash)
        } else if self.is_action_just_pressed(InputAction::Jump) {
            Some(Action::Jump)
        } else if self.is_action_just_pressed(InputAction::AttackLight) {
            Some(Action::AttackLight)
        } else if self.is_action_just_pressed(InputAction::AttackHeavy) {
            Some(Action::AttackHeavy)
        } else {
            None
        }
    }
}

#[derive(GodotClass)]
#[class(init, base = Node)]
pub struct DeviceManagerNode {
    base: Base<Node>,
}

#[godot_api]
impl INode for DeviceManagerNode {}
