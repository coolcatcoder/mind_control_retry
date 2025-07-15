#![warn(clippy::pedantic)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::type_complexity)]
#![warn(clippy::unwrap_used)]
#![allow(clippy::needless_for_each)]
#![allow(clippy::too_many_arguments)]

use bevy::{ecs::error::GLOBAL_ERROR_HANDLER, prelude::*};

mod areas;
mod controls;
mod creatures;
mod error_handling;
mod lost;
mod machines;
mod mind_control;
mod mouse;
mod physics;
mod propagate;
mod render;
mod sync;

fn main() {
    if GLOBAL_ERROR_HANDLER
        .set(error_handling::error_handler)
        .is_err()
    {
        eprintln!("Failed to set error handler. Defaulting to panicking.");
    }
    App::new()
        .add_plugins((
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
        ))
        .run();
}
