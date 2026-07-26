use crate::{
    fighter::{Fighter2D, FighterState},
    input::Action,
};

impl Fighter2D {
    // [--------] ATTACK AIR HEAVY [--------]

    pub(super) fn enter_attack_air_heavy(&mut self) {
        self.state = FighterState::AttackAirHeavy;

        self.attack_contact = false;
        self.anim_state.bind_mut().reset();

        self.play_anim("attack_air_heavy");
    }

    pub(super) fn process_attack_air_heavy(&mut self, delta: f64) {
        if self.can_cancel_attack() {
            match self.peek_action() {
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

        if self.anim_state.bind().anim_finished {
            self.enter_standing_walking_or_falling(collided, self.get_horizontal_input());
        }
    }
}
