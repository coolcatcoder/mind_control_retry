use crate::plugin_module;
use avian3d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

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
#[require(Sway)]
pub struct Accelerate(pub Vec3);

#[derive(Default, Component)]
pub struct Sway {
    seconds_remaining: f32,
    sway: Vec3,
}

fn accelerate(query: Query<(&Accelerate, &mut Sway, &mut LinearVelocity)>, time: Res<Time>) {
    let time_delta = time.delta_secs();
    let mut rng = rand::rng();
    for (accelerate, mut sway, mut linear_velocity) in query {
        if sway.seconds_remaining <= 0. {
            sway.sway = Vec3::new(
                rng.random_range(-0.3..0.3),
                rng.random_range(0.0..0.3),
                rng.random_range(-0.3..0.3),
            );
            sway.seconds_remaining = rng.random_range(5.0..15.0);
            // sway.seconds_remaining = rng.random_range(0.5..2.0);
            // sway.sway = Vec3::new(rng.random_range(-1.0..1.0),
            // rng.random_range(0.0..0.3), rng.random_range(-1.0..1.0));
        } else {
            sway.seconds_remaining -= time_delta;
        }

        **linear_velocity += (accelerate.0 + sway.sway) * time_delta;
    }
}
