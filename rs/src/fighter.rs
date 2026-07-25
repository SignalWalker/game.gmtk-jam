use godot::{
    classes::{
        AnimationPlayer, CharacterBody2D, ICharacterBody2D, Sprite2D,
        class_macros::private::virtuals::{
            Xrvrs::Gd,
            ZipReader::{Vector2, real},
        },
    },
    obj::{WithBaseField as _, WithUserSignals},
    prelude::{Base, GodotClass, GodotConvert, InstanceId, OnReady, godot_api, godot_dyn},
};
use godot_utils::ArrayExt;

use crate::{
    attack::{Attack2D, Attackable},
    input::{Action, FighterController},
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, GodotConvert)]
#[godot(via = u8)]
pub enum PrimaryState {
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

    controller: Option<Gd<FighterController>>,

    /// how many air jumps we have left
    jumps_remaining: u32,

    /// how many frames of dash we have left
    dash_frames_remaining: u32,

    fastfall: bool,

    /// number of frames left of hitstun
    hitstun_frames_remaining: u32,
    /// if we're in hitstun, this is the id of the attack that caused it
    hitstun_attack: Option<InstanceId>,

    /// the number of iframes remaining
    iframes_remaining: u32,

    pub frame_count: u64,

    /// godot why
    pub velocity: Vector2,

    state: PrimaryState,
    facing: FacingDirection,

    attack_contact: bool,
    attack_frames: u32,

    #[export]
    #[init(val = 45)]
    attack_ground_heavy_frames: u32,
}

#[godot_dyn]
impl Attackable for Fighter2D {
    fn hit(&mut self, attack: &Gd<Attack2D>) {
        let attack_id = attack.instance_id();
        if let PrimaryState::HitStun = self.state
            && self.hitstun_attack.is_some_and(|id| id == attack_id)
        {
            // ignore attacks that've already hit us
            return;
        }
        self.hitstun_attack = Some(attack_id);
        self.enter_hitstun(
            attack.bind().hitstun_frames,
            attack.bind().get_knockback_adjusted(),
        );
    }
}

impl Fighter2D {
    pub fn register_controller(&mut self, controller: Gd<FighterController>) {
        self.controller = Some(controller);
    }

    pub fn move_and_slide(&mut self) -> bool {
        let vel = self.velocity;
        self.base_mut().set_velocity(vel);
        self.base_mut().move_and_slide()
    }

    fn get_horizontal_input(&self) -> Option<real> {
        self.controller
            .as_ref()
            .and_then(|c| c.bind().current_horizontal())
    }

    fn apply_walk_input(&mut self) {
        if let Some(input) = self.get_horizontal_input() {
            self.velocity.x = self.walk_speed.copysign(input);
        }
    }

    fn apply_walk_input_preserving(&mut self) {
        let Some(input) = self.get_horizontal_input() else {
            return;
        };

        let speed = self.walk_speed.copysign(input);

        // if we're trying to move in the opposite direction, do it
        if speed.is_sign_positive() != self.velocity.x.is_sign_positive() {
            self.velocity.x = speed;
        }

        if speed.abs() > self.velocity.x.abs() {
            self.velocity.x = speed;
        }
    }

    fn apply_gravity(&mut self, delta: f64) {
        let grav = (self.base().get_gravity().y as f64 * delta) as f32;
        self.velocity.y = (self.velocity.y + grav).min(self.terminal_speed);
    }

    fn consume_action(&mut self) -> Option<Action> {
        self.controller
            .as_mut()
            .and_then(|c| c.bind_mut().consume_action())
    }

    fn peek_action(&self) -> Option<Action> {
        self.controller
            .as_ref()
            .and_then(|c| c.bind().peek_action())
    }

    fn should_maintain_jump(&self) -> bool {
        self.controller
            .as_ref()
            .is_some_and(|c| c.bind().maintaining_jump)
    }

    fn should_maintain_dash(&self) -> bool {
        self.controller
            .as_ref()
            .is_some_and(|c| c.bind().should_maintain_dash(self.facing))
    }

    fn wants_fastfall(&self) -> bool {
        self.controller
            .as_ref()
            .is_some_and(|c| c.bind().wants_fastfall())
    }

