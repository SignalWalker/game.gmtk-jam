use godot::{
    classes::Area2D,
    prelude::{Base, GodotClass, Node2D},
};

#[derive(GodotClass)]
#[class(init, base = Node2D)]
pub struct Attack2D {
    base: Base<Node2D>,
}

#[derive(GodotClass)]
#[class(init, base = Area2D)]
pub struct HitArea2D {
    base: Base<Area2D>,
}
