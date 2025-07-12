use avian3d::prelude::*;
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_plugins((PhysicsPlugins::default(), PhysicsDebugPlugin::default()))
        .add_systems(Startup, create_floor);
}

#[derive(PhysicsLayer, Default)]
pub enum CollisionLayer {
    #[default]
    Default,
    Floor,
}

#[derive(Component)]
struct Floor;

fn create_floor(mut commands: Commands) {
    commands.spawn((
        Floor,
        RigidBody::Static,
        Collider::half_space(Vec3::Y),
        CollisionLayers::new(CollisionLayer::Floor, CollisionLayer::Floor),
    ));
}
