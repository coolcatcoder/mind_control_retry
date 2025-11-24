use crate::{
    areas::{Area, AreaLoadedEntity},
    chain::{FUNGUS_PURPLE_GLOW, fungus_a, player_body},
    controls::{Action, Actions},
    physics::{Accelerate, CollisionLayer},
};
use avian3d::prelude::*;
pub use bevy::prelude::*;

mod maze {
    use super::*;

    pub fn load(on: On<AreaLoadedEntity>, loaded: Query<&Name>, mut commands: Commands) {
        let name = loaded
            .get(on.loaded)
            .else_error("Could not get components on loaded entity.")?;

        if name.contains("box") {
            commands
                .entity(on.loaded)
                .insert((RigidBody::Static, Collider::cuboid(0.4, 0.3, 0.4)));
        }
    }
}

mod edge_mushrooms {
    use super::*;

    pub fn load(
        on: On<AreaLoadedEntity>,
        loaded: Query<(&Name, &Transform, &Children)>,
        area_transforms: Query<&Transform>,
        stem: Query<(&Transform, &Children)>,
        cap_transforms: Query<&Transform, Without<Mesh3d>>,
        mut commands: Commands,
        asset_server: Res<AssetServer>,
    ) {
        let (name, local_transform, children) = loaded.get(on.loaded).else_return()?;
        let area_transform = area_transforms
            .get(on.area)
            .else_error("Could not get area transform.")?;

        let translation = local_transform.translation + area_transform.translation;

        if !name.contains("mushroom on edge") {
            return;
        }

        let mut stem_transform_and_children = None;

        for stem_entity in children.iter() {
            if let Ok(got) = stem.get(stem_entity) {
                stem_transform_and_children = Some(got);
                break;
            }
        }

        let (stem_transform, stem_children) =
            stem_transform_and_children.else_error("No child's transform and children.")?;
        let stem_translation = stem_transform.translation + translation;

        let caps = stem_children
            .iter()
            .filter_map(|cap| {
                let Ok(cap_transform) = cap_transforms.get(cap) else {
                    return None;
                };
                Some((cap_transform.translation + stem_translation, cap_transform.translation))
            })
            .collect();

        // Remove the default cube.
        commands.entity(on.loaded).despawn();

        light_shroom(
            translation,
            stem_translation,
            caps,
            &asset_server,
            &mut commands,
        );
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, load).add_systems(
        Update,
        (control, link_descendants, lock_in_place, light_fade),
    );
}

fn load(asset_server: Res<AssetServer>, mut commands: Commands) {
    let scene = asset_server.load("grave_yard/mesh.glb#Scene0");
    commands
        .spawn((SceneRoot(scene), Area))
        .observe(maze::load)
        .observe(edge_mushrooms::load);

    // Floor.
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(50., 1., 50.),
        Transform::from_translation(Vec3(0., -0.5, 0.)),
    ));

    // Back fence.
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(50., 2., 0.2),
        Transform::from_translation(Vec3(0., 0., -0.1)),
        CollisionLayers::new(CollisionLayer::Cable, CollisionLayer::Default),
    ));

    // Front fence.
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(50., 2., 0.2),
        Transform::from_translation(Vec3(0., 0., 5.1)),
        CollisionLayers::new(CollisionLayer::Cable, CollisionLayer::Default),
    ));

    // Right Wall.
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(0.2, 2., 2.5),
        Transform::from_translation(Vec3(5.1, 0., 1.25)),
        CollisionLayers::new(CollisionLayer::Cable, CollisionLayer::Default),
    ));
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(0.2, 2., 2.5),
        Transform::from_translation(Vec3(5.1, 0., 4.75)),
        CollisionLayers::new(CollisionLayer::Cable, CollisionLayer::Default),
    ));

    let player_scene = asset_server.load("player/mesh.glb#Scene0");
    commands
        .spawn((
            SceneRoot(player_scene),
            Area,
            Transform::from_translation(Vec3(0., 0.15, 0.)),
            RigidBody::Dynamic,
            Collider::sphere(0.15),
            Mass(30.),
            LockedAxes::ROTATION_LOCKED,
            Control,
            Player,
        ))
        .observe(player);

    spawn_fungal_fallen(Vec3(2.5, 0., 2.5), &asset_server, &mut commands);
    spawn_fungal_fallen(Vec3(-0.3, 0., 1.3), &asset_server, &mut commands);

    // light_shroom(Vec3(-7.8, 0., -1.5), &asset_server, &mut commands);
    // light_shroom(Vec3(-6.0, 0., -1.5), &asset_server, &mut commands);
    // light_shroom(Vec3(-4.2, 0., -1.5), &asset_server, &mut commands);
    // light_shroom(Vec3(-1.8, 0., -1.5), &asset_server, &mut commands);
    // light_shroom(Vec3(0.3, 0., -1.5), &asset_server, &mut commands);
    // light_shroom(Vec3(2.4, 0., -1.5), &asset_server, &mut commands);
    // light_shroom(Vec3(4.4, 0., -1.5), &asset_server, &mut commands);
    // light_shroom(Vec3(6.1, 0., -1.5), &asset_server, &mut commands);
    // light_shroom(Vec3(7.9, 0., -1.5), &asset_server, &mut commands);

    // light_shroom(Vec3(-1.8, 0., 4.9), &asset_server, &mut commands);
    // light_shroom(Vec3(-4.4, 0., 5.0), &asset_server, &mut commands);
    // light_shroom(Vec3(0.3, 0., 5.0), &asset_server, &mut commands);
    // light_shroom(Vec3(2.4, 0., 4.8), &asset_server, &mut commands);
    // light_shroom(Vec3(4.4, 0., 5.2), &asset_server, &mut commands);
}

