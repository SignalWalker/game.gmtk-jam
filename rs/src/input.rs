use godot::classes::class_macros::private::virtuals::ZipReader::real;

use crate::fighter::{FacingDirection, Fighter2D};

mod movement;
pub use movement::*;

mod player;
pub use player::*;

mod ai;
pub use ai::*;

pub trait FighterController {
    fn preprocess(&mut self, fighter: &Fighter2D);

    fn current_horizontal(&self, fighter: &Fighter2D) -> Option<real>;
    fn consume_action(&mut self, fighter: &Fighter2D) -> Option<Action>;
    fn peek_action(&self, fighter: &Fighter2D) -> Option<Action>;
    fn should_maintain_jump(&self, fighter: &Fighter2D) -> bool;
    fn should_maintain_dash(&self, fighter: &Fighter2D) -> bool;
    fn wants_fastfall(&self, fighter: &Fighter2D) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Action {
    AttackLight,
    AttackHeavy,
    Dash,
    Jump,
}
