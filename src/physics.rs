use crate::plugin_modules;
use avian3d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

plugin_modules!(pub scene);

pub mod common_properties;

const SHOW_COLLIDERS: bool = false;
const PAUSE: bool = false;

pub fn plugin(app: &mut App) {
    if SHOW_COLLIDERS {
        app.add_plugins(PhysicsDebugPlugin).insert_gizmo_config(
            PhysicsGizmos {
                axis_lengths: None,
                ..default()
            },
            GizmoConfig::default(),
        );
    }
    if PAUSE {
        app.add_systems(Startup, pause);
    }
    app.add_plugins((PhysicsPlugins::default(), plugins_in_modules))
        .add_systems(Update, accelerate);
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

#[derive(Component)]
#[require(Sway, GravityScale(0.))]
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

        //sway.sway = Vec3::ZERO;
        **linear_velocity += (accelerate.0 + sway.sway) * time_delta;
    }
}
