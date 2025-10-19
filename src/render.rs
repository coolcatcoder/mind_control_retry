pub use crate::bevy_prelude::*;
use crate::{frustum_gizmo::ShowFrustumGizmo, machines::player::Player};
use bevy::{
    app::HierarchyPropagatePlugin, core_pipeline::tonemapping::Tonemapping, light::NotShadowCaster,
    post_process::bloom::Bloom,
};

pub mod outlines;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        crate::frustum_gizmo::FrustumGizmoPlugin,
        outlines::plugin,
        HierarchyPropagatePlugin::<SceneNotShadowCaster>::new(Update),
        HierarchyPropagatePlugin::<ComesFromRootEntity>::new(Update),
    ))
    .add_systems(Startup, spawn_camera)
    .add_systems(
        PostUpdate,
        camera_follow.before(TransformSystems::Propagate),
    )
    .insert_resource(AmbientLight {
        brightness: 0.0,
        ..default()
    });
}

/// Stops a gltf scene from casting shadows.
#[derive(PartialEq, Clone, Component)]
#[require(NotShadowCaster)]
pub struct SceneNotShadowCaster;

#[derive(PartialEq, Clone, Component)]
pub struct ComesFromRootEntity(pub Entity);

/// Camera's offset from the controlled character.
const CAMERA_OFFSET: Vec3 = Vec3::new(0., 2.5, 3.);

pub fn spawn_camera(mut commands: Commands, mut clear_colour: ResMut<ClearColor>) {
    let h: f32 = 2.5;
    let w: f32 = 3.0;

    // Designed by the wondrous gibimicro!
    // https://www.desmos.com/calculator/ytzzv3kpca
    let z = (h + w * (1.0 + (2.0_f32.sqrt()))) / 2.0;
    let y = (h * (1.0 + (2.0_f32.sqrt())) + w) / 2.0;

    info!("x:{z},y:{y}");

    clear_colour.0 = Color::BLACK;
    commands.spawn((
        Transform {
            translation: Vec3::new(0., y, z),
            rotation: Quat::from_euler(EulerRot::XYZ, (-45_f32).to_radians(), 0., 0.),
            ..default()
        },
        Camera { ..default() },
        Camera3d { ..default() },
        Tonemapping::TonyMcMapface,
        Bloom::NATURAL,
        Projection::Perspective(PerspectiveProjection {
            fov: 45.0_f32.to_radians(),
            ..default()
        }),
    ));
}

pub fn camera_follow(
    follow: Query<&Transform, With<Player>>,
    mut camera: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    time: Res<Time>,
) {
    let follow = follow.single().else_return()?;
    let mut camera = camera.single_mut().else_error("Could not get camera.")?;

    // Weird.
    //camera.look_at(Vec3::new(follow.translation.x, 1., 0.), Vec3::Y);

    // let camera_no_offset = camera.translation.xz() - CAMERA_OFFSET.xz();

    // let vector_from_camera_to_follow = follow.translation.xz() -
    // camera_no_offset; let amount_to_translate =
    // vector_from_camera_to_follow * (6. * time.delta_secs());

    // let new_xz_translation = camera_no_offset + amount_to_translate +
    // CAMERA_OFFSET.xz();

    // camera.translation.x = new_xz_translation.x;
    // camera.translation.z = new_xz_translation.y;
}
