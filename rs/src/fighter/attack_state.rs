use godot::prelude::{Base, GodotClass, Node};

#[derive(GodotClass)]
#[class(init, base = Node)]
pub struct AnimationState {
    base: Base<Node>,

    #[export]
    pub cancellable: bool,

    #[export]
    pub anim_finished: bool,
}

impl AnimationState {
    pub fn reset(&mut self) {
        self.cancellable = false;
        self.anim_finished = false;
    }
}
