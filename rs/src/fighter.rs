use godot::{
    classes::{
        AnimationPlayer, CharacterBody2D, ICharacterBody2D, Sprite2D,
        class_macros::private::virtuals::{
            Xrvrs::{Gd, math::FloatExt},
            ZipReader::{Vector2, real},
        },
    },
    obj::WithBaseField as _,
    prelude::{Base, GodotClass, InstanceId, OnReady, godot_api, godot_dyn},
};

use crate::{
    attack::{Attack2D, Attackable},
    input::{Action, FighterController},
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
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
    #[init(val = 12.0)]
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

    state: PrimaryState,
    facing: FacingDirection,
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

    fn get_horizontal_input(&self) -> Option<real> {
        self.controller
            .as_ref()
            .and_then(|c| c.bind().current_horizontal())
    }

    const fn horizontal_movement_to_velocity(&self, input: f32) -> real {
        if input < 0.0 {
            -self.walk_speed
        } else {
            self.walk_speed
        }
    }

    fn get_horizontal_velocity(&self) -> real {
        self.get_horizontal_input()
            .map(|i| self.horizontal_movement_to_velocity(i))
            .unwrap_or(0.0)
    }

    fn get_horizontal_velocity_with_min_mag(&self, min: real) -> real {
        let res = self.get_horizontal_velocity();
        res.abs().max(min.abs()).copysign(res)
    }

    fn get_velocity_plus_gravity(&mut self, c_vel: Vector2, delta: f64) -> Vector2 {
        let grav = (self.base().get_gravity().y as f64 * delta) as f32;
        let g_vel = (grav + c_vel.y).min(self.terminal_speed);
        Vector2::new(c_vel.x, g_vel)
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
        self.facing = FacingDirection::from_input(move_input);

        match self.facing {
            FacingDirection::Left => self.sprite.set_flip_h(true),
            FacingDirection::Right => self.sprite.set_flip_h(false),
        }
    }

    // [--------] INIT [--------]

    fn process_init(&mut self, delta: f64) {
        self.enter_falling();
        self.process_falling(delta);
    }

    // [--------] STANDING [--------]

    fn enter_standing(&mut self) {
        self.state = PrimaryState::Stand;
        self.anim.play_ex().name("idle").done();
        self.jumps_remaining = self.air_jump_count;
        self.base_mut().set_velocity(Vector2::ZERO);
    }

    fn process_standing(&mut self, delta: f64) {
        match self.consume_action() {
            Some(Action::Jump) => {
                self.enter_jumping();
                self.process_jumping(delta);
            }
            _ => {
                if self.get_horizontal_input().is_some() {
                    self.enter_walking();
                    self.process_walking(delta);
                }
            }
        }
    }

    // [--------] WALKING [--------]

    fn update_walk_anim(&mut self) {
        self.anim.play_ex().name("run").done();
    }

    fn enter_walking(&mut self) {
        self.state = PrimaryState::Walk;
        self.update_walk_anim();

        self.jumps_remaining = self.air_jump_count;
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
        self.update_walk_anim();

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
            _ => {}
        }

        // no actions, we're walking

        let vel = self.horizontal_movement_to_velocity(input);
        self.base_mut().set_velocity(Vector2::new(vel, 0.0));
        if self.base_mut().move_and_slide() {
            // TODO :: handle collisions
        }
    }

    // [--------] JUMPING [--------]

    fn enter_jumping(&mut self) {
        self.state = PrimaryState::Jumping;
        self.anim.play_ex().name("jump").done();

        let mut vel = self.base().get_velocity();
        vel.y = -self.jump_speed;
        self.base_mut().set_velocity(vel);
    }

    fn process_jumping(&mut self, delta: f64) {
        fn start_falling(fighter: &mut Fighter2D) {
            let mut vel = fighter.base().get_velocity();
            vel.y = 0.0;
            fighter.base_mut().set_velocity(vel);

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

        let grav_y = (self.base().get_gravity().y as f64 * delta) as f32;
        let c_vel = self.base().get_velocity();
        let vel_y = c_vel.y + grav_y;

        if vel_y >= 0.0 {
            start_falling(self);
            self.process_falling(delta);
            return;
        }

        let vel_x = self.get_horizontal_velocity_with_min_mag(c_vel.x);

        let mut vel = Vector2::new(vel_x, vel_y);

        self.base_mut().set_velocity(vel);
        if self.base_mut().move_and_slide() {
            // TODO :: jump collisions
        }

        // apply air damping
        self.apply_air_damping(&mut vel);
        self.base_mut().set_velocity(vel);

        // check for whether the player let go of the jump button
        if !self.should_maintain_jump() {
            start_falling(self);
        }
    }

    fn apply_air_damping(&self, vel: &mut Vector2) {
        if vel.x.abs() > self.walk_speed {
            vel.x = (vel.x.abs() - self.air_damping)
                .clamp(0.0, self.walk_speed)
                .copysign(vel.x);
        }
    }

    // [--------] AIRJUMP [--------]

    fn enter_airjump(&mut self) {
        self.state = PrimaryState::AirJump;
        self.anim.play_ex().name("jump").done();

        let mut vel = self.base().get_velocity();
        vel.y = -self.airjump_speed;
        self.base_mut().set_velocity(vel);
    }

    fn process_airjumping(&mut self, delta: f64) {
        fn start_falling(fighter: &mut Fighter2D) {
            let mut vel = fighter.base().get_velocity();
            vel.y = 0.0;
            fighter.base_mut().set_velocity(vel);

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

        let grav_y = (self.base().get_gravity().y as f64 * delta) as f32;
        let c_vel = self.base().get_velocity();
        let vel_y = c_vel.y + grav_y;

        if vel_y >= 0.0 {
            start_falling(self);
            self.process_falling(delta);
            return;
        }

        let vel_x = self.get_horizontal_velocity_with_min_mag(c_vel.x);
        let mut vel = Vector2::new(vel_x, vel_y);

        self.base_mut().set_velocity(vel);
        if self.base_mut().move_and_slide() {
            // TODO :: jump collisions
        }

        // apply air damping
        self.apply_air_damping(&mut vel);
        self.base_mut().set_velocity(vel);
    }

    // [--------] FALLING [--------]

    fn enter_falling(&mut self) {
        self.state = PrimaryState::Falling;
        self.fastfall = false;
        self.anim.play_ex().name("fall").done();
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

        let c_vel = self.base().get_velocity();
        let move_x = self.get_horizontal_velocity_with_min_mag(c_vel.x);

        let mut new_vel = if self.fastfall {
            Vector2::new(move_x, self.fastfall_speed)
        } else {
            self.get_velocity_plus_gravity(Vector2::new(move_x, c_vel.y), delta)
        };

        self.base_mut().set_velocity(new_vel);
        if self.base_mut().move_and_slide() {
            // we collided with something, so iterate through collisions to see if we hit a floor
            if self.collided_with_floor() {
                self.enter_standing_or_walking(move_input);
            }
        }

        // apply air damping
        self.apply_air_damping(&mut new_vel);
        self.base_mut().set_velocity(new_vel);
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
        self.anim.play_ex().name("dash").done();
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

        let vel = self.facing.to_vec() * self.airdash_speed;
        self.base_mut().set_velocity(vel);
        let collided = self.base_mut().move_and_slide();

        if self.dash_frames_remaining == 0 {
            // TODO :: dash attack
            self.enter_standing_walking_or_falling(collided, self.get_horizontal_input());
        }
    }

    // [--------] GROUND DASH [--------]

    fn enter_ground_dash(&mut self) {
        self.state = PrimaryState::GroundDash;
        self.dash_frames_remaining = self.ground_dash_frames;
        self.anim.play_ex().name("dash").done();
        let vel = self.facing.to_vec() * self.ground_dash_speed;
        self.base_mut().set_velocity(vel);
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

        let mut c_vel = self.base().get_velocity();
        c_vel.x = (c_vel.x.abs() - self.dash_damping)
            .max(self.walk_speed)
            .copysign(c_vel.x);

        self.base_mut().set_velocity(c_vel);
        let collided = self.base_mut().move_and_slide();

        if self.dash_frames_remaining == 0 || c_vel.x.abs() <= self.walk_speed {
            self.enter_standing_walking_or_falling(collided, self.get_horizontal_input());
        }
    }

    // [--------] HITSTUN [--------]

    fn enter_hitstun(&mut self, frames: u32, knockback: Vector2) {
        self.state = PrimaryState::HitStun;
        self.hitstun_frames_remaining = frames;
        self.base_mut().set_velocity(knockback);
        self.anim.play_ex().name("hitstun").done();
    }

    fn process_hitstun(&mut self, delta: f64) {
        self.hitstun_frames_remaining = self.hitstun_frames_remaining.saturating_sub(1);

        let c_vel = self.base().get_velocity();
        let new_x = (c_vel.x.abs() - self.knockback_horizontal_damping)
            .max(0.0)
            .copysign(c_vel.x);
        let new_vel = self.get_velocity_plus_gravity(Vector2::new(new_x, c_vel.y), delta);

        self.base_mut().set_velocity(new_vel);
        let collided = self.base_mut().move_and_slide();

        if self.hitstun_frames_remaining == 0 {
            self.hitstun_attack = None;
            self.enter_standing_walking_or_falling(collided, self.get_horizontal_input());
        }
    }
}

#[godot_api]
impl Fighter2D {
    #[signal]
    fn enter_state(state: PrimaryState);
}

#[godot_api]
impl ICharacterBody2D for Fighter2D {
    fn enter_tree(&mut self) {
        self.current_health = self.max_health;
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
            PrimaryState::AttackGroundHeavy => todo!(),
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
