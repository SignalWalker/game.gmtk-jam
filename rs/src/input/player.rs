use std::collections::VecDeque;

use godot::{
    classes::{Input, InputEvent, class_macros::private::virtuals::Xrvrs::Gd},
    obj::{NewAlloc as _, Singleton as _, WithBaseField as _},
    prelude::{Base, GodotClass, INode, Node, godot_api, godot_dyn},
};
use godot_utils::ArrayExt;

use crate::{
    fighter::{FacingDirection, Fighter2D},
    input::{Action, AxisDir, FighterController, InputAction, InputChannel, MovementFrame},
};

pub const AVATAR_UP: &str = "avatar_move_up";
pub const AVATAR_DOWN: &str = "avatar_move_down";
pub const AVATAR_LEFT: &str = "avatar_move_left";
pub const AVATAR_RIGHT: &str = "avatar_move_right";

pub const AVATAR_JUMP: &str = "avatar_jump";
pub const AVATAR_DASH: &str = "avatar_dash";
pub const AVATAR_LIGHT: &str = "avatar_attack_light";
pub const AVATAR_HEAVY: &str = "avatar_attack_heavy";

pub struct QueuedAction {
    pub kind: Action,
    pub age: u32,
}

impl QueuedAction {
    pub fn capture(input: &Input, kind: Action) -> Option<Self> {
        if input.is_action_just_pressed(match kind {
            Action::AttackLight => AVATAR_LIGHT,
            Action::AttackHeavy => AVATAR_HEAVY,
            Action::Dash => AVATAR_DASH,
            Action::Jump => AVATAR_JUMP,
        }) {
            Some(Self { kind, age: 0 })
        } else {
            None
        }
    }
}

#[derive(GodotClass)]
#[class(init, base = Node)]
pub struct FighterControllerPlayer {
    base: Base<Node>,

    #[export]
    #[init(val = 0.2)]
    pub movement_deadzone: f32,

    #[export]
    #[init(val = 12)]
    #[var(pub, set)]
    movement_buffer_len: u32,

    #[export]
    #[init(val = 12)]
    pub attack_light_buffer_len: u32,

    #[export]
    #[init(val = 12)]
    pub attack_heavy_buffer_len: u32,

    #[export]
    #[init(val = 6)]
    pub dash_buffer_len: u32,

    #[export]
    #[init(val = 6)]
    pub jump_buffer_len: u32,

    #[export]
    #[init(val = 1)]
    pub fastfall_window: u32,

    movement_buffer: VecDeque<MovementFrame>,

    pub maintaining_jump: bool,

    action: Option<QueuedAction>,

    input: Option<InputChannel>,
}

#[godot_dyn]
impl FighterController for FighterControllerPlayer {
    fn preprocess(&mut self, _: &Fighter2D) {}

    fn current_horizontal(&self, _: &Fighter2D) -> AxisDir {
        self.current_horizontal()
    }

    fn consume_action(&mut self, _: &Fighter2D) -> Option<Action> {
        self.consume_action()
    }

    fn peek_action(&self, _: &Fighter2D) -> Option<Action> {
        self.peek_action()
    }

    fn should_maintain_jump(&self, _: &Fighter2D) -> bool {
        self.maintaining_jump
    }

    fn should_maintain_dash(&self, fighter: &Fighter2D) -> bool {
        self.should_maintain_dash(fighter.facing)
    }

    fn wants_fastfall(&self, _: &Fighter2D) -> bool {
        self.wants_fastfall()
    }
}

impl FighterControllerPlayer {
    pub fn with_input_channel(mixer: InputChannel) -> Gd<Self> {
        let mut res = Self::new_alloc();
        res.bind_mut().input = Some(mixer);
        res
    }

    pub fn current_horizontal(&self) -> AxisDir {
        self.movement_buffer
            .front()
            .map_or(AxisDir::Neutral, |m| m.horizontal)
    }

    pub fn peek_action(&self) -> Option<Action> {
        self.action.as_ref().map(|f| f.kind)
    }

    pub fn consume_action(&mut self) -> Option<Action> {
        self.action.take().map(|f| f.kind)
    }

    pub fn wants_fastfall(&self) -> bool {
        fn is_fastfall_movement(frame: MovementFrame) -> bool {
            frame.vertical == AxisDir::Negative && frame.horizontal == AxisDir::Neutral
        }

        let mut frames = self
            .movement_buffer
            .iter()
            .take(self.fastfall_window.saturating_add(1).saturating_cast())
            .copied();
        let Some(current) = frames.next() else {
            return false;
        };

        // if our current movement is not a fastfall frame, we don't care
        if !is_fastfall_movement(current) {
            return false;
        }

        //
        for frame in frames {
            if !is_fastfall_movement(frame) {
                return true;
            }
        }
        false
    }

