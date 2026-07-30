use std::collections::VecDeque;

use godot::{
    classes::class_macros::private::virtuals::{
        Xrvrs::Gd,
        ZipReader::{Vector2, real},
    },
    obj::WithBaseField as _,
    prelude::{Base, GodotClass, INode, Node, godot_api, godot_dyn},
};
use godot_utils::ArrayExt;

use crate::{
    fighter::{FacingDirection, Fighter2D, FighterState},
    input::{Action, AxisDir, FighterController},
};

#[derive(Debug, Clone, Copy)]
struct FighterSnapshot {
    pub position: Vector2,
    pub velocity: Vector2,
    pub facing: FacingDirection,
    pub state: FighterState,
}

impl FighterSnapshot {
    pub fn capture(opp: &Fighter2D) -> Self {
        Self {
            position: opp.base().get_global_position(),
            velocity: opp.velocity,
            facing: opp.facing,
            state: opp.state,
        }
    }
}

#[derive(GodotClass)]
#[class(init, base = Node)]
pub struct FighterControllerAi {
    base: Base<Node>,

    #[export]
    #[init(val = true)]
    pub enable: bool,

    #[export]
    #[init(val = 12)]
    pub reaction_delay: u32,

    #[export]
    #[init(val = 6)]
    pub self_delay: u32,

    #[export]
    #[init(val = 50.0)]
    pub jump_dist: f32,

    #[export]
    #[init(val = 100.0)]
    pub attack_ground_heavy_dist: f32,

    opp: Option<Gd<Fighter2D>>,

    opp_snapshots: VecDeque<FighterSnapshot>,
    self_snapshots: VecDeque<FighterSnapshot>,
}

impl FighterControllerAi {
    fn is_facing_opp(&self) -> bool {
        let Some(opp) = self.opp_prediction() else {
            return false;
        };
        let Some(fighter) = self.self_prediction() else {
            return false;
        };
        let o_pos = opp.position;
        let f_pos = fighter.position;
        let facing_x = fighter.facing.to_vec().x;

        facing_x.is_sign_positive() == (o_pos.x - f_pos.x).is_sign_positive()
    }

    fn opp_prediction(&self) -> Option<FighterSnapshot> {
        self.opp_snapshots.back().cloned()
    }

    fn self_prediction(&self) -> Option<FighterSnapshot> {
        self.self_snapshots.back().cloned()
    }
}

#[godot_dyn]
impl FighterController for FighterControllerAi {
    fn preprocess(&mut self, fighter: &Fighter2D) {
        self.self_snapshots
            .push_front(FighterSnapshot::capture(fighter));
        if self.self_snapshots.len() > self.self_delay.saturating_cast() {
            self.self_snapshots.pop_back();
        }
    }

    fn current_horizontal(&self, _: &Fighter2D) -> AxisDir {
        if !self.enable {
            return AxisDir::Neutral;
        }

        let Some(o_snap) = self.opp_prediction() else {
            return AxisDir::Neutral;
        };
        let Some(s_snap) = self.self_prediction() else {
            return AxisDir::Neutral;
        };
        let diff = o_snap.position.x - s_snap.position.x;

        if diff.abs() >= self.attack_ground_heavy_dist || !self.is_facing_opp() {
            AxisDir::from_sign(diff)
        } else {
            AxisDir::Neutral
        }
    }

    fn consume_action(&mut self, fighter: &Fighter2D) -> Option<Action> {
        self.peek_action(fighter)
    }

