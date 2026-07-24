use godot::{
    classes::{
        AnimatedSprite2D, CharacterBody2D, ICharacterBody2D, Input,
        class_macros::private::virtuals::{
            Xrvrs::Gd,
            ZipReader::{Vector2, real},
        },
    },
    obj::{NewAlloc, Singleton as _, WithBaseField as _},
    prelude::{Base, GodotClass, InstanceId, OnReady, godot_api, godot_dyn},
};

use crate::attack::{Attack2D, Attackable};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PrimaryState {
    #[default]
    Init,
    Stand,
    Walk,
    Run,
    GroundDash,
    AirDash,
    JumpSquat,
    Jumping,
    Falling,
    Sleep,
    HitStun,
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

    pub const fn to_dash_anim(self) -> &'static str {
        match self {
            Self::Left => "dash_left",
            Self::Right => "dash_right",
        }
    }
}

pub const AVATAR_UP: &str = "avatar_move_up";
pub const AVATAR_DOWN: &str = "avatar_move_down";
pub const AVATAR_JUMP: &str = "avatar_jump";
pub const AVATAR_DASH: &str = "avatar_dash";

#[derive(GodotClass)]
#[class(init, base = CharacterBody2D)]
pub struct Fighter2D {
    base: Base<CharacterBody2D>,

    #[export]
    #[init(val = 1000)]
    pub max_health: u32,

    pub current_health: u32,

    /// The minimum input vector length.
    #[export]
    #[init(val = 0.2)]
    pub movement_inner_deadzone: real,

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

    /// The number of times the fighter can jump in midair.
    #[export]
    #[init(val = 1)]
    pub air_jump_count: u32,

    #[export]
    #[init(val = 12)]
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

    #[export]
    #[init(val = Vector2::UP * 256.0)]
    pub hitstun_test_knockback: Vector2,

    #[init(node = "%Sprite")]
    sprite: OnReady<Gd<AnimatedSprite2D>>,

    /// how many air jumps we have left
    jumps_remaining: u32,

    /// how many frames of dash we have left
    dash_frames_remaining: u32,

    fastfall: bool,

    /// number of frames left of hitstun
    hitstun_frames_remaining: u32,
    /// if we're in hitstun, this is the id of the attack that caused it
    hitstun_attack: Option<InstanceId>,

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
    /// Negative: Left, Positive: Right
    fn raw_movement_input() -> real {
        const AVATAR_LEFT: &str = "avatar_move_left";
        const AVATAR_RIGHT: &str = "avatar_move_right";
        godot::classes::Input::singleton().get_axis(AVATAR_LEFT, AVATAR_RIGHT)
    }

    fn get_movement_input(&self) -> Option<real> {
        let input = Self::raw_movement_input();
        if input.abs() < self.movement_inner_deadzone {
            None
        } else {
            Some(input)
        }
    }

    fn movement_input_to_velocity(&self, input: f32) -> Vector2 {
        Vector2::new(
            if input < 0.0 {
                -self.walk_speed
            } else {
                self.walk_speed
            },
            0.0,
        )
    }

    fn get_movement_velocity(&self) -> Vector2 {
        self.get_movement_input()
            .map(|i| self.movement_input_to_velocity(i))
            .unwrap_or(Vector2::ZERO)
    }

    fn get_velocity_plus_gravity(&mut self, c_vel: Vector2, delta: f64) -> Vector2 {
        let grav = (self.base().get_gravity().y as f64 * delta) as f32;
        let g_vel = (grav + c_vel.y).min(self.terminal_speed);
        Vector2::new(c_vel.x, g_vel)
    }

    // [--------] INIT [--------]

    fn process_init(&mut self, delta: f64) {
        self.enter_falling();
        self.process_falling(delta);
    }

    // [--------] STANDING [--------]

    fn enter_standing(&mut self) {
        tracing::trace!("enter_standing");
        self.state = PrimaryState::Stand;
        self.sprite.play_ex().name("stand").done();
        self.jumps_remaining = self.air_jump_count;
    }

    fn process_standing(&mut self, _delta: f64) {
        if Input::singleton().is_action_pressed(AVATAR_JUMP) {
            self.enter_jumping();
        } else if let Some(input) = self.get_movement_input() {
            self.facing = FacingDirection::from_input(input);
            self.enter_walking();
        }
    }