fn light_shroom(
    base: Vec3,
    stem: Vec3,
    // (global position, local position)
    cap: Vec<(Vec3, Vec3)>,
    asset_server: &AssetServer,
    commands: &mut Commands,
) {
    let stem = FUNGUS_PURPLE_GLOW
        .rigid_body(RigidBody::Static)
        .start(base)
        .rigid_body(RigidBody::Dynamic)
        .gravity_override(Vec3::ZERO)
        .to(stem)
        .gravity_override(Vec3::Y * 1.5)
        .one(Vec3::Y)
        .gravity_override(Vec3::ZERO)
        .run(asset_server, commands);

    for (index, (global_translation, local_translation)) in cap.into_iter().enumerate() {
        let to_alter = stem
            .to(global_translation)
            .gravity_override(local_translation)
            .mesh(fungus_a::CAP)
            .gap_between_points(-0.5);

        let to_alter = if index == 0 {
            to_alter.alter(|commands| {
                commands.insert((
                    FadeLight,
                    PointLight {
                        //color: Color::srgb(0.937, 0.149, 0.941),
                        color: Color::srgb(1., 0.5, 1.),
                        intensity: 10000.,
                        radius: 0.5,
                        range: 5.,
                        shadows_enabled: false,
                        ..default()
                    },
                ));
            })
        } else {
            to_alter.alter(|_| {})
        };

        to_alter.one(Vec3::Y).run(asset_server, commands);
    }
}

fn spawn_fungal_fallen(translation: Vec3, asset_server: &AssetServer, commands: &mut Commands) {
    let fungal_fallen_scene = asset_server.load("fungal_fallen/mesh.glb#Scene0");

    commands
        .spawn((
            SceneRoot(fungal_fallen_scene),
            Area,
            Transform::from_translation(translation),
            LinkDescendants::new([
                ("left foot", "left leg lower", |joint| {
                    joint
                        .with_local_anchor1(Vec3(0., 0.03, -0.07))
                        .with_local_anchor2(Vec3(0., -(0.35 * 0.5), 0.))
                }),
                ("right foot", "right leg lower", |joint| {
                    joint
                        .with_local_anchor1(Vec3(0., 0.03, -0.07))
                        .with_local_anchor2(Vec3(0., -(0.35 * 0.5), 0.))
                }),
                ("right leg lower", "right leg higher", |joint| {
                    joint
                        .with_local_anchor1(Vec3(0., 0.35 * 0.5, 0.))
                        .with_local_anchor2(Vec3(0., -(0.3 * 0.5), 0.))
                }),
                ("left leg lower", "left leg higher", |joint| {
                    joint
                        .with_local_anchor1(Vec3(0., 0.35 * 0.5, 0.))
                        .with_local_anchor2(Vec3(0., -(0.3 * 0.5), 0.))
                }),
                ("right leg higher", "spine", |joint| {
                    joint
                        .with_local_anchor1(Vec3(0., 0.3 * 0.5, 0.))
                        .with_local_anchor2(Vec3(-0.1, -(0.45 * 0.5), 0.))
                }),
                ("left leg higher", "spine", |joint| {
                    joint
                        .with_local_anchor1(Vec3(0., 0.3 * 0.5, 0.))
                        .with_local_anchor2(Vec3(0.1, -(0.45 * 0.5), 0.))
                }),
                ("spine", "head", |joint| {
                    joint
                        .with_local_anchor1(Vec3(0., 0.45 * 0.5, 0.))
                        .with_local_anchor2(Vec3(0., -0.1, 0.))
                }),
            ]),
        ))
        .observe(fungal_fallen)
        .observe(coat);
}

