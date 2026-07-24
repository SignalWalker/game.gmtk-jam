use godot::{
    classes::{
        Area2D, IArea2D,
        class_macros::private::virtuals::{Xrvrs::Gd, ZipReader::Vector2},
    },
    obj::{WithBaseField, WithUserSignals},
    prelude::{Base, GodotClass, Node, Node2D, Resource, godot_api},
};

// #[derive(GodotClass)]
// #[class(init, base = Node2D)]
// pub struct Attack2D {
//     base: Base<Node2D>,
// }

pub trait Attackable {
    fn hit(&mut self, attack: &Gd<Attack2D>);
}

#[derive(GodotClass)]
#[class(init, base = Resource)]
pub struct AttackDefinition {
    base: Base<Resource>,

    #[export]
    #[var]
    pub damage: u32,

    #[export]
    #[var]
    #[init(val = Vector2::UP)]
    pub knockback: Vector2,

    /// The number of frames for which targets hit by this attack will be put into hitstun.
    #[export]
    #[var]
    #[init(val = 1)]
    pub hitstun_frames: u32,
}

#[derive(GodotClass)]
#[class(init, base = Area2D)]
pub struct Attack2D {
    base: Base<Area2D>,

    #[export]
    #[var]
    pub damage: u32,

    #[export]
    #[var]
    #[init(val = Vector2::UP)]
    pub knockback: Vector2,

    /// The number of frames for which targets hit by this attack will be put into hitstun.
    #[export]
    #[var]
    #[init(val = 1)]
    pub hitstun_frames: u32,

    #[var]
    pub source: Option<Gd<Node>>,
}

impl Attack2D {
    fn on_body_entered(attack: Gd<Self>, body: Gd<Node2D>) {
        let body_uncast = body.clone();
        if let Ok(mut body) = body.try_dynify::<dyn Attackable>() {
            Self::on_hit(attack.clone(), body_uncast);
            body.dyn_bind_mut().hit(&attack);
        }
    }

    pub fn get_knockback_adjusted(&self) -> Vector2 {
        (self.knockback * self.base().get_scale()).rotated(self.base().get_rotation())
    }
}

#[godot_api]
impl Attack2D {
    #[func(virtual, gd_self)]
    pub fn on_hit(
        #[allow(unused_variables)] attack: Gd<Self>,
        #[allow(unused_variables)] target: Gd<Node2D>,
    ) {
    }
}

#[godot_api]
impl IArea2D for Attack2D {
    fn enter_tree(&mut self) {
        self.signals()
            .body_entered()
            .builder()
            .connect_self_gd(Self::on_body_entered);
    }
}
