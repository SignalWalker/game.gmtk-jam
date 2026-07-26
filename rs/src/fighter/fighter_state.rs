use godot::prelude::GodotConvert;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, GodotConvert)]
#[godot(via = u8)]
pub enum FighterState {
    #[default]
    Init,
    Stand,
    Walk,
    GroundDash,
    AirDash,
    Jumping,
    AirJump,
    Falling,
    HitStun,

    // normals
    AttackGroundLight1,
    AttackGroundLight2,
    AttackGroundLight3,

    AttackGroundHeavy,

    AttackAirLight1,
    AttackAirLight2,
    AttackAirLight3,

    AttackAirHeavy,

    // commands
    AttackLauncher,
    AttackStinger,
    AttackRisingSlash,
    AttackDivekick,
    AttackHelmBreaker,
}