#[derive(Component)]
pub struct Player;

fn player(
    on: On<AreaLoadedEntity>,
    loaded: Query<(&Name, &Transform)>,
    area_transform: Query<&Transform>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let (name, transform) = loaded
        .get(on.loaded)
        .else_error("Could not get components on loaded entity.")?;
    let area_transform = area_transform
        .get(on.area)
        .else_error("Could not get area transform.")?;

    let translation = transform.translation + area_transform.translation;

    if name.as_str() == "main" {
        // tail
        player_body::MAIN
            .rigid_body(RigidBody::Dynamic)
            .start_connected_to(translation, on.area)
            .mesh(FUNGUS_PURPLE_GLOW)
            .gravity_override(Vec3::ZERO)
            .to(translation + Vec3::X * 1.0)
            // .gravity_override(Vec3::Y * 1.5)
            // .one(Vec3::Y)
            // .gravity_override(Vec3::ZERO)
            // .alter(|commands| {
            //     commands.insert(PointLight {
            //         color: Color::srgb(0.937, 0.149, 0.941),
            //         color: Color::srgb(1., 0.5, 1.),
            //         intensity: 1000.,
            //         radius: 0.5,
            //         range: 5.,
            //         shadows_enabled: false,
            //         ..default()
            //     });
            // })
            .run(&asset_server, &mut commands);
    }
}

#[derive(Component)]
struct Control;

fn control(
    velocity: Query<&mut LinearVelocity, With<Control>>,
    actions: Res<Actions>,
    time: Res<Time>,
) {
    for mut velocity in velocity {
        //let horizontal = Vec2::from_angle(45.0_f32.to_radians()).rotate(
        let horizontal = actions
            .clamped_axis_pair(&Action::Horizontal)
            .normalize_or_zero();

        let direction = Vec2::new(1., -1.);

        // speed = distance / time
        // TO DO: Why is 1 centimetre a second so fast? It actually causes us to move at
        // 0.5 metres a second, which can't be right.
        let speed = 0.01 / time.delta_secs();
        if speed.is_nan() || speed.is_infinite() {
            error!("Controls are broken. Skipping.");
            return;
        }
        let velocity_xz = horizontal * speed * direction;

        velocity.x = velocity_xz.x;
        velocity.z = velocity_xz.y;

        // TO DO: Have some acceleration, instead of this instant velocity
        // control.
    }
}

#[derive(Component)]
struct LinkDescendants(Vec<(Find, Find, fn(SphericalJoint) -> SphericalJoint)>);

impl LinkDescendants {
    fn new<const N: usize>(
        links: [(
            &'static str,
            &'static str,
            fn(SphericalJoint) -> SphericalJoint,
        ); N],
    ) -> Self {
        LinkDescendants(
            links
                .into_iter()
                .map(|link| (Find::Searching(link.0), Find::Searching(link.1), link.2))
                .collect(),
        )
    }
}

