use godot::{
    classes::{
        Area2D, CharacterBody2D, IArea2D,
        class_macros::private::virtuals::{Xrvrs::Gd, ZipReader::Vector2},
    },
    prelude::{Base, GodotClass, Node, godot_api},
};

// #[derive(GodotClass)]
// #[class(init, base = Node2D)]
// pub struct Attack2D {
//     base: Base<Node2D>,
// }

pub trait Attackable: GodotClass {
    fn hit(target: &Gd<Self>, attack: &Gd<Attack2D>);
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
    #[init(val = 0)]
    pub hitstun_frames: u32,

    #[var]
    pub source: Option<Gd<Node>>,
}

#[godot_api]
impl Attack2D {
    #[signal]
    fn hit();

    #[func(virtual, gd_self)]
    pub fn on_hit(
        #[allow(unused_variables)] attack: Gd<Self>,
        #[allow(unused_variables)] target: Gd<CharacterBody2D>,
    ) {
    }
}

#[godot_api]
impl IArea2D for Attack2D {}