    fn peek_action(&self, _: &Fighter2D) -> Option<Action> {
        if !self.enable {
            return None;
        }

        let snap = self.opp_prediction()?;
        let opp_pos = snap.position;
        let s_snap = self.self_prediction()?;
        let f_pos = s_snap.position;

        let x_diff = opp_pos.x - f_pos.x;
        let y_diff = (-opp_pos.y) - (-f_pos.y);

        match s_snap.state {
            FighterState::Stand
            | FighterState::Walk
            | FighterState::AttackGroundLight1
            | FighterState::AttackGroundLight2 => {
                if y_diff >= self.jump_dist {
                    Some(Action::Jump)
                } else if x_diff.abs() >= self.attack_ground_heavy_dist {
                    Some(Action::Dash)
                } else if self.is_facing_opp() {
                    Some(Action::AttackLight)
                } else {
                    None
                }
            }
            FighterState::AttackGroundLight3 => {
                if y_diff >= self.jump_dist {
                    Some(Action::Jump)
                } else if x_diff.abs() >= self.attack_ground_heavy_dist {
                    Some(Action::Dash)
                } else if self.is_facing_opp() {
                    Some(Action::AttackHeavy)
                } else {
                    None
                }
            }
            FighterState::AttackGroundHeavy => {
                // always try to cancel ground heavies
                if y_diff >= self.jump_dist {
                    Some(Action::Jump)
                } else {
                    Some(Action::Dash)
                }
            }
            FighterState::GroundDash => Some(Action::Jump),
            FighterState::Jumping | FighterState::Falling => {
                if y_diff >= self.jump_dist {
                    Some(Action::Jump)
                } else if x_diff.abs() >= self.attack_ground_heavy_dist {
                    Some(Action::Dash)
                } else {
                    None
                }
            }
            FighterState::AirDash => None,
            _ => None,
        }
    }

    fn should_maintain_jump(&self, _: &Fighter2D) -> bool {
        if !self.enable {
            return false;
        }

        let Some(opp) = self.opp_prediction() else {
            return false;
        };
        let Some(fighter) = self.self_prediction() else {
            return false;
        };

        (-opp.position.y) - (-fighter.position.y) >= self.jump_dist
    }

    fn should_maintain_dash(&self, _: &Fighter2D) -> bool {
        if !self.enable {
            return false;
        }

        let Some(opp) = self.opp_prediction() else {
            return false;
        };
        let Some(fighter) = self.self_prediction() else {
            return false;
        };

        let diff = opp.position.x - fighter.position.x;

        diff.abs() >= self.attack_ground_heavy_dist && self.is_facing_opp()
    }

    fn wants_fastfall(&self, _: &Fighter2D) -> bool {
        if !self.enable {
            return false;
        }

        let Some(opp) = self.opp_prediction() else {
            return false;
        };
        let Some(fighter) = self.self_prediction() else {
            return false;
        };

        (-opp.position.y) - (-fighter.position.y) <= 0.0
    }
}

#[godot_api]
impl INode for FighterControllerAi {
    fn enter_tree(&mut self) {
        if self.base().get_physics_process_priority() >= 0 {
            self.base_mut().set_physics_process_priority(-1);
        }

        // register this controller in the parent
        if let Some(mut p) = self
            .base()
            .get_parent()
            .and_then(|p| p.try_cast::<Fighter2D>().ok())
        {
            p.bind_mut().register_controller(self.to_gd());
        }
    }

    fn ready(&mut self) {
        if let Some(p) = self
            .base()
            .get_parent()
            .and_then(|p| p.try_cast::<Fighter2D>().ok())
            && let Some(p_parent) = p.get_parent()
        {
            for fighter in p_parent
                .get_children()
                .into_iter_shared_of_type::<Fighter2D>()
            {
                if fighter.instance_id() != p.instance_id() {
                    self.opp = Some(fighter);
                    break;
                }
            }
        }
    }

    fn physics_process(&mut self, _delta: f64) {
        if let Some(o_snap) = self
            .opp
            .as_ref()
            .map(|o| FighterSnapshot::capture(&o.bind()))
        {
            self.opp_snapshots.push_front(o_snap);
            if self.opp_snapshots.len() > self.reaction_delay.saturating_cast() {
                self.opp_snapshots.pop_back();
            }
        }
    }
}
