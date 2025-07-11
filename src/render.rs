use crate::{error_handling::ToFailure, mind_control::Controlled};
use avian3d::prelude::*;
use bevy::prelude::*;
use core::f32;

pub mod outlines;

pub fn plugin(app: &mut App) {
    app.add_plugins(outlines::plugin)
        .add_systems(Startup, (spawn_camera, spawn_map, spawn_light))
        .add_systems(
            PostUpdate,
            move_camera_to_controlled.before(TransformSystem::TransformPropagate),
        )
        .insert_resource(AmbientLight {
            brightness: 0.0,
            ..default()
        });
}

/// No one fully understands this. Be careful.
const CAMERA_OFFSET: Vec3 = Vec3::new(0., 10., 13.);

pub fn spawn_camera(mut commands: Commands, mut clear_colour: ResMut<ClearColor>) {
    clear_colour.0 = Color::BLACK;
    commands.spawn((
        Transform::from_translation(CAMERA_OFFSET).looking_at(Vec3::ZERO, Vec3::Y),
        Camera { ..default() },
        Camera3d { ..default() },
    ));
}

pub fn spawn_light(mut commands: Commands) {
    commands.spawn((
        SpotLight {
            intensity: 40_000.0, // lumens
            color: Color::WHITE,
            shadows_enabled: true,
            inner_angle: f32::consts::PI / 3.0 * 0.85,
            outer_angle: f32::consts::PI / 3.0,
            ..default()
        },
        Transform::from_xyz(0., 3., 0.).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
    ));
}

pub fn spawn_map(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(SceneRoot(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("map/tutorial_base.glb")),
    ));

    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(200., 1., 200.),
        Transform::from_xyz(0., -0.5, 0.),
    ));
}

pub fn move_camera_to_controlled(
    controlled: Option<Single<&Transform, With<Controlled>>>,
    mut camera: Single<&mut Transform, (With<Camera>, Without<Controlled>)>,
    time: Res<Time>,
) -> Result {
    let controlled_translation = controlled.else_return()?.translation.xz();
    let mut camera_translation =
        camera.translation.xz() - Vec2::new(CAMERA_OFFSET.x, CAMERA_OFFSET.z);
    camera_translation.smooth_nudge(&controlled_translation, 10., time.delta_secs());
    camera.translation.x = camera_translation.x + CAMERA_OFFSET.x;
    camera.translation.z = camera_translation.y + CAMERA_OFFSET.z;
    Ok(())
}
