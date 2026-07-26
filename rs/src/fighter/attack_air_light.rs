use godot::classes::class_macros::private::virtuals::ZipReader::Vector2;

use crate::{
    fighter::{Fighter2D, FighterState},
    input::Action,
};

impl Fighter2D {
    // [--------] ATTACK AIR LIGHT 1 [--------]

    pub(super) fn enter_attack_air_light_1(&mut self) {
        self.state = FighterState::AttackAirLight1;

        self.attack_contact = false;
        self.anim_state.bind_mut().reset();

        self.velocity = Vector2::ZERO;

        self.play_anim("attack_air_light_1");
    }

    pub(super) fn process_attack_air_light_1(&mut self, delta: f64) {
        if self.can_cancel_attack() {
            match self.peek_action() {
                Some(Action::AttackLight) => {
                    self.consume_action();
                    self.enter_attack_air_light_2();
                    self.process_attack_air_light_2(delta);
                    return;
                }
                Some(Action::AttackHeavy) => {
                    self.consume_action();
                    self.enter_attack_air_heavy();
                    self.process_attack_air_heavy(delta);
                    return;
                }
                Some(Action::Jump) => {
                    if self.jumps_remaining >= 1 {
                        self.consume_action();
                        self.enter_airjump();
                        self.process_jumping(delta);
                        return;
                    }
                }
                Some(Action::Dash) => {
                    if self.jumps_remaining >= 1 {
                        self.consume_action();
                        self.enter_airdash();
                        self.process_airdash(delta);
                        return;
                    }
                }
                _ => {}
            }
        }

        let collided = self.apply_movement_for_aerial_attack(delta);

        if collided && self.collided_with_floor() {
            self.enter_standing_or_walking(self.get_horizontal_input());
        }

        if self.attack_anim_finished() {
            self.enter_standing_walking_or_falling(collided, self.get_horizontal_input());
        }
    }

    // [--------] ATTACK AIR LIGHT 2 [--------]

    pub(super) fn enter_attack_air_light_2(&mut self) {
        self.state = FighterState::AttackAirLight2;

        self.attack_contact = false;
        self.anim_state.bind_mut().reset();

        self.velocity = Vector2::ZERO;

        self.play_anim("attack_air_light_2");
    }

    pub(super) fn process_attack_air_light_2(&mut self, delta: f64) {
        if self.can_cancel_attack() {
            match self.peek_action() {
                Some(Action::AttackLight) => {
                    self.consume_action();
                    self.enter_attack_air_light_3();
                    self.process_attack_air_light_3(delta);
                    return;
                }
                Some(Action::AttackHeavy) => {
                    self.consume_action();
                    self.enter_attack_air_heavy();
                    self.process_attack_air_heavy(delta);
                    return;
                }
                Some(Action::Jump) => {
                    if self.jumps_remaining >= 1 {
                        self.consume_action();
                        self.enter_airjump();
                        self.process_jumping(delta);
                        return;
                    }
                }
                Some(Action::Dash) => {
                    if self.jumps_remaining >= 1 {
                        self.consume_action();
                        self.enter_airdash();
                        self.process_airdash(delta);
                        return;
                    }
                }
                _ => {}
            }
        }

        let collided = self.apply_movement_for_aerial_attack(delta);

        if collided && self.collided_with_floor() {
            self.enter_standing_or_walking(self.get_horizontal_input());
        }

        if self.attack_anim_finished() {
            self.enter_standing_walking_or_falling(collided, self.get_horizontal_input());
        }
    }

    // [--------] ATTACK AIR LIGHT 3 [--------]

    pub(super) fn enter_attack_air_light_3(&mut self) {
        self.state = FighterState::AttackAirLight3;

        self.attack_contact = false;
        self.anim_state.bind_mut().reset();

        self.velocity = Vector2::ZERO;

        self.play_anim("attack_air_light_3");
    }

    pub(super) fn process_attack_air_light_3(&mut self, delta: f64) {
        if self.can_cancel_attack() {
            match self.peek_action() {
                Some(Action::AttackHeavy) => {
                    self.consume_action();
                    self.enter_attack_air_heavy();
                    self.process_attack_air_heavy(delta);
                    return;
                }
                Some(Action::Jump) => {
                    if self.jumps_remaining >= 1 {
                        self.consume_action();
                        self.enter_airjump();
                        self.process_jumping(delta);
                        return;
                    }
                }
                Some(Action::Dash) => {
                    if self.jumps_remaining >= 1 {
                        self.consume_action();
                        self.enter_airdash();
                        self.process_airdash(delta);
                        return;
                    }
                }
                _ => {}
            }
        }

        let collided = self.apply_movement_for_aerial_attack(delta);

        if collided && self.collided_with_floor() {
            self.enter_standing_or_walking(self.get_horizontal_input());
        }

        if self.attack_anim_finished() {
            self.enter_standing_walking_or_falling(collided, self.get_horizontal_input());
        }
    }
}
