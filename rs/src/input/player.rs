use std::collections::VecDeque;

use godot::{
    classes::{
        Input,
        class_macros::private::virtuals::ZipReader::{Vector2, real},
    },
    obj::{Singleton as _, WithBaseField as _},
    prelude::{Base, GodotClass, INode, Node, godot_api, godot_dyn},
};

use crate::{
    fighter::{FacingDirection, Fighter2D},
    input::{Action, AnalogMovementFrame, FighterController},
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

    #[export]
    #[init(val = 0.5)]
    pub fastfall_min_strength: real,

    #[export]
    #[init(val = 0.5)]
    pub fastfall_min_diff: real,

    #[export]
    #[init(val = 0.25)]
    pub dash_min_strength: real,

    movement_buffer: VecDeque<AnalogMovementFrame>,

    pub maintaining_jump: bool,

    frame_count: u64,

    action: Option<QueuedAction>,
}

#[godot_dyn]
impl FighterController for FighterControllerPlayer {
    fn preprocess(&mut self, _: &Fighter2D) {}

    fn current_horizontal(&self, _: &Fighter2D) -> Option<real> {
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
    pub fn current_horizontal(&self) -> Option<real> {
        let res = Input::singleton().get_axis(AVATAR_LEFT, AVATAR_RIGHT);
        if res.abs() <= self.movement_deadzone {
            None
        } else {
            Some(res)
        }
    }

    // pub fn current_movement(&self) -> Option<Vector2> {
    //     let res = AnalogMovementFrame::capture(
    //         self.movement_deadzone,
    //         AVATAR_LEFT,
    //         AVATAR_RIGHT,
    //         AVATAR_UP,
    //         AVATAR_DOWN,
    //     )
    //     .0;
    //     if res.is_zero_approx() {
    //         None
    //     } else {
    //         Some(res)
    //     }
    // }

    pub fn peek_action(&self) -> Option<Action> {
        self.action.as_ref().map(|f| f.kind)
    }

    pub fn consume_action(&mut self) -> Option<Action> {
        self.action.take().map(|f| f.kind)
    }

    pub fn wants_fastfall(&self) -> bool {
        fn get_strength(v: Vector2) -> f32 {
            (-v).dot(Vector2::DOWN).clamp(-1.0, 1.0)
        }
        let mut frames = self
            .movement_buffer
            .iter()
            .take(self.fastfall_window.saturating_add(1).saturating_cast())
            .map(|f| f.0);
        let Some(current) = frames.next() else {
            return false;
        };
        let strength = get_strength(current);

        if strength < self.fastfall_min_strength {
            return false;
        }
        for frame in frames {
            if (strength - get_strength(frame)) >= self.fastfall_min_diff {
                return true;
            }
        }
        false
    }

    pub fn should_maintain_dash(&self, dash_dir: FacingDirection) -> bool {
        let Some(strength) = self.current_horizontal() else {
            return false;
        };
        (dash_dir == FacingDirection::Right && strength >= self.dash_min_strength)
            || (dash_dir == FacingDirection::Left && strength <= -self.dash_min_strength)
    }
}

#[godot_api]
impl FighterControllerPlayer {
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
        }
    }

    fn physics_process(&mut self, _delta: f64) {
        // update the movement buffer
        let movement = AnalogMovementFrame::capture(
            self.movement_deadzone,
            AVATAR_LEFT,
            AVATAR_RIGHT,
            AVATAR_DOWN,
            AVATAR_UP,
        );
        self.movement_buffer.push_front(movement);
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

        // check for new actions
        let input = Input::singleton();
        let act = [
            QueuedAction::capture(&input, Action::AttackLight),
            QueuedAction::capture(&input, Action::AttackHeavy),
            QueuedAction::capture(&input, Action::Dash),
            QueuedAction::capture(&input, Action::Jump),
        ]
        .into_iter()
        .fold(None, |acc, right| {
            let Some(left) = acc else { return right };
            let Some(right) = right else {
                return Some(left);
            };
            if left.kind < right.kind {
                Some(right)
            } else {
                Some(left)
            }
        });

        if let Some(action) = act {
            self.action = Some(action);
        }

        // update jump maintanence
        self.maintaining_jump = input.is_action_pressed(AVATAR_JUMP);

        self.frame_count = self.frame_count.wrapping_add(1);
    }
}