    pub fn should_maintain_dash(&self, dash_dir: FacingDirection) -> bool {
        let c = self.current_horizontal();
        (dash_dir == FacingDirection::Right && c == AxisDir::Positive)
            || (dash_dir == FacingDirection::Left && c == AxisDir::Negative)
    }

    fn get_movement_frame(&self) -> Option<MovementFrame> {
        self.input.as_ref().and_then(|i| {
            i.get_movement_frame(
                self.movement_deadzone,
                InputAction::AvatarLeft,
                InputAction::AvatarRight,
                InputAction::AvatarDown,
                InputAction::AvatarUp,
            )
        })
    }

    fn get_dominant_action(&self) -> Option<Action> {
        self.input
            .as_ref()
            .and_then(InputChannel::get_dominant_action)
    }
}

#[godot_api]
impl FighterControllerPlayer {
    #[func]
    fn new_with_input_channel(channel: u32) -> Gd<Self> {
        Self::with_input_channel(InputChannel::new(channel.saturating_cast()))
    }

    #[func]
    pub fn register_input_device(&self, device: i32) {
        if let Some(input) = self.input.as_ref() {
            input.register_device(device);
        }
    }

    #[func]
    pub fn set_movement_buffer_len(&mut self, len: u32) {
        self.movement_buffer_len = len;

        // resize the buffer
        let new = len.saturating_cast::<usize>();
        let old = self.movement_buffer.len();
        if old < new {
            self.movement_buffer.reserve_exact((new - old) + 1);
        } else {
            self.movement_buffer.truncate(new);
        }
    }
}

#[godot_api]
impl INode for FighterControllerPlayer {
    fn enter_tree(&mut self) {
        // make sure this gets processed before anything else so we don't have a single-frame input
        // delay
        if self.base().get_physics_process_priority() >= 0 {
            self.base_mut().set_physics_process_priority(-1);
        }

        Input::singleton().set_use_accumulated_input(false);

        // register this controller in the parent
        if let Some(mut p) = self
            .base()
            .get_parent()
            .and_then(|p| p.try_cast::<Fighter2D>().ok())
        {
            p.bind_mut().register_controller(self.to_gd());

            if !crate::state::started_game_normally()
                && self.input.is_none()
                && let Some(p) = p.get_parent()
            {
                let self_id = self.base().instance_id();
                let mut mixer_index = 0;

                for fighter in p.get_children().into_iter_shared_of_type::<Fighter2D>() {
                    for controller in fighter
                        .get_children()
                        .into_iter_shared_of_type::<FighterControllerPlayer>()
                    {
                        if controller.instance_id() == self_id {
                            continue;
                        }
                        if controller.bind().input.is_some() {
                            mixer_index += 1;
                        }
                    }
                }

                tracing::trace!(index = mixer_index, name = %self.base().get_name(), "register fighter controller mixer");
                let mixer = InputChannel::new(mixer_index);
                match mixer_index {
                    0 => mixer.register_device(0),
                    1 => {
                        mixer.register_device(1);
                        mixer.register_device(InputEvent::DEVICE_ID_KEYBOARD);
                        mixer.register_device(InputEvent::DEVICE_ID_MOUSE);
                    }
                    _ => {}
                }
                self.input = Some(mixer);
            }
        }
    }

    fn physics_process(&mut self, _delta: f64) {
        // update the movement buffer
        let movement = self.get_movement_frame();

        self.movement_buffer
            .push_front(movement.unwrap_or(MovementFrame::NEUTRAL));
        if self.movement_buffer.len() > self.movement_buffer_len.saturating_cast::<usize>() {
            self.movement_buffer.pop_back();
        }

        // age the queued action
        if let Some(action) = self.action.as_mut() {
            action.age += 1;
            if action.age
                >= (match action.kind {
                    Action::AttackLight => self.attack_light_buffer_len,
                    Action::AttackHeavy => self.attack_heavy_buffer_len,
                    Action::Dash => self.dash_buffer_len,
                    Action::Jump => self.jump_buffer_len,
                })
            {
                self.action = None;
            }
        }

        // insert new actions
        if let Some(action) = self.get_dominant_action() {
            self.action = Some(QueuedAction {
                kind: action,
                age: 0,
            });
        }

        // update jump maintanence
        self.maintaining_jump = self
            .input
            .as_ref()
            .is_some_and(|i| i.is_action_pressed(InputAction::Jump));
    }
}
