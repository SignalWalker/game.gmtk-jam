use godot::{
    prelude::{Base, GodotClass, Node},
    tools::get_autoload_by_name,
};

pub fn started_game_normally() -> bool {
    get_autoload_by_name::<GlobalStateNode>("GlobalState")
        .bind()
        .started_game_normally
}

#[derive(GodotClass)]
#[class(init, base = Node)]
pub struct GlobalStateNode {
    base: Base<Node>,

    #[var]
    pub started_game_normally: bool,
}
