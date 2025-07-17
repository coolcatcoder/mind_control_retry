use avian3d::prelude::{Collider, RigidBody};
use bevy::prelude::*;

use crate::{areas::LoadArea, instantiate::Instantiate, machines::{battery::Battery, cable::CableConfig, light::LightBulb}, mouse::drag};

pub fn plugin(_: &mut App) {}

pub fn load(commands: &mut Commands) {
    // floor collision
    commands.spawn((RigidBody::Static, Collider::cuboid(30., 1., 15.), Transform::from_xyz(0., -0.5, 0.)));

    // load area
    commands.spawn((LoadArea, Collider::cuboid(25., 10., 10.), Transform::from_xyz(0., 4., 0.)));

    commands.spawn((Battery::default(), RigidBody::Dynamic, Transform::from_xyz(0., 0.5, -1.))).observe(drag);
    commands.spawn((LightBulb, Transform::from_xyz(3., 0.5, 1.)));

    // Cable test.
    commands.instantiate(CableConfig {
        transform: Transform::from_xyz(0., 5., 0.),
        length: 100,
    });

    commands.instantiate(CableConfig {
        transform: Transform::from_xyz(-10., 5., 3.),
        length: 100,
    });
    commands.instantiate(CableConfig {
        transform: Transform::from_xyz(-10., 7., 2.),
        length: 100,
    });
}