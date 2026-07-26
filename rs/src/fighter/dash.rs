use godot::classes::ICharacterBody2D as _;

use crate::{
    fighter::{Fighter2D, FighterState},
    input::Action,
};

impl Fighter2D {
    // [--------] AIR DASH [--------]

    pub(super) fn enter_airdash(&mut self) {
        self.state = FighterState::AirDash;

        self.jumps_remaining -= 1;
        self.dash_frames_remaining = self.airdash_frames;

        self.velocity = self.facing.to_vec() * self.airdash_speed;

        self.play_anim("dash");
    }

    pub(super) fn process_airdash(&mut self, delta: f64) {
        self.dash_frames_remaining -= 1;

        match self.consume_action() {
            _ => {}
        }

        if !self.should_maintain_dash() {
            self.enter_falling();
            self.process_falling(delta);
            return;
        }

        let collided = self.move_and_slide();

        if self.dash_frames_remaining == 0 {
            // TODO :: dash attack
            self.enter_standing_walking_or_falling(collided, self.get_horizontal_input());
        }
    }

    // [--------] GROUND DASH [--------]

    pub(super) fn enter_ground_dash(&mut self) {
        self.state = FighterState::GroundDash;
        self.dash_frames_remaining = self.ground_dash_frames;

        self.velocity = self.facing.to_vec() * self.ground_dash_speed;

        self.play_anim("dash");
    }

    pub(super) fn process_ground_dash(&mut self, delta: f64) {
        self.dash_frames_remaining -= 1;

        match self.consume_action() {
            Some(Action::Jump) => {
                self.enter_jumping();
                self.process_jumping(delta);
                return;
            }
            _ => {}
        }

        if !self.should_maintain_dash() {
            self.enter_standing_or_walking(self.get_horizontal_input());
            self.physics_process(delta);
            return;
        }

        self.velocity.x = (self.velocity.x.abs() - self.dash_damping)
            .max(self.walk_speed)
            .copysign(self.velocity.x);

        let collided = self.move_and_slide();

        if self.dash_frames_remaining == 0 || self.velocity.x.abs() <= self.walk_speed.abs() {
            self.enter_standing_walking_or_falling(collided, self.get_horizontal_input());
        }
    }
}
