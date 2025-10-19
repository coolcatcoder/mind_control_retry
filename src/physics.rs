use crate::{error_handling::ToUnwrapResult, plugin_module};
use avian3d::prelude::*;
use bevy::prelude::*;

plugin_module!(pub scene);

pub mod common_properties;

const DEVELOP_OVERRIDE: bool = true;

pub fn plugin(app: &mut App) {
    if DEVELOP_OVERRIDE || crate::DEVELOP {
        app.add_plugins(PhysicsDebugPlugin);
    }
    app.add_plugins((PhysicsPlugins::default(), plugins_in_modules))
        .add_systems(Startup, pause);
}

fn pause(mut time: ResMut<Time<Physics>>) {
    time.pause();
}

#[derive(PhysicsLayer, Default)]
pub enum CollisionLayer {
    #[default]
    Default,
    Cable,
}