    fn update_facing(&mut self, move_input: real) {
        fn set_attack_scales(fighter: &mut Fighter2D, scale: Vector2) {
            for mut attack in fighter
                .base()
                .get_children()
                .into_iter_shared_of_type::<Attack2D>()
            {
                attack.set_scale(scale);
            }
        }

        self.facing = FacingDirection::from_input(move_input);

        match self.facing {
            FacingDirection::Left => {
                self.sprite.set_flip_h(true);
                set_attack_scales(self, Vector2::new(-1.0, 1.0));
            }
            FacingDirection::Right => {
                self.sprite.set_flip_h(false);
                set_attack_scales(self, Vector2::new(1.0, 1.0));
            }
        }
    }

    fn play_anim(&mut self, name: &str) {
        self.anim.play_ex().name(name).done();
        // ensure the animation starts on this frame
        self.anim.advance(0.0);
    }

    // [--------] INIT [--------]

    fn process_init(&mut self, delta: f64) {
        self.enter_falling();
        self.process_falling(delta);
    }

    // [--------] STANDING [--------]

    fn enter_standing(&mut self) {
        self.state = PrimaryState::Stand;
        self.jumps_remaining = self.air_jump_count;

        self.velocity = Vector2::ZERO;

        tracing::trace!(out_vel = %self.velocity, "stand");

        self.play_anim("idle");

        self.signals().enter_state().emit(PrimaryState::Stand);
    }

    fn process_standing(&mut self, delta: f64) {
        match self.consume_action() {
            Some(Action::Jump) => {
                self.enter_jumping();
                self.process_jumping(delta);
            }
            Some(Action::AttackHeavy) => {
                self.enter_attack_ground_heavy();
                self.process_attack_ground_heavy(delta);
            }
            _ => {
                if self.get_horizontal_input().is_some() {
                    tracing::trace!(vel = %self.velocity, "stand->walk");
                    self.enter_walking();
                    self.process_walking(delta);
                }
            }
        }
    }

    // [--------] WALKING [--------]

    fn enter_walking(&mut self) {
        self.state = PrimaryState::Walk;

        self.play_anim("run");

        self.jumps_remaining = self.air_jump_count;

        tracing::trace!(out_vel = %self.velocity, "walk");

        self.signals().enter_state().emit(PrimaryState::Walk);
    }

    fn process_walking(&mut self, delta: f64) {
        // get input
        let Some(input) = self.get_horizontal_input() else {
            self.enter_standing();
            self.process_standing(delta);
            return;
        };

        // update anim
        self.update_facing(input);

        // check for actions
        match self.consume_action() {
            Some(Action::Jump) => {
                self.enter_jumping();
                self.process_jumping(delta);
                return;
            }
            Some(Action::Dash) => {
                self.enter_ground_dash();
                self.process_ground_dash(delta);
                return;
            }
            Some(Action::AttackHeavy) => {
                self.enter_attack_ground_heavy();
                self.process_attack_ground_heavy(delta);
                return;
            }
            _ => {}
        }

        // no actions, we're walking

        self.apply_walk_input();
        self.velocity.y = 0.0;

        self.move_and_slide();
    }

    // [--------] JUMPING [--------]

    fn enter_jumping(&mut self) {
        self.state = PrimaryState::Jumping;

        self.play_anim("jump");

        tracing::trace!(in_vel = %self.velocity, "jump");
        self.velocity.y = -self.jump_speed;

        self.signals().enter_state().emit(PrimaryState::Jumping);
    }

    fn process_jumping(&mut self, delta: f64) {
        fn start_falling(fighter: &mut Fighter2D) {
            fighter.velocity.y = 0.0;

            fighter.enter_falling();
        }

        let move_input = self.get_horizontal_input();
        if let Some(input) = move_input {
            self.update_facing(input);
        }

        #[allow(clippy::single_match)]
        match self.peek_action() {
            Some(Action::Dash) =>
            {
                #[allow(clippy::collapsible_match)]
                if self.jumps_remaining >= 1 && move_input.is_some() {
                    self.consume_action();
                    self.jumps_remaining -= 1;
                    self.enter_airdash();
                    self.process_airdash(delta);
                    return;
                }
            }
            _ => {}
        }

        self.apply_gravity(delta);

        if self.velocity.y >= 0.0 {
            start_falling(self);
            self.process_falling(delta);
            return;
        }

        self.apply_walk_input_preserving();

        self.move_and_slide();

        // apply air damping
        self.apply_air_damping();

        // check for whether the player let go of the jump button
        if !self.should_maintain_jump() {
            start_falling(self);
        }
    }

