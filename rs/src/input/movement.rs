use godot::{
    classes::{
        Input,
        class_macros::private::virtuals::ZipReader::{Vector2, real},
    },
    obj::Singleton as _,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AxisDir {
    Positive,
    #[default]
    Neutral,
    Negative,
}

impl AxisDir {
    fn capture(input: &Input, negative: &str, positive: &str) -> Self {
        match (
            input.is_action_pressed(negative),
            input.is_action_pressed(positive),
        ) {
            (false, false) | (true, true) => Self::Neutral,
            (true, false) => Self::Negative,
            (false, true) => Self::Positive,
        }
    }

    pub const fn to_f32(self) -> f32 {
        match self {
            Self::Positive => 1.0,
            Self::Neutral => 0.0,
            Self::Negative => -1.0,
        }
    }

    pub const fn to_i32(self) -> i32 {
        match self {
            Self::Positive => 1,
            Self::Neutral => 0,
            Self::Negative => -1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct MovementFrame {
    pub vertical: AxisDir,
    pub horizontal: AxisDir,
}
impl MovementFrame {
    pub const NEUTRAL: Self = Self {
        vertical: AxisDir::Neutral,
        horizontal: AxisDir::Neutral,
    };

    pub fn capture(left: &str, right: &str, up: &str, down: &str) -> Self {
        let input = Input::singleton();
        Self {
            vertical: AxisDir::capture(&input, down, up),
            horizontal: AxisDir::capture(&input, left, right),
        }
    }

    pub const fn to_unit_vector(self) -> Option<Vector2> {
        use std::f32::consts::FRAC_1_SQRT_2 as MU;
        match (self.horizontal, self.vertical) {
            (AxisDir::Positive, AxisDir::Positive) => Some(Vector2::new(MU, MU)),
            (AxisDir::Positive, AxisDir::Neutral) => Some(Vector2::RIGHT),
            (AxisDir::Positive, AxisDir::Negative) => Some(Vector2::new(MU, -MU)),
            (AxisDir::Neutral, AxisDir::Positive) => Some(Vector2::UP),
            (AxisDir::Neutral, AxisDir::Neutral) => None,
            (AxisDir::Neutral, AxisDir::Negative) => Some(Vector2::DOWN),
            (AxisDir::Negative, AxisDir::Positive) => Some(Vector2::new(-MU, MU)),
            (AxisDir::Negative, AxisDir::Neutral) => Some(Vector2::LEFT),
            (AxisDir::Negative, AxisDir::Negative) => Some(Vector2::new(-MU, -MU)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct AnalogMovementFrame(pub Vector2);

impl AnalogMovementFrame {
    pub const NEUTRAL: Self = Self(Vector2::ZERO);

    pub fn capture(deadzone: real, left: &str, right: &str, up: &str, down: &str) -> Self {
        let vec = Input::singleton()
            .get_vector_ex(left, right, up, down)
            .deadzone(deadzone)
            .done();
        if vec.length() <= deadzone {
            Self::NEUTRAL
        } else {
            Self(vec)
        }
    }

    pub fn try_normalized(self) -> Option<Vector2> {
        self.0.try_normalized()
    }
}
