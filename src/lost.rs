// TODO: Anything in here must be removed eventually.
use crate::{creatures::tester::Tester, mind_control::Controlled, physics::CollisionLayer};
use avian3d::prelude::CollisionLayers;
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, testing);
}

fn testing(mut commands: Commands) {
    commands.spawn((
        Tester,
        Controlled,
        CollisionLayers::new(
            [CollisionLayer::Default, CollisionLayer::Floor],
            [
                CollisionLayer::Default,
                CollisionLayer::Floor,
                CollisionLayer::Cable,
            ],
        ),
        Transform::from_xyz(10., 5., 0.),
    ));
    commands.spawn((
        Tester,
        Controlled,
        CollisionLayers::new(
            [CollisionLayer::Default, CollisionLayer::Floor],
            [
                CollisionLayer::Default,
                CollisionLayer::Floor,
                CollisionLayer::Cable,
            ],
        ),
        Transform::from_xyz(-5., 0.5, 0.),
    ));
}
