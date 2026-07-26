use crate::{
    fighter::{Fighter2D, FighterState},
    input::Action,
};

impl Fighter2D {
    // [--------] JUMPING [--------]

    pub(super) fn enter_jumping(&mut self) {
        self.state = FighterState::Jumping;

        self.play_anim("jump");

        self.velocity.y = -self.jump_speed;
    }

    pub(super) fn process_jumping(&mut self, delta: f64) {
        let move_input = self.get_horizontal_input();
        if let Some(input) = move_input {
            self.update_facing(input);
        }

        #[allow(clippy::single_match)]
        match self.peek_action() {
            Some(Action::AttackLight) => {
                self.consume_action();
                self.enter_attack_air_light_1();
                self.process_attack_air_light_1(delta);
                return;
            }
            Some(Action::AttackHeavy) => {
                self.consume_action();
                self.enter_attack_air_heavy();
                self.process_attack_air_heavy(delta);
                return;
            }
            Some(Action::Dash) =>
            {
                #[allow(clippy::collapsible_match)]
                if self.jumps_remaining >= 1 && move_input.is_some() {
                    self.consume_action();
                    self.enter_airdash();
                    self.process_airdash(delta);
                    return;
                }
            }
            _ => {}
        }

        self.apply_gravity(delta);

        if self.velocity.y >= 0.0 {
            self.enter_falling();
            self.process_falling(delta);
            return;
        }

        self.apply_walk_input_preserving();

        self.move_and_slide();

        // apply air damping
        self.apply_air_damping();

        // check for whether the player let go of the jump button
        if !self.should_maintain_jump() {
            self.enter_falling();
        }
    }

    // [--------] AIRJUMP [--------]

    pub(super) fn enter_airjump(&mut self) {
        self.state = FighterState::AirJump;

        self.jumps_remaining -= 1;

        self.velocity.y = -self.airjump_speed;

        self.play_anim("jump");
    }

    pub(super) fn process_airjumping(&mut self, delta: f64) {
        let move_input = self.get_horizontal_input();
        if let Some(input) = move_input {
            self.update_facing(input);
        }

        #[allow(clippy::single_match)]
        match self.peek_action() {
            Some(Action::AttackLight) => {
                self.consume_action();
                self.enter_attack_air_light_1();
                self.process_attack_air_light_1(delta);
                return;
            }
            Some(Action::AttackHeavy) => {
                self.consume_action();
                self.enter_attack_air_heavy();
                self.process_attack_air_heavy(delta);
                return;
            }
            Some(Action::Dash) =>
            {
                #[allow(clippy::collapsible_match)]
                if self.jumps_remaining >= 1 && move_input.is_some() {
                    self.consume_action();
                    self.enter_airdash();
                    self.process_airdash(delta);
                    return;
                }
            }
            _ => {}
        }

        self.apply_gravity(delta);

        if self.velocity.y >= 0.0 {
            self.enter_falling();
            self.process_falling(delta);
            return;
        }

        self.apply_walk_input_preserving();

        self.move_and_slide();

        // apply air damping
        self.apply_air_damping();
    }

    // [--------] FALLING [--------]

    pub(super) fn enter_falling(&mut self) {
        self.state = FighterState::Falling;
        self.fastfall = false;
        self.play_anim("fall");
    }

    pub(super) fn process_falling(&mut self, delta: f64) {
        let move_input = self.get_horizontal_input();
        if let Some(input) = move_input {
            self.update_facing(input);
        }

        match self.peek_action() {
            Some(Action::AttackLight) => {
                self.consume_action();
                self.enter_attack_air_light_1();
                self.process_attack_air_light_1(delta);
                return;
            }
            Some(Action::AttackHeavy) => {
                self.consume_action();
                self.enter_attack_air_heavy();
                self.process_attack_air_heavy(delta);
                return;
            }
            Some(Action::Dash) => {
                if self.jumps_remaining >= 1 && move_input.is_some() {
                    self.consume_action();
                    self.enter_airdash();
                    self.process_airdash(delta);
                    return;
                }
            }
            Some(Action::Jump) =>
            {
                #[allow(clippy::collapsible_match)]
                if self.jumps_remaining >= 1 {
                    self.consume_action();
                    self.enter_airjump();
                    self.process_airjumping(delta);
                    return;
                }
            }
            _ => {}
        }

        // check for fastfall
        if !self.fastfall && self.wants_fastfall() {
            self.fastfall = true;
        }

        self.apply_walk_input_preserving();

        if self.fastfall {
            self.velocity.y = self.fastfall_speed;
        } else {
            self.apply_gravity(delta);
        };

        if self.move_and_slide() {
            // we collided with something, so iterate through collisions to see if we hit a floor
            if self.collided_with_floor() {
                self.enter_standing_or_walking(move_input);
            }
        }

        // apply air damping
        self.apply_air_damping();
    }
}
