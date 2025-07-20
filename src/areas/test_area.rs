use avian3d::prelude::{Collider, RigidBody};
use bevy::prelude::*;

use crate::{areas::LoadArea, instantiate::InstantiateInto, machines::{battery::BatteryConfig, cable::CableConfig, light::LightBulb}, mouse::drag};

pub fn plugin(_: &mut App) {}

pub fn load(commands: &mut Commands) {
    // floor collision
    commands.spawn((RigidBody::Static, Collider::cuboid(30., 1., 15.), Transform::from_xyz(0., -0.5, 0.)));

    // load area
    commands.spawn((LoadArea, Collider::cuboid(25., 10., 10.), Transform::from_xyz(0., 4., 0.)));

    commands.spawn((RigidBody::Dynamic, Transform::from_xyz(0., 0.5, -1.))).observe(drag).instantiate(BatteryConfig {charge: 50});
    //commands.spawn((Battery::default(), RigidBody::Dynamic, Transform::from_xyz(0., 0.5, -1.))).observe(drag);
    commands.spawn((LightBulb, Transform::from_xyz(3., 0.5, 1.)));

    // Cable test.
    commands.spawn(Transform::from_xyz(0., 5., 0.)).instantiate(CableConfig {
        length: 100,
    });

    commands.spawn(Transform::from_xyz(-10., 5., 3.)).instantiate(CableConfig {
        length: 100,
    });
    commands.spawn(Transform::from_xyz(-10., 7., 2.)).instantiate(CableConfig {
        length: 100,
    });
}