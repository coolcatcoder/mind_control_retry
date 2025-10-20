use crate::plugin_module;
use avian3d::prelude::*;
use bevy::prelude::*;

plugin_module!(pub scene);

pub mod common_properties;

const DEVELOP_OVERRIDE: bool = false;

pub fn plugin(app: &mut App) {
    if DEVELOP_OVERRIDE || crate::DEVELOP {
        app.add_plugins(PhysicsDebugPlugin);
    }
    app.add_plugins((PhysicsPlugins::default(), plugins_in_modules))
        .add_systems(Update, accelerate);
}

// fn pause(mut time: ResMut<Time<Physics>>) {
//     time.pause();
// }

#[derive(PhysicsLayer, Default)]
pub enum CollisionLayer {
    #[default]
    Default,
    Cable,
}

#[derive(Component)]
pub struct Accelerate(pub Vec3);

fn accelerate(query: Query<(&Accelerate, &mut LinearVelocity)>, time: Res<Time>) {
    let time_delta = time.delta_secs();
    for (accelerate, mut linear_velocity) in query {
        **linear_velocity += accelerate.0 * time_delta;
    }
}