    // [--------] WALKING [--------]

    fn update_walk_anim(&mut self) {
        let anim_name = match self.facing {
            FacingDirection::Left => "walk_left",
            FacingDirection::Right => "walk_right",
        };
        self.sprite.play_ex().name(anim_name).done();
    }

    fn enter_walking(&mut self) {
        tracing::trace!("enter_walking");
        self.state = PrimaryState::Walk;
        self.update_walk_anim();

        self.jumps_remaining = self.air_jump_count;
    }

    fn process_walking(&mut self, delta: f64) {
        if Input::singleton().is_action_pressed(AVATAR_JUMP) {
            self.enter_jumping();
            return;
        }
        if let Some(input) = self.get_movement_input() {
            self.process_walking_with_input(delta, input);
        } else {
            self.enter_standing();
        }
    }

    fn process_walking_with_input(&mut self, delta: f64, input: f32) {
        self.facing = FacingDirection::from_input(input);
        // update anim
        self.update_walk_anim();

        if Input::singleton().is_action_just_pressed(AVATAR_DASH) {
            self.enter_ground_dash();
            self.process_ground_dash(delta);
            return;
        }

        let vel = self.movement_input_to_velocity(input);
        self.base_mut().set_velocity(vel);
        if self.base_mut().move_and_slide() {
            // TODO :: handle collisions
        }
    }

    // [--------] JUMPING [--------]

    fn enter_jumping(&mut self) {
        tracing::trace!("enter_jumping");
        self.state = PrimaryState::Jumping;
        self.sprite.play_ex().name("jump").done();

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

        if !Input::singleton().is_action_pressed(AVATAR_JUMP) {
            return start_falling(self);
        }

        let move_input = self.get_movement_input();

        if self.jumps_remaining >= 1
            && Input::singleton().is_action_just_pressed(AVATAR_DASH)
            && let Some(move_input) = move_input
        {
            self.jumps_remaining -= 1;
            self.facing = FacingDirection::from_input(move_input);
            self.enter_airdash();
            self.process_airdash(delta);
            return;
        }

        // self.jump_countdown = self.jump_countdown.saturating_sub(1);
        // if self.jump_countdown == 0 {
        //     return start_falling(self);
        // }

        let grav_y = (self.base().get_gravity().y as f64 * delta) as f32;
        let c_vel_y = self.base().get_velocity().y;
        let vel_y = c_vel_y + grav_y;

        if vel_y >= 0.0 {
            return start_falling(self);
        }

        let move_vel = self.get_movement_velocity();
        self.base_mut()
            .set_velocity(Vector2::new(move_vel.x, vel_y));
        if self.base_mut().move_and_slide() {
            // TODO :: jump collisions
        }
    }

    // [--------] FALLING [--------]

    fn enter_falling(&mut self) {
        tracing::trace!("enter_falling");
        self.state = PrimaryState::Falling;
        self.fastfall = false;
        self.sprite.play_ex().name("fall").done();
    }

    fn process_falling(&mut self, delta: f64) {
        if self.jumps_remaining >= 1 && Input::singleton().is_action_just_pressed(AVATAR_JUMP) {
            self.jumps_remaining -= 1;
            self.enter_jumping();
            self.process_jumping(delta);
            return;
        }

        let move_input = self.get_movement_input();

        if self.jumps_remaining >= 1
            && Input::singleton().is_action_just_pressed(AVATAR_DASH)
            && let Some(move_input) = move_input
        {
            self.jumps_remaining -= 1;
            self.facing = FacingDirection::from_input(move_input);
            self.enter_airdash();
            self.process_airdash(delta);
            return;
        }

        if !self.fastfall
            && Input::singleton().is_action_just_pressed(AVATAR_DOWN)
            && move_input.is_none_or(|i| i <= self.movement_inner_deadzone)
        {
            self.fastfall = true;
        }

        let move_x = move_input
            .map(|i| self.movement_input_to_velocity(i).x)
            .unwrap_or(0.0);

        let new_vel = if self.fastfall {
            Vector2::new(move_x, self.fastfall_speed)
        } else {
            let c_vel = self.base().get_velocity();
            self.get_velocity_plus_gravity(Vector2::new(move_x, c_vel.y), delta)
        };

        self.base_mut().set_velocity(new_vel);
        if self.base_mut().move_and_slide() {
            let range = 0..self.base().get_slide_collision_count();
            for col in range.filter_map(|i| self.base().get_slide_collision(i)) {
                if col.get_angle() < std::f32::consts::FRAC_PI_3 {
                    if move_input.is_some() {
                        self.enter_walking();
                    } else {
                        self.enter_standing();
                    }
                    break;
                }
            }
        }
    }

