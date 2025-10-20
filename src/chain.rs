pub use crate::bevy_prelude::*;
use crate::physics::{CollisionLayer, common_properties::AIR_RESISTANCE};
use avian3d::prelude::*;

pub struct ChainConfig {
    pub mesh: &'static str,
    pub gap_between_points: f32,
    pub radius: f32,
    pub gravity_scale: f32,
    pub rigid_body: RigidBody,
    pub mass: f32,
    pub alter: fn(&mut EntityCommands<'_>),
}

impl ChainConfig {
    fn point_bundle(&self, mesh: Handle<Scene>, translation: Vec3) -> impl Bundle {
        (
            SceneRoot(mesh),
            Transform::from_translation(translation),
            self.rigid_body,
            LockedAxes::ROTATION_LOCKED,
            Mass(self.mass),
            GravityScale(self.gravity_scale),
            Collider::sphere(self.radius),
            SleepThreshold {
                linear: 0.01,
                ..default()
            },
            CollisionLayers::new(CollisionLayer::Cable, CollisionLayer::Default),
            LinearDamping(1.),
        )
    }

    pub fn start<'f, 'a, 'c, 'w, 's>(
        &'f self,
        commands: &'c mut Commands<'w, 's>,
        asset_server: &'a AssetServer,
        translation: Vec3,
    ) -> ChainBuilder<'f, 'a, 'c, 'w, 's> {
        let mesh = asset_server.load(format!("{}/mesh.glb#Scene0", self.mesh));
        let mut previous_entity = commands.spawn(self.point_bundle(mesh.clone(), translation));
        (self.alter)(&mut previous_entity);
        let previous_entity = previous_entity.id();

        ChainBuilder {
            commands,
            asset_server,
            config: self,

            state: BuilderState {
                previous_entity,
                previous_translation: translation,
                previous_radius: self.radius,
                mesh,
            },
        }
    }
}

pub struct ChainBuilder<'f, 'a, 'c, 'w, 's> {
    commands: &'c mut Commands<'w, 's>,
    asset_server: &'a AssetServer,
    config: &'f ChainConfig,

    pub state: BuilderState,
}

#[derive(Clone)]
pub struct BuilderState {
    previous_entity: Entity,
    previous_translation: Vec3,
    previous_radius: f32,
    mesh: Handle<Scene>,
}

impl<'f> ChainBuilder<'f, '_, '_, '_, '_> {
    pub fn configure(&mut self, config: &'f ChainConfig) -> &mut Self {
        self.state.mesh = self
            .asset_server
            .load(format!("{}/mesh.glb#Scene0", config.mesh));
        self.config = config;
        self
    }

    pub fn one(&mut self) -> &mut Self {
        let last_final_translation = self.state.previous_translation;

        // radius gap radius?
        let point_translation =
            (self.state.previous_radius + self.config.gap_between_points + self.config.radius)
                * Vec3::Y
                + last_final_translation;

        let saved_previous_entity = self.state.previous_entity;

        let mut previous_entity = self.commands.spawn(
            self.config
                .point_bundle(self.state.mesh.clone(), point_translation),
        );
        (self.config.alter)(&mut previous_entity);
        self.state.previous_entity = previous_entity.id();

        self.commands.spawn(
            DistanceJoint::new(saved_previous_entity, self.state.previous_entity)
                .with_limits(0., self.config.radius * 2. + self.config.gap_between_points),
        );

        self.state.previous_translation = point_translation;

        self.state.previous_radius = self.config.radius;
        self
    }