    fn apply_air_damping(&mut self) {
        if self.velocity.x.abs() > self.walk_speed.abs() {
            self.velocity.x = (self.velocity.x.abs() - self.air_damping.abs())
                .max(0.0)
                .copysign(self.velocity.x);
        }
    }

    // [--------] AIRJUMP [--------]

    fn enter_airjump(&mut self) {
        self.state = PrimaryState::AirJump;
        self.play_anim("jump");

        tracing::trace!(in_vel = %self.velocity, "airjump");
        self.velocity.y = -self.airjump_speed;

        self.signals().enter_state().emit(PrimaryState::AirJump);
    }

    fn process_airjumping(&mut self, delta: f64) {
        fn start_falling(fighter: &mut Fighter2D) {
            fighter.velocity.y = 0.0;

            fighter.enter_falling();
        }

        let move_input = self.get_horizontal_input();
        if let Some(input) = move_input {
            self.update_facing(input);
        }

        #[allow(clippy::single_match)]
        match self.peek_action() {
            Some(Action::Dash) =>
            {
                #[allow(clippy::collapsible_match)]
                if self.jumps_remaining >= 1 && move_input.is_some() {
                    self.consume_action();
                    self.jumps_remaining -= 1;
                    self.enter_airdash();
                    self.process_airdash(delta);
                    return;
                }
            }
            _ => {}
        }

        self.apply_gravity(delta);

        if self.velocity.y >= 0.0 {
            start_falling(self);
            self.process_falling(delta);
            return;
        }

        self.apply_walk_input_preserving();

        self.move_and_slide();

        // apply air damping
        self.apply_air_damping();
    }

    // [--------] FALLING [--------]

    fn enter_falling(&mut self) {
        self.state = PrimaryState::Falling;
        self.fastfall = false;
        self.play_anim("fall");

        tracing::trace!(in_vel = %self.velocity, "fall");

        self.signals().enter_state().emit(PrimaryState::Falling);
    }

