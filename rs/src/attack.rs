use std::collections::HashSet;

use godot::{
    classes::{
        Area2D, AudioStream, AudioStreamPlayer, IArea2D,
        class_macros::private::virtuals::{Xrvrs::Gd, ZipReader::Vector2},
    },
    obj::{NewAlloc, WithBaseField, WithUserSignals},
    prelude::{Base, GodotClass, InstanceId, Node, Node2D, Resource, godot_api},
};

use crate::fighter::Fighter2D;

// #[derive(GodotClass)]
// #[class(init, base = Node2D)]
// pub struct Attack2D {
//     base: Base<Node2D>,
// }

pub trait Attackable {
    fn vulnerable(&self, attack: &Gd<Attack2D>) -> bool;
    fn hit(&mut self, attack: &Gd<Attack2D>);
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

    /// The number of iframes this attack will cause on hit
    #[export]
    #[var]
    #[init(val = 0)]
    pub invincibility_frames: u32,

    #[var]
    pub source: Option<Gd<Node>>,

    #[export]
    pub hit_sound: Option<Gd<AudioStream>>,

    #[export]
    pub allow_multi_hits: bool,

    hit_targets: HashSet<InstanceId>,
}

impl Attack2D {
    pub fn accept_hit(&mut self, id: InstanceId) {
        self.hit_targets.insert(id);
    }

    fn on_body_entered(mut attack: Gd<Self>, body: Gd<Node2D>) {
        if let Some(src) = attack.bind().source.as_ref()
            && body.instance_id() == src.instance_id()
        {
            return;
        }
        if let Ok(mut body) = body.try_dynify::<dyn Attackable>() {
            if !body.dyn_bind().vulnerable(&attack) {
                return;
            }

            if !attack.bind().allow_multi_hits {
                let b_id = body.instance_id();
                if attack.bind().hit_targets.contains(&b_id) {
                    return;
                }
                attack.bind_mut().hit_targets.insert(b_id);
            }

            if let Some(mut src) = attack
                .bind()
                .source
                .as_ref()
                .and_then(|src| src.clone().try_cast::<Fighter2D>().ok())
            {
                // play sound
                if let Some(hit_sound) = attack.bind().hit_sound.as_ref() {
                    let mut stream = AudioStreamPlayer::new_alloc();
                    stream.set_stream(hit_sound);
                    stream
                        .signals()
                        .tree_entered()
                        .builder()
                        .connect_self_gd(|mut stream| {
                            stream.set_playing(true);
                        });
                    stream
                        .signals()
                        .finished()
                        .builder()
                        .connect_self_gd(|mut stream| {
                            stream.queue_free();
                        });
                    src.add_child(&stream);
                }

                // inform source that we hit something
                Fighter2D::attack_hit(src, &attack, &body);
            }

            // tell the target that we hit it
            body.dyn_bind_mut().hit(&attack);
        }
    }

    pub fn get_knockback_adjusted(&self) -> Vector2 {
        (self.knockback * self.base().get_scale()).rotated(self.base().get_rotation())
    }
}

#[godot_api]
impl Attack2D {
    #[func]
    pub fn clear_hit_targets(&mut self) {
        self.hit_targets.clear();
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