    pub fn to(&mut self, translation: Vec3) -> &mut Self {
        let direction = (translation - self.state.previous_translation).normalize_or_zero();

        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        let quantity = (translation.distance(self.state.previous_translation)
            / (self.config.radius + self.config.gap_between_points + self.config.radius))
            .floor() as u16;

        let last_final_translation = self.state.previous_translation;

        for i in 0..quantity {
            // radius gap radius?
            let point_translation = (self.state.previous_radius
                + self.config.gap_between_points
                + self.config.radius
                + (f32::from(i)
                    * (self.config.radius + self.config.gap_between_points + self.config.radius)))
                * direction
                + last_final_translation;

            let saved_previous_entity = self.state.previous_entity;

            let mut previous_entity = self.commands.spawn(
                self.config
                    .point_bundle(self.state.mesh.clone(), point_translation),
            );
            (self.config.alter)(&mut previous_entity);
            self.state.previous_entity = previous_entity.id();

            // if i % 6 == 0 {
            //     cable.insert((Collider::sphere(CABLE_RADIUS), collision_layers));
            // } else {
            //     cable.insert(GravityScale(-0.01));
            // }

            self.commands.spawn(
                DistanceJoint::new(saved_previous_entity, self.state.previous_entity)
                    .with_limits(0., self.config.radius * 2. + self.config.gap_between_points),
            );

            self.state.previous_translation = point_translation;
        }

        self.state.previous_radius = self.config.radius;
        self
    }
}

