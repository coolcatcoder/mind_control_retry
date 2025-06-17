use crate::{error_handling::ToFailure, mind_control::Controlled};
use avian3d::prelude::*;
use bevy::{pbr::light_consts::lux, prelude::*, render::camera::ScalingMode};

mod outlines;

pub fn plugin(app: &mut App) {
    app.add_plugins(outlines::plugin)
        .add_systems(Startup, (spawn_camera, spawn_map, spawn_light))
        .add_systems(
            PostUpdate,
            move_camera_to_controlled.before(TransformSystem::TransformPropagate),
        );
}

/// No one fully understand this. Be careful.
const CAMERA_OFFSET: f32 = 50.;

pub fn spawn_camera(mut commands: Commands, mut clear_colour: ResMut<ClearColor>) {
    clear_colour.0 = Srgba::new(0.2, 0.5, 0.9, 1.).into();
    commands.spawn((
        Transform::from_xyz(-CAMERA_OFFSET, CAMERA_OFFSET, CAMERA_OFFSET)
            .looking_at(Vec3::ZERO, Vec3::Y),
        Camera { ..default() },
        Camera3d { ..default() },
        Projection::Orthographic(OrthographicProjection {
            scale: 1.,
            near: -1000.,
            far: 1000.,
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 25.,
            },
            ..OrthographicProjection::default_3d()
        }),
    ));
}

pub fn spawn_light(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.98, 0.95, 0.82),
            illuminance: lux::AMBIENT_DAYLIGHT,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0., 0., 0.).looking_at(Vec3::new(0.1, -0.05, -0.1), Vec3::Y),
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
    let mut camera_translation = camera.translation.xz() - Vec2::new(-CAMERA_OFFSET, CAMERA_OFFSET);
    camera_translation.smooth_nudge(&controlled_translation, 10., time.delta_secs());
    camera.translation.x = camera_translation.x - CAMERA_OFFSET;
    camera.translation.z = camera_translation.y + CAMERA_OFFSET;
    Ok(())
}
