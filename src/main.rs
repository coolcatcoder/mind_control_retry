#![warn(clippy::pedantic)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::type_complexity)]
#![warn(clippy::unwrap_used)]
#![allow(clippy::needless_for_each)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::too_many_lines)]

use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    ecs::error::GLOBAL_ERROR_HANDLER,
    prelude::*,
};

mod areas;
mod controls;
mod creatures;
mod error_handling;
mod instantiate;
mod lost;
mod machines;
mod mind_control;
mod mouse;
mod physics;
mod propagate;
mod render;
mod sync;

const FPS_DEBUG: bool = true;

fn main() {
    if GLOBAL_ERROR_HANDLER
        .set(error_handling::error_handler)
        .is_err()
    {
        eprintln!("Failed to set error handler. Defaulting to panicking.");
    }

    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins,
        render::plugin,
        controls::plugin,
        lost::plugin,
        creatures::plugin,
        mind_control::plugin,
        machines::plugin,
        mouse::plugin,
        sync::plugin,
        physics::plugin,
        areas::plugin,
        instantiate::plugin,
    ));

    if FPS_DEBUG {
        app.add_plugins(FpsOverlayPlugin {
            config: FpsOverlayConfig {
                text_config: TextFont {
                    font_size: 42.0,
                    ..default()
                },
                text_color: Srgba::GREEN.into(),
                refresh_interval: core::time::Duration::from_millis(100),
                enabled: true,
            },
        });
    }

    app.run();
}
