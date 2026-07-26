use godot::classes::class_macros::private::virtuals::ZipReader::Vector2;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FacingDirection {
    Left,
    #[default]
    Right,
}

impl FacingDirection {
    pub const fn from_input(input: f32) -> Self {
        if input < 0.0 { Self::Left } else { Self::Right }
    }

    pub const fn to_vec(self) -> Vector2 {
        match self {
            Self::Left => Vector2::LEFT,
            Self::Right => Vector2::RIGHT,
        }
    }
}
