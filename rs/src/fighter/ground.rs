use godot::classes::class_macros::private::virtuals::ZipReader::Vector2;

use crate::{
    fighter::{Fighter2D, FighterState},
    input::Action,
};

impl Fighter2D {
    // [--------] STANDING [--------]

    pub(super) fn enter_standing(&mut self) {
        self.state = FighterState::Stand;
        self.jumps_remaining = self.air_jump_count;

        self.velocity = Vector2::ZERO;

        self.play_anim("idle");
    }

    pub(super) fn process_standing(&mut self, delta: f64) {
        match self.consume_action() {
            Some(Action::Jump) => {
                self.enter_jumping();
                self.process_jumping(delta);
            }
            Some(Action::AttackLight) => {
                self.enter_attack_ground_light_1();
                self.process_attack_ground_light_1(delta);
            }
            Some(Action::AttackHeavy) => {
                self.enter_attack_ground_heavy();
                self.process_attack_ground_heavy(delta);
            }
            _ => {
                if self.get_horizontal_input().is_some() {
                    self.enter_walking();
                    self.process_walking(delta);
                }
            }
        }
    }

    // [--------] WALKING [--------]

    pub(super) fn enter_walking(&mut self) {
        self.state = FighterState::Walk;

        self.play_anim("run");

        self.jumps_remaining = self.air_jump_count;
    }

    pub(super) fn process_walking(&mut self, delta: f64) {
        // get input
        let Some(input) = self.get_horizontal_input() else {
            self.enter_standing();
            self.process_standing(delta);
            return;
        };

        // update anim
        self.update_facing(input);

        // check for actions
        match self.consume_action() {
            Some(Action::Jump) => {
                self.enter_jumping();
                self.process_jumping(delta);
                return;
            }
            Some(Action::Dash) => {
                self.enter_ground_dash();
                self.process_ground_dash(delta);
                return;
            }
            Some(Action::AttackLight) => {
                self.enter_attack_ground_light_1();
                self.process_attack_ground_light_1(delta);
                return;
            }
            Some(Action::AttackHeavy) => {
                self.enter_attack_ground_heavy();
                self.process_attack_ground_heavy(delta);
                return;
            }
            _ => {}
        }

        // no actions, we're walking

        self.apply_walk_input();
        self.velocity.y = 0.0;

        self.move_and_slide();
    }
}