    // [--------] AIR DASH [--------]

    fn enter_airdash(&mut self) {
        tracing::trace!("enter_airdash");
        self.state = PrimaryState::AirDash;
        self.dash_frames_remaining = self.airdash_frames;
        self.sprite
            .play_ex()
            .name(self.facing.to_dash_anim())
            .done();
    }

    fn process_airdash(&mut self, _delta: f64) {
        self.dash_frames_remaining -= 1;

        let vel = self.facing.to_vec() * self.airdash_speed;
        self.base_mut().set_velocity(vel);
        if self.base_mut().move_and_slide() {
            // TODO :: collision
        }

        if self.dash_frames_remaining == 0 {
            self.enter_falling();
        }
    }

    // [--------] GROUND DASH [--------]

    fn enter_ground_dash(&mut self) {
        tracing::trace!("enter_ground_dash");
        self.state = PrimaryState::GroundDash;
        self.dash_frames_remaining = self.ground_dash_frames;
        self.sprite
            .play_ex()
            .name(self.facing.to_dash_anim())
            .done();
    }

    fn process_ground_dash(&mut self, _delta: f64) {
        self.dash_frames_remaining -= 1;

        let vel = self.facing.to_vec() * self.ground_dash_speed;
        self.base_mut().set_velocity(vel);
        if self.base_mut().move_and_slide() {
            // TODO :: collision
        }

        if self.dash_frames_remaining == 0 {
            if self.get_movement_input().is_some() {
                self.enter_walking();
            } else {
                self.enter_standing();
            }
        }
    }

    // [--------] HITSTUN [--------]

    fn enter_hitstun(&mut self, frames: u32, knockback: Vector2) {
        self.state = PrimaryState::HitStun;
        self.hitstun_frames_remaining = frames;
        self.base_mut().set_velocity(knockback);
        self.sprite.play_ex().name("hitstun").done();
    }

    fn process_hitstun(&mut self, delta: f64) {
        self.hitstun_frames_remaining = self.hitstun_frames_remaining.saturating_sub(1);

        let c_vel = self.base().get_velocity();
        let new_x = (c_vel.x.abs() - self.knockback_horizontal_damping)
            .max(0.0)
            .copysign(c_vel.x);
        let new_vel = self.get_velocity_plus_gravity(Vector2::new(new_x, c_vel.y), delta);

        self.base_mut().set_velocity(new_vel);
        if self.base_mut().move_and_slide() {
            // TODO :: collisions?
        }

        if self.hitstun_frames_remaining == 0 {
            self.hitstun_attack = None;
            self.enter_falling()
        }
    }
}

#[godot_api]
impl ICharacterBody2D for Fighter2D {
    fn enter_tree(&mut self) {
        self.current_health = self.max_health;
    }

    fn physics_process(&mut self, delta: f64) {
        if Input::singleton().is_action_just_pressed("hitstun_test") {
            self.run_deferred(|fighter| {
                let mut attack = Attack2D::new_alloc();
                attack.bind_mut().hitstun_frames = 60;
                attack.bind_mut().knockback = fighter.hitstun_test_knockback;
                attack.bind_mut().damage = 0;
                if let FacingDirection::Left = fighter.facing {
                    attack.set_scale(Vector2::new(-1.0, 1.0));
                }
                fighter.hit(&attack);
                attack.free();
            });
        }
        match self.state {
            PrimaryState::Init => self.process_init(delta),
            PrimaryState::Stand => self.process_standing(delta),
            PrimaryState::Walk => self.process_walking(delta),
            PrimaryState::Run => todo!(),
            PrimaryState::GroundDash => self.process_ground_dash(delta),
            PrimaryState::AirDash => self.process_airdash(delta),
            PrimaryState::JumpSquat => todo!(),
            PrimaryState::Jumping => self.process_jumping(delta),
            PrimaryState::Falling => self.process_falling(delta),
            PrimaryState::Sleep => todo!(),
            PrimaryState::HitStun => self.process_hitstun(delta),
        }
    }
}