enum Find {
    Searching(&'static str),
    Found(Entity),
}

fn link_descendants(
    mut links: Query<(Entity, &mut LinkDescendants), Without<SphericalJoint>>,
    children: Query<&Children>,
    names: Query<&Name>,
    mut commands: Commands,
) {
    const COMPLIANCE: f32 = 0.001; //0.0001;

    links.iter_mut().for_each(|(parent_entity, mut links)| {
        // If it is empty, then there is nothing left to link.
        // If we had this check further in then it would be more optimal,
        // as it would break as soon as it becomes empty.
        // I decided to place it here as it works well enough.
        if links.0.is_empty() {
            return;
        }

        children
            .iter_descendants(parent_entity)
            .for_each(|descendant_entity| {
                let name = names.get(descendant_entity).else_return()?.as_str();
                // Because we are going in reverse we can safely remove without worrying about
                // being out of bounds.
                (0..links.0.len()).rev().for_each(|index| {
                    let link = &mut links.0[index];
                    match link {
                        (Find::Searching(name_1), Find::Searching(name_2), _) => {
                            if name == *name_1 {
                                link.0 = Find::Found(descendant_entity);
                            } else if name == *name_2 {
                                link.1 = Find::Found(descendant_entity);
                            }
                        }
                        (Find::Searching(name_1), Find::Found(entity_2), alter) => {
                            if name == *name_1 {
                                let entity_1 = descendant_entity;
                                let entity_2 = *entity_2;
                                let alter = *alter;
                                links.0.swap_remove(index);

                                let joint = SphericalJoint::new(entity_1, entity_2)
                                    .with_point_compliance(COMPLIANCE)
                                    .with_swing_compliance(COMPLIANCE)
                                    .with_twist_compliance(COMPLIANCE);

                                commands.spawn(alter(joint));

                                info!("Completed.");
                            }
                        }
                        (Find::Found(entity_1), Find::Searching(name_2), alter) => {
                            if name == *name_2 {
                                let entity_1 = *entity_1;
                                let entity_2 = descendant_entity;
                                let alter = *alter;
                                links.0.swap_remove(index);

                                let joint = SphericalJoint::new(entity_1, entity_2)
                                    .with_point_compliance(COMPLIANCE)
                                    .with_swing_compliance(COMPLIANCE)
                                    .with_twist_compliance(COMPLIANCE);

                                commands.spawn(alter(joint));
                            }
                        }
                        _ => unreachable!(),
                    }
                });
            });
    });
}

fn fungal_fallen(
    on: On<AreaLoadedEntity>,
    loaded: Query<&Name>,
    transforms: Query<&Transform>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    // TO DO: Global transforms don't work On<AreaLoadedEntity>.
    let name = loaded
        .get(on.loaded)
        .else_error("Could not get components on loaded entity.")?;

    let area_transform = transforms
        .get(on.area)
        .else_error("Could not get area's transform.")?;

    match name.as_str() {
        "left foot" | "right foot" => {
            commands.entity(on.loaded).insert((
                RigidBody::Dynamic,
                Collider::cuboid(0.1, 0.06, 0.18),
                Mass(5.),
            ));
        }
        "left leg lower" | "right leg lower" => {
            commands.entity(on.loaded).insert((
                RigidBody::Dynamic,
                Collider::cuboid(0.08, 0.35, 0.08),
                Mass(5.),
                Accelerate(Vec3(0., 0.1, 0.)),
            ));
        }
        "left leg higher" | "right leg higher" => {
            commands.entity(on.loaded).insert((
                RigidBody::Dynamic,
                Collider::cuboid(0.12, 0.3, 0.12),
                Mass(5.),
                Accelerate(Vec3(0., 0.1, 0.)),
            ));
        }
        "spine" => {
            commands.entity(on.loaded).insert((
                RigidBody::Dynamic,
                Collider::cuboid(0.05, 0.45, 0.05),
                Mass(5.),
                Accelerate(Vec3(0., 0.1, 0.)),
            ));
        }
        "head" => {
            commands.entity(on.loaded).insert((
                RigidBody::Dynamic,
                Collider::sphere(0.1),
                Mass(5.),
                Accelerate(Vec3(0., 0.1, 0.)),
            ));

            // I have no explanation.
            // Joints don't work in any reasonable manner.

            let offset_from_head = Vec3(0., 0.14, 0.);
            let head_translation = Vec3(0., 1.3026, 0.);
            let anchor_translation =
                area_transform.translation + head_translation + offset_from_head;

            let secret_anchor = commands
                .spawn((
                    GravityScale(0.),
                    LockedAxes::ROTATION_LOCKED,
                    RigidBody::Dynamic,
                    Collider::sphere(0.01),
                    Mass(1.),
                    Transform::from_translation(anchor_translation),
                ))
                .id();
            commands.spawn(
                DistanceJoint::new(on.loaded, secret_anchor)
                    .with_limits(0., 0.01)
                    .with_local_anchor1(offset_from_head),
            );

            crate::chain::fungal_fallen::ANCHOR
                .rigid_body(RigidBody::Dynamic)
                .start_connected_to(anchor_translation, secret_anchor)
                .mesh(crate::chain::fungal_fallen::STEM)
                .gravity_override(Vec3::ZERO)
                .to(anchor_translation + Vec3::Y * 0.3)
                .gravity_override(Vec3::Y * 1.5)
                .mesh(crate::chain::fungal_fallen::CAP)
                .one(Vec3::Y)
                .run(&asset_server, &mut commands);
        }
        _ => (),
    }
}

enum Coat {
    Searching(Vec<Entity>),
    Found(Entity),
}

struct CoatLocal(Vec<(&'static str, Coat)>);

impl Default for CoatLocal {
    fn default() -> Self {
        Self(vec![
            ("right foot", Coat::Searching(vec![])),
            ("right leg lower", Coat::Searching(vec![])),
            ("spine", Coat::Searching(vec![])),
        ])
    }
}

fn coat(
    on: On<AreaLoadedEntity>,
    loaded: Query<(&Name, &Transform)>,
    transforms: Query<&Transform>,
    mut commands: Commands,
    mut coat: Local<CoatLocal>,
) {
    fn fix_coat(
        attach_to_entity: Entity,
        attach_to_transform: &Transform,
        coat_entity: Entity,
        coat_transform: &Transform,
        commands: &mut Commands,
    ) {
        // Create a dummy that has attach_to_entity as its parent and is precisely the
        // same spot as the coat. Account for the space change.
        let dummy_translation = coat_transform.translation - attach_to_transform.translation;
        let dummy_entity = commands
            .spawn((
                Transform::from_translation(dummy_translation),
                LockInPlace(Transform::from_translation(dummy_translation)),
                RigidBody::Dynamic,
                Mass(1.),
                LockedAxes::ALL_LOCKED,
            ))
            .id();
        commands.entity(attach_to_entity).add_child(dummy_entity);

        // Coat then needs a distance joint to the dummy.
        commands.spawn(
            DistanceJoint::new(dummy_entity, coat_entity)
                .with_limits(0., 0.01)
                .with_compliance(0.0001),
        );

        // Coat need a gravity scale of zero.
        commands.entity(coat_entity).insert((
            GravityScale(0.),
            RigidBody::Dynamic,
            AngularInertia::from_shape(&Collider::sphere(0.001), 1.),
            Mass(10.),
            LockedAxes::ROTATION_LOCKED,
            LinearDamping(2.),
            SleepingDisabled,
        ));
    }

    let (name, transform) = loaded
        .get(on.loaded)
        .else_error("Could not get components on loaded entity.")?;

    let name = name.as_str();

    coat.0.iter_mut().for_each(|(attach_to, coat)| match coat {
        Coat::Searching(coats) => {
            if name == *attach_to {
                coats.iter().copied().for_each(|coat| {
                    let coat_transform = transforms
                        .get(coat)
                        .else_error("Could not get transform of coat.")?;

                    fix_coat(on.loaded, transform, coat, coat_transform, &mut commands);
                });
                *coat = Coat::Found(on.loaded);
            } else if name.contains("coat") && name.contains(*attach_to) {
                coats.push(on.loaded);
            }
        }
        Coat::Found(attach_to_entity) => {
            if name.contains("coat") && name.contains(*attach_to) {
                let attach_to_transform = transforms
                    .get(*attach_to_entity)
                    .else_error("Could not get transform of attach_to.")?;

                fix_coat(
                    *attach_to_entity,
                    attach_to_transform,
                    on.loaded,
                    transform,
                    &mut commands,
                );
            }
        }
    });
}

/// Prevent an entity from having a different transform than the one stored.
/// This should not be required, and yet it is.
#[derive(Component)]
struct LockInPlace(Transform);

fn lock_in_place(mut locks: Query<(&mut Transform, &LockInPlace)>) {
    locks.iter_mut().for_each(|(mut transform, lock)| {
        *transform = lock.0;
    });
}

#[derive(Component)]
struct FadeLight;

enum FadeStage {
    Dark,
    Light,
}

struct FadeState {
    stage: FadeStage,
    time_remaining: f32,
    intensity: f32,
}

impl FadeState {
    const LIGHT_CHANGE_PER_SECOND: f32 = 10000.;
    const LIGHT_SECONDS: f32 = 3.;
    const DARK_SECONDS: f32 = 8.;
}

impl Default for FadeState {
    fn default() -> Self {
        Self {
            stage: FadeStage::Dark,
            time_remaining: Self::DARK_SECONDS,
            intensity: 0.,
        }
    }
}

fn light_fade(
    mut lights: Query<&mut PointLight, With<FadeLight>>,
    mut state: Local<FadeState>,
    time: Res<Time>,
) {
    state.time_remaining -= time.delta_secs();

    let light_change = match state.stage {
        FadeStage::Dark => {
            if state.time_remaining <= 0. {
                info!("From dark to light.");
                state.stage = FadeStage::Light;
                state.time_remaining = FadeState::LIGHT_SECONDS;
                FadeState::LIGHT_CHANGE_PER_SECOND
            } else {
                -FadeState::LIGHT_CHANGE_PER_SECOND
            }
        }
        FadeStage::Light => {
            if state.time_remaining <= 0. {
                info!("From light to dark.");
                state.stage = FadeStage::Dark;
                state.time_remaining = FadeState::DARK_SECONDS;
                -FadeState::LIGHT_CHANGE_PER_SECOND
            } else {
                FadeState::LIGHT_CHANGE_PER_SECOND
            }
        }
    };

    state.intensity = (state.intensity + light_change * time.delta_secs()).clamp(0., 10000.);

    lights.iter_mut().for_each(|mut light| {
        light.intensity = state.intensity;
    });
}
