use godot::{
    classes::{
        AnimationPlayer, CharacterBody2D, ICharacterBody2D, Sprite2D,
        class_macros::private::virtuals::{
            Xrvrs::Gd,
            ZipReader::{Vector2, real},
        },
    },
    obj::{Bounds, WithBaseField as _, bounds::DeclUser},
    prelude::{
        AsDyn, Base, DynGd, GodotClass, Inherits, InstanceId, Node, Node2D, OnReady, godot_api,
        godot_dyn,
    },
};
use godot_utils::ArrayExt;

use crate::{
    attack::{Attack2D, Attackable},
    input::FighterController,
};

// misc
mod util;
// states
mod attack_air_heavy;
mod attack_air_light;
mod attack_dive_kick;
mod attack_ground_heavy;
mod attack_ground_light;
mod attack_helm_breaker;
mod attack_launcher;
mod attack_rising_slash;
mod attack_stinger;
mod dash;
mod ground;
mod hitstun;
mod jump;

mod fighter_state;
pub use fighter_state::*;

mod facing;
pub use facing::*;

mod attack_state;
pub use attack_state::*;

#[derive(GodotClass)]
#[class(init, base = CharacterBody2D)]
pub struct Fighter2D {
    base: Base<CharacterBody2D>,

    #[export]
    #[init(val = 1000)]
    pub max_health: u32,

    pub current_health: u32,

    /// The fighter's maximum walk speed.
    #[export]
    #[init(val = 256.0)]
    pub walk_speed: real,

    /// The maximum speed at which the fighter should fall.
    #[export]
    #[init(val = 512.0)]
    pub terminal_speed: real,

    /// The fighter's fastfall speed.
    #[export]
    #[init(val = 768.0)]
    pub fastfall_speed: real,

    /// Initial vertical speed of jumps.
    #[export]
    #[init(val = 512.0)]
    pub jump_speed: real,

    #[export]
    #[init(val = 448.0)]
    pub airjump_speed: real,

    /// The number of times the fighter can jump in midair.
    #[export]
    #[init(val = 1)]
    pub air_jump_count: u32,

    #[export]
    #[init(val = 24)]
    pub ground_dash_frames: u32,

    #[export]
    #[init(val = 1024.0)]
    pub ground_dash_speed: real,

    #[export]
    #[init(val = 12)]
    pub airdash_frames: u32,

    #[export]
    #[init(val = 1024.0)]
    pub airdash_speed: real,

    /// How many airdashes can be performed before touching the ground
    #[export]
    #[init(val = 1)]
    pub airdash_count: u32,

    /// Pixels per frame to damp horizontal knockback during hitstun.
    #[export]
    #[init(val = 8.0)]
    pub knockback_horizontal_damping: f32,

    /// Pixels per frame to damp air momentum
    #[export]
    #[init(val = 12.0)]
    pub air_damping: f32,

    #[export]
    #[init(val = 6.0)]
    pub dash_damping: f32,

    #[init(node = "%AnimationPlayer")]
    anim: OnReady<Gd<AnimationPlayer>>,
    #[init(node = "%Sprite2D")]
    sprite: OnReady<Gd<Sprite2D>>,
    #[init(node = "%AnimationState")]
    anim_state: OnReady<Gd<AnimationState>>,

    controller: Option<DynGd<Node, dyn FighterController>>,

    /// how many air jumps we have left
    jumps_remaining: u32,

    /// how many frames of dash we have left
    dash_frames_remaining: u32,

    fastfall: bool,

    /// number of frames left of hitstun
    hitstun_frames_remaining: u32,

    /// the number of iframes remaining
    iframes_remaining: u32,

    /// godot why
    pub velocity: Vector2,

    pub state: FighterState,
    pub facing: FacingDirection,

    attack_contact: bool,
}

#[godot_dyn]
impl Attackable for Fighter2D {
    fn vulnerable(&self, _: &Gd<Attack2D>) -> bool {
        self.iframes_remaining == 0
    }

    fn hit(&mut self, attack: &Gd<Attack2D>) {
        self.iframes_remaining = attack.bind().invincibility_frames;
        self.enter_hitstun(
            attack.bind().hitstun_frames,
            attack.bind().get_knockback_adjusted(),
        );
    }
}

impl Fighter2D {
    pub fn register_controller<Controller>(&mut self, controller: Gd<Controller>)
    where
        Controller: Inherits<Node> + AsDyn<dyn FighterController> + Bounds<Declarer = DeclUser>,
    {
        self.controller = Some(controller.into_dyn().upcast::<Node>());
    }

    pub fn can_cancel_attack(&self) -> bool {
        self.attack_contact
            || self.anim_state.bind().cancellable
            || self.anim_state.bind().anim_finished
    }

    pub fn attack_hit(mut fighter: Gd<Self>, _: &Gd<Attack2D>, _: &DynGd<Node2D, dyn Attackable>) {
        fighter.bind_mut().attack_contact = true;
    }

    // [--------] INIT [--------]

    fn process_init(&mut self, delta: f64) {
        self.enter_falling();
        self.process_falling(delta);
    }
}

#[godot_api]
impl Fighter2D {
    #[func]
    fn get_internal_velocity(&self) -> Vector2 {
        self.velocity
    }
}

#[godot_api]
impl ICharacterBody2D for Fighter2D {
    fn enter_tree(&mut self) {
        self.current_health = self.max_health;
    }

    fn ready(&mut self) {
        for mut attack in self
            .base()
            .get_children()
            .into_iter_shared_of_type::<Attack2D>()
        {
            attack.bind_mut().source = Some(self.to_gd().upcast());
        }
    }

    fn physics_process(&mut self, delta: f64) {
        if let Some(mut con) = self.controller.clone() {
            con.dyn_bind_mut().preprocess(self);
        };

        match self.state {
            FighterState::Init => self.process_init(delta),
            FighterState::Stand => self.process_standing(delta),
            FighterState::Walk => self.process_walking(delta),
            FighterState::GroundDash => self.process_ground_dash(delta),
            FighterState::AirDash => self.process_airdash(delta),
            FighterState::Jumping => self.process_jumping(delta),
            FighterState::AirJump => self.process_airjumping(delta),
            FighterState::Falling => self.process_falling(delta),
            FighterState::HitStun => self.process_hitstun(delta),
            // normals
            FighterState::AttackGroundLight1 => self.process_attack_ground_light_1(delta),
            FighterState::AttackGroundLight2 => self.process_attack_ground_light_2(delta),
            FighterState::AttackGroundLight3 => self.process_attack_ground_light_3(delta),
            FighterState::AttackGroundHeavy => self.process_attack_ground_heavy(delta),
            FighterState::AttackAirLight1 => self.process_attack_air_light_1(delta),
            FighterState::AttackAirLight2 => self.process_attack_air_light_2(delta),
            FighterState::AttackAirLight3 => self.process_attack_air_light_3(delta),
            FighterState::AttackAirHeavy => self.process_attack_air_heavy(delta),
            // commands
            FighterState::AttackLauncher => self.process_attack_launcher(delta),
            FighterState::AttackStinger => self.process_attack_stinger(delta),
            FighterState::AttackRisingSlash => self.process_attack_rising_slash(delta),
            FighterState::AttackDivekick => self.process_attack_dive_kick(delta),
            FighterState::AttackHelmBreaker => self.process_attack_helm_breaker(delta),
        }

        self.iframes_remaining = self.iframes_remaining.saturating_sub(1);
    }
}
