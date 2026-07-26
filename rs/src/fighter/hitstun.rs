use godot::classes::class_macros::private::virtuals::ZipReader::Vector2;

use crate::fighter::{Fighter2D, FighterState};

impl Fighter2D {
    // [--------] HITSTUN [--------]

    pub(super) fn enter_hitstun(&mut self, frames: u32, knockback: Vector2) {
        self.state = FighterState::HitStun;
        self.hitstun_frames_remaining = frames;
        self.velocity = knockback;
        self.play_anim("hitstun");
    }

    pub(super) fn process_hitstun(&mut self, delta: f64) {
        self.hitstun_frames_remaining = self.hitstun_frames_remaining.saturating_sub(1);

        self.velocity.x = (self.velocity.x.abs() - self.knockback_horizontal_damping)
            .max(0.0)
            .copysign(self.velocity.x);
        self.apply_gravity(delta);

        let collided = self.move_and_slide();

        if self.hitstun_frames_remaining == 0 {
            self.enter_standing_walking_or_falling(collided, self.get_horizontal_input());
        }
    }
}