    fn process_falling(&mut self, delta: f64) {
        let move_input = self.get_horizontal_input();
        if let Some(input) = move_input {
            self.update_facing(input);
        }

        match self.peek_action() {
            Some(Action::Dash) => {
                if self.jumps_remaining >= 1 && move_input.is_some() {
                    self.consume_action();
                    self.jumps_remaining -= 1;
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
                    self.jumps_remaining -= 1;
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

    fn enter_standing_or_walking(&mut self, input: Option<real>) {
        if input.is_some() {
            self.enter_walking()
        } else {
            self.enter_standing()
        }
    }

    fn collided_with_floor(&self) -> bool {
        let range = 0..self.base().get_slide_collision_count();
        for col in range.filter_map(|i| self.base().get_slide_collision(i)) {
            // did we hit a floor?
            if col.get_angle() < std::f32::consts::FRAC_PI_3 {
                // we hit a floor...
                return true;
            }
        }
        false
    }

    fn enter_standing_walking_or_falling(&mut self, collided: bool, move_input: Option<real>) {
        if collided && self.collided_with_floor() {
            self.enter_standing_or_walking(move_input)
        } else {
            self.enter_falling()
        }
    }

    // [--------] AIR DASH [--------]

    fn enter_airdash(&mut self) {
        self.state = PrimaryState::AirDash;
        self.dash_frames_remaining = self.airdash_frames;
        self.play_anim("dash");

        self.velocity = self.facing.to_vec() * self.airdash_speed;

        tracing::trace!(out_vel = %self.velocity, "airdash");

        self.signals().enter_state().emit(PrimaryState::AirDash);
    }

    fn process_airdash(&mut self, delta: f64) {
        self.dash_frames_remaining -= 1;

        match self.consume_action() {
            _ => {}
        }

        if !self.should_maintain_dash() {
            self.enter_falling();
            self.process_falling(delta);
            return;
        }

        let collided = self.move_and_slide();

        if self.dash_frames_remaining == 0 {
            // TODO :: dash attack
            self.enter_standing_walking_or_falling(collided, self.get_horizontal_input());
        }
    }

    // [--------] GROUND DASH [--------]

    fn enter_ground_dash(&mut self) {
        self.state = PrimaryState::GroundDash;
        self.dash_frames_remaining = self.ground_dash_frames;

        self.play_anim("dash");

        self.velocity = self.facing.to_vec() * self.ground_dash_speed;

        tracing::trace!(out_vel = %self.velocity, "ground dash");

        self.signals().enter_state().emit(PrimaryState::GroundDash);
    }

    fn process_ground_dash(&mut self, delta: f64) {
        self.dash_frames_remaining -= 1;

        match self.consume_action() {
            Some(Action::Jump) => {
                self.enter_jumping();
                self.process_jumping(delta);
                return;
            }
            _ => {}
        }

        if !self.should_maintain_dash() {
            self.enter_standing_or_walking(self.get_horizontal_input());
            self.physics_process(delta);
            return;
        }

        self.velocity.x = (self.velocity.x.abs() - self.dash_damping)
            .max(self.walk_speed)
            .copysign(self.velocity.x);

        let collided = self.move_and_slide();

        if self.dash_frames_remaining == 0 || self.velocity.x.abs() <= self.walk_speed.abs() {
            self.enter_standing_walking_or_falling(collided, self.get_horizontal_input());
        }
    }

    // [--------] HITSTUN [--------]

    fn enter_hitstun(&mut self, frames: u32, knockback: Vector2) {
        self.state = PrimaryState::HitStun;
        self.hitstun_frames_remaining = frames;
        self.velocity = knockback;
        self.play_anim("hitstun");

        tracing::trace!(out_vel = %self.velocity, "hitstun");

        self.signals().enter_state().emit(PrimaryState::HitStun);
    }

    fn process_hitstun(&mut self, delta: f64) {
        self.hitstun_frames_remaining = self.hitstun_frames_remaining.saturating_sub(1);

        self.velocity.x = (self.velocity.x.abs() - self.knockback_horizontal_damping)
            .max(0.0)
            .copysign(self.velocity.x);
        self.apply_gravity(delta);

        let collided = self.move_and_slide();

        if self.hitstun_frames_remaining == 0 {
            self.hitstun_attack = None;
            self.enter_standing_walking_or_falling(collided, self.get_horizontal_input());
        }
    }

    // [--------] ATTACK GROUND HEAVY [--------]

    fn enter_attack_ground_heavy(&mut self) {
        self.state = PrimaryState::AttackGroundHeavy;

        self.attack_contact = false;
        self.attack_frames = 0;

        self.velocity = Vector2::ZERO;

        self.play_anim("attack_ground_heavy");

        self.signals()
            .enter_state()
            .emit(PrimaryState::AttackGroundHeavy)
    }

    fn process_attack_ground_heavy(&mut self, _delta: f64) {
        self.attack_frames += 1;

        if self.attack_frames > self.attack_ground_heavy_frames {
            self.enter_standing_or_walking(self.get_horizontal_input());
        }
    }
}

#[godot_api]
impl Fighter2D {
    #[signal]
    fn enter_state(state: PrimaryState);

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
        match self.state {
            PrimaryState::Init => self.process_init(delta),
            PrimaryState::Stand => self.process_standing(delta),
            PrimaryState::Walk => self.process_walking(delta),
            PrimaryState::GroundDash => self.process_ground_dash(delta),
            PrimaryState::AirDash => self.process_airdash(delta),
            PrimaryState::Jumping => self.process_jumping(delta),
            PrimaryState::AirJump => self.process_airjumping(delta),
            PrimaryState::Falling => self.process_falling(delta),
            PrimaryState::HitStun => self.process_hitstun(delta),
            // normals
            PrimaryState::AttackGroundLight1 => todo!(),
            PrimaryState::AttackGroundLight2 => todo!(),
            PrimaryState::AttackGroundLight3 => todo!(),
            PrimaryState::AttackGroundHeavy => self.process_attack_ground_heavy(delta),
            PrimaryState::AttackAirLight1 => todo!(),
            PrimaryState::AttackAirLight2 => todo!(),
            PrimaryState::AttackAirLight3 => todo!(),
            PrimaryState::AttackAirHeavy => todo!(),
            // commands
            PrimaryState::AttackLauncher => todo!(),
            PrimaryState::AttackStinger => todo!(),
            PrimaryState::AttackRisingSlash => todo!(),
            PrimaryState::AttackDivekick => todo!(),
            PrimaryState::AttackHelmBreaker => todo!(),
        }
        self.frame_count = self.frame_count.wrapping_add(1);
    }
}