/*
const PLUG_DENSITY: f32 = 25.;
const PLUG_COMPLIANCE: f32 = 0.0001;

const CABLE_RADIUS: f32 = 0.25 * 0.5;
const CABLE_DENSITY: f32 = 10.;
const CABLE_COMPLIANCE: f32 = 0.01;

const MAX_DISTANCE: f32 = 0.2;

pub fn load(
    names: Query<(Entity, &Name, &Transform, &LoadedFromArea), Added<LoadedFromArea>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for (root_entity, name, transform, loaded_from_area) in names {
        if name.starts_with("cable") {
            let plug_scene = asset_server.load("machines/plug.glb#Scene0");
            let cable_scene = asset_server.load("machines/cable.glb#Scene0");

            let collision_layers =
                CollisionLayers::new(CollisionLayer::Cable, CollisionLayer::Default);

            let mut select_others = vec![root_entity];

            let head_joint = commands.spawn_empty().id();
            let tail = commands.spawn_empty().id();
            select_others.push(tail);

            let head = commands
                .entity(root_entity)
                .insert((
                    Plug {
                        outlet_sensors_within_range: vec![],
                        dragged: false,
                        outlet_sensor_connected_to: None,
                        joint: head_joint,
                        other_end: tail,
                    },
                    RigidBody::Dynamic,
                    MassPropertiesBundle::from_shape(&Cuboid::new(0.8, 0.4, 0.8), PLUG_DENSITY),
                    Collider::cuboid(0.8, 0.4, 0.8),
                    collision_layers,
                    SceneRoot(plug_scene.clone()),
                    Propagate(ComesFromRootEntity(root_entity)),
                    Interactable,
                    Dragged::default(),
                ))
                .observe(drag_start)
                .observe(drag_end)
                .id();

            let mut previous_transform = *transform;
            previous_transform.translation.y -= 0.2 + CABLE_RADIUS;
            let mut previous = commands
                .spawn((
                    RigidBody::Dynamic,
                    LockedAxes::ROTATION_LOCKED,
                    MassPropertiesBundle::from_shape(&Sphere::new(CABLE_RADIUS), CABLE_DENSITY),
                    Collider::sphere(CABLE_RADIUS),
                    collision_layers,
                    SceneRoot(cable_scene.clone()),
                    Propagate(ComesFromRootEntity(root_entity)),
                    previous_transform,
                    Name::new(format!("block_loading_{name}_first_previous")),
                    LoadedFromArea(loaded_from_area.0),
                ))
                .id();
            select_others.push(previous);

            commands.spawn(
                SphericalJoint::new(head, previous)
                    .with_local_anchor1(Vec3::NEG_Y * 0.2)
                    .with_local_anchor2(Vec3::Y * CABLE_RADIUS)
                    .with_point_compliance(PLUG_COMPLIANCE)
                    .with_swing_compliance(PLUG_COMPLIANCE)
                    .with_twist_compliance(PLUG_COMPLIANCE),
            );
            commands.spawn(
                DistanceJoint::new(head, previous)
                    .with_limits(0., CABLE_RADIUS + 0.2 + MAX_DISTANCE),
            );

            // TODO: Find a proper way to do length.
            let length: u8 = 10;
            for i in 1..length {
                let mut transform = *transform;
                transform.translation.y -= 0.2 + CABLE_RADIUS;
                transform.translation.x += f32::from(i) * CABLE_RADIUS * 2.;

                let mut cable = commands.spawn((
                    RigidBody::Dynamic,
                    LockedAxes::ROTATION_LOCKED,
                    MassPropertiesBundle::from_shape(&Sphere::new(CABLE_RADIUS), CABLE_DENSITY),
                    SceneRoot(cable_scene.clone()),
                    Propagate(ComesFromRootEntity(root_entity)),
                    transform,
                    Name::new(format!("block_loading_{name}_cable_{i}")),
                    LoadedFromArea(loaded_from_area.0),
                ));
                let current = cable.id();

                if i % 6 == 0 {
                    cable.insert((Collider::sphere(CABLE_RADIUS), collision_layers));
                } else {
                    cable.insert(GravityScale(-0.01));
                }

                commands.spawn(
                    SphericalJoint::new(previous, current)
                        .with_local_anchor1(Vec3::NEG_Y * CABLE_RADIUS)
                        .with_local_anchor2(Vec3::Y * CABLE_RADIUS)
                        .with_point_compliance(CABLE_COMPLIANCE)
                        .with_swing_compliance(CABLE_COMPLIANCE)
                        .with_twist_compliance(CABLE_COMPLIANCE),
                );
                commands.spawn(
                    DistanceJoint::new(previous, current)
                        .with_limits(0., CABLE_RADIUS * 2. + MAX_DISTANCE),
                );

                previous = current;
                select_others.push(previous);
            }

            let tail_joint = commands.spawn_empty().id();

            let mut tail_transform = *transform;
            tail_transform.translation.x += f32::from(length - 1) * CABLE_RADIUS * 2.;

            let tail = commands
                .entity(tail)
                .insert((
                    Plug {
                        outlet_sensors_within_range: vec![],
                        dragged: false,
                        outlet_sensor_connected_to: None,
                        joint: tail_joint,
                        other_end: head,
                    },
                    RigidBody::Dynamic,
                    MassPropertiesBundle::from_shape(&Cuboid::new(0.8, 0.4, 0.8), PLUG_DENSITY),
                    Collider::cuboid(0.8, 0.4, 0.8),
                    collision_layers,
                    SceneRoot(plug_scene.clone()),
                    Propagate(ComesFromRootEntity(root_entity)),
                    tail_transform,
                    Interactable,
                    Dragged::default(),
                    SelectOthers(select_others.clone()),
                    Name::new(format!("block_loading_{name}_tail")),
                    LoadedFromArea(loaded_from_area.0),
                ))
                .observe(drag_start)
                .observe(drag_end)
                .id();

            commands.spawn(
                SphericalJoint::new(previous, tail)
                    .with_local_anchor1(Vec3::Y * CABLE_RADIUS)
                    .with_local_anchor2(Vec3::NEG_Y * 0.2)
                    .with_point_compliance(PLUG_COMPLIANCE)
                    .with_swing_compliance(PLUG_COMPLIANCE)
                    .with_twist_compliance(PLUG_COMPLIANCE),
            );
            commands.spawn(
                DistanceJoint::new(previous, tail)
                    .with_limits(0., CABLE_RADIUS + 0.2 + MAX_DISTANCE),
            );

            commands
                .entity(root_entity)
                .insert(SelectOthers(select_others));
        }
    }
}
*/
