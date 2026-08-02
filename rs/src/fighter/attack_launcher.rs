use godot::classes::class_macros::private::virtuals::ZipReader::Vector2;

use crate::{
    fighter::{Fighter2D, FighterState},
    input::Action,
};

impl Fighter2D {
    pub(super) fn enter_attack_launcher(&mut self) {
        self.state = FighterState::AttackLauncher;

        self.attack_contact = false;
        self.anim_state.bind_mut().reset();

        self.velocity = Vector2::ZERO;

        self.play_anim("attack_launcher");
    }

    pub(super) fn process_attack_launcher(&mut self, delta: f64) {
        if self.can_cancel_attack() {
            match self.peek_action() {
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
