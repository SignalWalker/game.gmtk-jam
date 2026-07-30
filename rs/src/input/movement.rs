use godot::classes::{
    Input,
    class_macros::private::virtuals::ZipReader::{Vector2, real},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AxisDir {
    Positive,
    #[default]
    Neutral,
    Negative,
}

impl AxisDir {
    fn capture(input: &Input, deadzone: real, negative: &str, positive: &str) -> Self {
        let strength = input.get_axis(negative, positive);
        if strength.abs() <= deadzone.abs() {
            Self::Neutral
        } else if deadzone.is_sign_positive() {
            Self::Positive
        } else {
            Self::Negative
        }
    }

    pub const fn from_sign(val: f32) -> Self {
        if val.is_sign_positive() {
            Self::Positive
        } else {
            Self::Negative
        }
    }

    pub const fn to_f32(self) -> f32 {
        match self {
            Self::Positive => 1.0,
            Self::Neutral => 0.0,
            Self::Negative => -1.0,
        }
    }

    pub const fn with_magnitude(self, mag: real) -> real {
        match self {
            Self::Positive => mag,
            Self::Neutral => 0.0,
            Self::Negative => -mag,
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

    pub fn from_vector(deadzone: real, mut vec: Vector2) -> Option<Self> {
        use std::f32::consts::FRAC_PI_2;
        const FRAC_PI_16: f32 = std::f32::consts::FRAC_PI_8 / 2.0;

        let len = vec.length();
        if len <= deadzone.abs() {
            return None;
        }

        // normalize
        vec /= len;

        // angle:
        // - +0 -> +pi/2: up right
        // - +pi/2 -> +pi: up left
        // - -0 -> -pi/2: down right
        // - -pi/2 -> -pi: down left
        let angle = vec.y.atan2(vec.x);
        // okay so imagine the above angle on a circle and then:
        // 1. fold it in half across the X axis
        // 2. rotate it clockwise pi/2 radians so it's halfway across the x axis again
        // 3. fold it in half across the X axis again so it's just the upper right quadrant now
        // 4. horizontally neutral angles are now pi/16 or less
        let fourth = (angle.abs() - FRAC_PI_2).abs();
        let horizontal = if fourth <= FRAC_PI_16 {
            AxisDir::Neutral
        } else if angle.abs() < FRAC_PI_2 {
            AxisDir::Positive
        } else {
            AxisDir::Negative
        };

        // the same trick as above but we rotate it counterclockwise and fold it a third time
        // so that values <= pi/16 are *vertically* neutral
        let vertical = if (fourth - FRAC_PI_2).abs() <= FRAC_PI_16 {
            AxisDir::Neutral
        } else if angle.is_sign_positive() {
            AxisDir::Positive
        } else {
            AxisDir::Negative
        };

        Some(MovementFrame {
            horizontal,
            vertical,
        })
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
