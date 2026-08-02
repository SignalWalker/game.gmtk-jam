use godot::{
    classes::class_macros::private::virtuals::ZipReader::{Vector2, real},
    obj::WithBaseField as _,
};
use godot_utils::ArrayExt as _;

use crate::{
    attack::Attack2D,
    fighter::{FacingDirection, Fighter2D},
    input::{Action, AxisDir},
};

impl Fighter2D {
    pub(super) fn move_and_slide(&mut self) -> bool {
        let vel = self.velocity;
        self.base_mut().set_velocity(vel);
        self.base_mut().move_and_slide()
    }

    pub(super) fn get_horizontal_input(&self) -> AxisDir {
        self.controller
            .as_ref()
            .map_or(AxisDir::Neutral, |c| c.dyn_bind().current_horizontal(self))
    }

    pub(super) fn apply_walk_input(&mut self) {
        let input = self.get_horizontal_input();
        if input != AxisDir::Neutral {
            self.velocity.x = input.with_magnitude(self.walk_speed);
        }
    }

    pub(super) fn apply_walk_input_preserving(&mut self) {
        let input = self.get_horizontal_input();
        let speed = match input {
            AxisDir::Positive => self.walk_speed,
            AxisDir::Neutral => return,
            AxisDir::Negative => -self.walk_speed,
        };

        // if we're trying to move in the opposite direction, do it
        if speed.is_sign_positive() != self.velocity.x.is_sign_positive()
            || speed.abs() > self.velocity.x.abs()
        {
            self.velocity.x = speed;
        }
    }

    pub(super) fn apply_gravity(&mut self, delta: f64) {
        let grav = (self.base().get_gravity().y as f64 * delta) as f32;
        self.velocity.y = (self.velocity.y + grav).min(self.terminal_speed);
    }

    pub(super) fn apply_air_damping(&mut self) {
        if self.velocity.x.abs() > self.walk_speed.abs() {
            self.velocity.x = (self.velocity.x.abs() - self.air_damping.abs())
                .max(0.0)
                .copysign(self.velocity.x);
        }
    }

    pub(super) fn consume_action(&mut self) -> Option<Action> {
        self.controller
            .clone()
            .and_then(|mut c| c.dyn_bind_mut().consume_action(self))
    }

    pub(super) fn peek_action(&self) -> Option<Action> {
        self.controller
            .as_ref()
            .and_then(|c| c.dyn_bind().peek_action(self))
    }

    pub(super) fn should_maintain_jump(&self) -> bool {
        self.controller
            .as_ref()
            .is_some_and(|c| c.dyn_bind().should_maintain_jump(self))
    }

    pub(super) fn should_maintain_dash(&self) -> bool {
        self.controller
            .as_ref()
            .is_some_and(|c| c.dyn_bind().should_maintain_dash(self))
    }

    pub(super) fn wants_fastfall(&self) -> bool {
        self.controller
            .as_ref()
            .is_some_and(|c| c.dyn_bind().wants_fastfall(self))
    }

    pub(super) fn update_facing(&mut self, input: AxisDir) {
        fn set_attack_scales(fighter: &mut Fighter2D, scale: Vector2) {
            for mut attack in fighter
                .base()
                .get_children()
                .into_iter_shared_of_type::<Attack2D>()
            {
                attack.set_scale(scale);
            }
        }

        match input {
            AxisDir::Positive => {
                self.facing = FacingDirection::Right;
                self.sprite.set_flip_h(false);
                set_attack_scales(self, Vector2::new(1.0, 1.0));
            }
            AxisDir::Negative => {
                self.facing = FacingDirection::Left;
                self.sprite.set_flip_h(true);
                set_attack_scales(self, Vector2::new(-1.0, 1.0));
            }
            _ => {}
        }
    }

    pub(super) fn play_anim(&mut self, name: &str) {
        self.anim.play_ex().name(name).done();
        // ensure the animation starts on this frame
        self.anim.advance(0.0);
    }

    pub(super) fn attack_anim_finished(&self) -> bool {
        self.anim_state.bind().anim_finished
    }

    pub(super) fn apply_movement_for_aerial_attack(&mut self, delta: f64) -> bool {
        let mut collided = false;
        if !self.attack_contact {
            self.apply_walk_input_preserving();
            self.apply_gravity(delta);
            collided = self.move_and_slide();
            self.apply_air_damping();
        }
        collided
    }

    pub(super) fn enter_standing_or_walking(&mut self, input: AxisDir) {
        match input {
            AxisDir::Neutral => self.enter_standing(),
            _ => self.enter_walking(),
        }
    }

    pub(super) fn collided_with_floor(&self) -> bool {
        let range = 0..self.base().get_slide_collision_count();
        for col in range.filter_map(|i| self.base().get_slide_collision(i)) {
            // did we hit a floor?
            if col.get_angle() < std::f32::consts::FRAC_PI_3 {
                // we hit a floor...
                return true;
            }
        }
        false
    }

    pub(super) fn enter_standing_walking_or_falling(&mut self, collided: bool, input: AxisDir) {
        if collided && self.collided_with_floor() {
            self.enter_standing_or_walking(input)
        } else {
            self.enter_falling()
        }
    }
}
