//! Entry point for this extension from Godot.
//!
//! Explained in more detail [here](https://godot-rust.github.io/book/intro/hello-world.html#rust-entry-point).

use dialogue_engine::ink::InkScriptResourceManager;
use godot::{
    init::{ExtensionLibrary, InitStage, gdextension},
    prelude::godot_error,
};
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

struct RoboExt;

// this attribute generates some extra low-level code that basically sets up some stuff godot
// will call to load the extension
#[gdextension]
unsafe impl ExtensionLibrary for RoboExt {
    fn on_stage_init(stage: InitStage) {
        // set up the bridge from rust logs to the game console
        match stage {
            InitStage::Scene => {
                if let Err(error) = InkScriptResourceManager::register_singleton() {
                    godot_error!("could not initialize ink resource saver/loader: {error}")
                }
            }
            InitStage::Editor => {
                tracing_subscriber::registry()
                    .with(tracing_godot::GodotLayer {})
                    .init();
            }
            _ => (),
        }
    }

    fn on_stage_deinit(stage: InitStage) {
        if let InitStage::Scene = stage {
            match InkScriptResourceManager::unregister_singleton() {
                Ok(singleton) => singleton.free(),
                Err(error) => {
                    godot_error!("could not unregister ink singleton: {error}");
                }
            }
        }
    }
}
