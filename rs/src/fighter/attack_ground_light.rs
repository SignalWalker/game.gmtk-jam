use godot::classes::class_macros::private::virtuals::ZipReader::Vector2;

use crate::{
    fighter::{Fighter2D, FighterState},
    input::Action,
};

impl Fighter2D {
    // [--------] ATTACK GROUND LIGHT 1 [--------]

    pub(super) fn enter_attack_ground_light_1(&mut self) {
        self.state = FighterState::AttackGroundLight1;

        self.attack_contact = false;
        self.anim_state.bind_mut().reset();

        self.velocity = Vector2::ZERO;

        self.play_anim("attack_ground_light_1");
    }

    pub(super) fn process_attack_ground_light_1(&mut self, delta: f64) {
        if self.can_cancel_attack() {
            match self.peek_action() {
                Some(Action::AttackLight) => {
                    self.consume_action();
                    self.enter_attack_ground_light_2();
                    self.process_attack_ground_light_2(delta);
                    return;
                }
                Some(Action::AttackHeavy) => {
                    self.consume_action();
                    self.enter_attack_ground_heavy();
                    self.process_attack_ground_heavy(delta);
                    return;
                }
                Some(Action::Jump) => {
                    self.consume_action();
                    self.enter_jumping();
                    self.process_jumping(delta);
                    return;
                }
                Some(Action::Dash) => {
                    self.consume_action();
                    self.enter_ground_dash();
                    self.process_ground_dash(delta);
                    return;
                }
                _ => {}
            }
        }

        if self.attack_anim_finished() {
            self.enter_standing_or_walking(self.get_horizontal_input());
        }
    }

    // [--------] ATTACK GROUND LIGHT 2 [--------]

    fn enter_attack_ground_light_2(&mut self) {
        self.state = FighterState::AttackGroundLight2;

        self.attack_contact = false;
        self.anim_state.bind_mut().reset();

        self.velocity = Vector2::ZERO;

        self.play_anim("attack_ground_light_2");
    }

    pub(super) fn process_attack_ground_light_2(&mut self, delta: f64) {
        if self.can_cancel_attack() {
            match self.peek_action() {
                Some(Action::AttackLight) => {
                    self.consume_action();
                    self.enter_attack_ground_light_3();
                    self.process_attack_ground_light_3(delta);
                    return;
                }
                Some(Action::AttackHeavy) => {
                    self.consume_action();
                    self.enter_attack_ground_heavy();
                    self.process_attack_ground_heavy(delta);
                    return;
                }
                Some(Action::Jump) => {
                    self.consume_action();
                    self.enter_jumping();
                    self.process_jumping(delta);
                    return;
                }
                Some(Action::Dash) => {
                    self.consume_action();
                    self.enter_ground_dash();
                    self.process_ground_dash(delta);
                    return;
                }
                _ => {}
            }
        }

        if self.attack_anim_finished() {
            self.enter_standing_or_walking(self.get_horizontal_input());
        }
    }

    // [--------] ATTACK GROUND LIGHT 3 [--------]

    fn enter_attack_ground_light_3(&mut self) {
        self.state = FighterState::AttackGroundLight3;

        self.attack_contact = false;
        self.anim_state.bind_mut().reset();

        self.velocity = Vector2::ZERO;

        self.play_anim("attack_ground_light_3");
    }

    pub(super) fn process_attack_ground_light_3(&mut self, delta: f64) {
        if self.can_cancel_attack() {
            match self.peek_action() {
                Some(Action::AttackHeavy) => {
                    self.consume_action();
                    self.enter_attack_ground_heavy();
                    self.process_attack_ground_heavy(delta);
                    return;
                }
                Some(Action::Jump) => {
                    self.consume_action();
                    self.enter_jumping();
                    self.process_jumping(delta);
                    return;
                }
                Some(Action::Dash) => {
                    self.consume_action();
                    self.enter_ground_dash();
                    self.process_ground_dash(delta);
                    return;
                }
                _ => {}
            }
        }

        if self.attack_anim_finished() {
            self.enter_standing_or_walking(self.get_horizontal_input());
        }
    }
}
