pub use crate::bevy_prelude::*;
use crate::physics::{Accelerate, CollisionLayer};
use avian3d::prelude::*;

pub const FUNGUS_PURPLE_GLOW: ChainPointConfigured =
    ChainPointConfigured::new("fungus_purple_glow", 0.05);

pub mod fungus_a {
    use super::ChainPointConfigured;

    pub const CAP: ChainPointConfigured = ChainPointConfigured::new("fungus_a/cap", 0.5 * 0.1);
}

pub struct ChainPointConfigured {
    mesh: &'static str,
    radius: f32,
    config: ChainOperationConfig,
}

impl ChainPointConfigured {
    const fn new(mesh: &'static str, radius: f32) -> Self {
        Self {
            mesh,
            radius,
            config: ChainOperationConfig::default(),
        }
    }
}

#[derive(Clone)]
pub struct ChainOperationConfig {
    gap_between_points: f32,
    gravity_override: Option<Vec3>,
    rigid_body: RigidBody,
    mass: f32,
    alter: fn(&mut EntityCommands<'_>),
}

impl ChainOperationConfig {
    const fn default() -> Self {
        Self {
            gap_between_points: 0.,
            gravity_override: None,
            rigid_body: RigidBody::Dynamic,
            mass: 0.05,
            alter: |_| {},
        }
    }
}

impl ChainPointConfigured {
    pub fn start(self, pos: Vec3) -> ChainOperationStart {
        ChainOperationStart {
            chain_point_configured: self,
            translation: pos,
        }
    }

    pub fn rigid_body(mut self, rigid_body: RigidBody) -> Self {
        self.config.rigid_body = rigid_body;
        self
    }
}

#[derive(Clone)]
pub struct ChainOperationState {
    previous_entity: Entity,
    previous_translation: Vec3,
    previous_radius: f32,

    mesh: Handle<Scene>,
    radius: f32,

    config: ChainOperationConfig,
}

impl ChainOperationState {
    fn insert_point_bundle(
        &self,
        entity_commands: &mut EntityCommands,
        translation: Vec3,
        extra: impl Bundle,
    ) {
        entity_commands.insert((
            SceneRoot(self.mesh.clone()),
            Transform::from_translation(translation),
            self.config.rigid_body,
            LockedAxes::ROTATION_LOCKED,
            Mass(self.config.mass),
            Collider::sphere(self.radius),
            SleepThreshold {
                linear: 0.01,
                ..default()
            },
            CollisionLayers::new(CollisionLayer::Cable, CollisionLayer::Default),
            LinearDamping(1.),
            extra,
        ));

        if let Some(gravity_override) = self.config.gravity_override {
            entity_commands.insert((GravityScale(0.), Accelerate(gravity_override)));
        }
    }
}

pub trait ChainOperation: Sized {
    fn internal_start(&self, asset_server: &AssetServer) -> ChainOperationState;

    fn internal_apply(
        self,
        state: &mut ChainOperationState,
        commands: &mut Commands,
        asset_server: &AssetServer,
    );

    fn run(self, asset_server: &AssetServer, commands: &mut Commands) -> ChainOperationFinished {
        let mut state = self.internal_start(asset_server);
        self.internal_apply(&mut state, commands, asset_server);

        ChainOperationFinished { state }
    }

    fn to(self, pos: Vec3) -> ChainOperationTo<Self> {
        ChainOperationTo {
            previous: self,
            translation: pos,
        }
    }

    fn one(self, dir: Vec3) -> ChainOperationOne<Self> {
        ChainOperationOne {
            previous: self,
            direction: dir,
        }
    }

    fn rigid_body(self, rigid_body: RigidBody) -> ChainOperationRigidBody<Self> {
        ChainOperationRigidBody {
            previous: self,
            rigid_body,
        }
    }

    fn gravity_override(self, gravity_override: Vec3) -> ChainOperationGravityOverride<Self> {
        ChainOperationGravityOverride {
            previous: self,
            acceleration: gravity_override,
        }
    }

    fn mesh(self, mesh: ChainPointConfigured) -> ChainOperationMesh<Self> {
        ChainOperationMesh {
            previous: self,
            mesh: mesh.mesh,
            radius: mesh.radius,
        }
    }
}

pub struct ChainOperationStart {
    chain_point_configured: ChainPointConfigured,
    translation: Vec3,
}

impl ChainOperation for ChainOperationStart {
    fn internal_start(&self, asset_server: &AssetServer) -> ChainOperationState {
        ChainOperationState {
            previous_entity: Entity::PLACEHOLDER,
            previous_translation: self.translation,
            previous_radius: self.chain_point_configured.radius,

            mesh: asset_server.load(format!(
                "{}/mesh.glb#Scene0",
                self.chain_point_configured.mesh
            )),
            radius: self.chain_point_configured.radius,

            config: self.chain_point_configured.config.clone(),
        }
    }

    fn internal_apply(
        self,
        state: &mut ChainOperationState,
        commands: &mut Commands,
        _: &AssetServer,
    ) {
        let mut previous_entity = commands.spawn_empty();
        state.insert_point_bundle(&mut previous_entity, self.translation, ());
        (state.config.alter)(&mut previous_entity);
        state.previous_entity = previous_entity.id();

        info!("Started!");
    }
}

#[derive(Clone)]
pub struct ChainOperationFinished {
    state: ChainOperationState,
}

impl ChainOperation for ChainOperationFinished {
    fn internal_start(&self, _: &AssetServer) -> ChainOperationState {
        self.state.clone()
    }

    fn internal_apply(self, _: &mut ChainOperationState, _: &mut Commands, _: &AssetServer) {}
}

pub struct ChainOperationTo<T> {
    previous: T,
    translation: Vec3,
}

impl<T: ChainOperation> ChainOperation for ChainOperationTo<T> {
    fn internal_start(&self, asset_server: &AssetServer) -> ChainOperationState {
        self.previous.internal_start(asset_server)
    }

    fn internal_apply(
        self,
        state: &mut ChainOperationState,
        commands: &mut Commands,
        asset_server: &AssetServer,
    ) {
        self.previous.internal_apply(state, commands, asset_server);

        let direction = (self.translation - state.previous_translation).normalize_or_zero();

        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        let quantity = (self.translation.distance(state.previous_translation)
            / (state.radius + state.config.gap_between_points + state.radius))
            .floor() as u16;

        let last_final_translation = state.previous_translation;

        for i in 0..quantity {
            // radius gap radius?
            let point_translation = (state.previous_radius
                + state.config.gap_between_points
                + state.radius
                + (f32::from(i) * (state.radius + state.config.gap_between_points + state.radius)))
                * direction
                + last_final_translation;

            let saved_previous_entity = state.previous_entity;

            let mut previous_entity = commands.spawn_empty();
            state.insert_point_bundle(&mut previous_entity, point_translation, ());
            (state.config.alter)(&mut previous_entity);
            state.previous_entity = previous_entity.id();

            // if i % 6 == 0 {
            //     cable.insert((Collider::sphere(CABLE_RADIUS), collision_layers));
            // } else {
            //     cable.insert(GravityScale(-0.01));
            // }

            // If i is 0, then we need to account for the previous radius.
            // This is a bad way of accounting for it. All the maths in this function should
            // be re-worked out.
            if i == 0 {
                commands.spawn(
                    DistanceJoint::new(saved_previous_entity, state.previous_entity).with_limits(
                        0.,
                        state.previous_radius + state.config.gap_between_points + state.radius,
                    ),
                );
            } else {
                commands.spawn(
                    DistanceJoint::new(saved_previous_entity, state.previous_entity)
                        .with_limits(0., state.radius * 2. + state.config.gap_between_points),
                );
            }

            state.previous_translation = point_translation;
        }

        state.previous_radius = state.radius;
    }
}

pub struct ChainOperationRigidBody<T> {
    previous: T,
    rigid_body: RigidBody,
}

impl<T: ChainOperation> ChainOperation for ChainOperationRigidBody<T> {
    fn internal_start(&self, asset_server: &AssetServer) -> ChainOperationState {
        self.previous.internal_start(asset_server)
    }

    fn internal_apply(
        self,
        state: &mut ChainOperationState,
        commands: &mut Commands,
        asset_server: &AssetServer,
    ) {
        self.previous.internal_apply(state, commands, asset_server);
        state.config.rigid_body = self.rigid_body;
    }
}

pub struct ChainOperationGravityOverride<T> {
    previous: T,
    acceleration: Vec3,
}

impl<T: ChainOperation> ChainOperation for ChainOperationGravityOverride<T> {
    fn internal_start(&self, asset_server: &AssetServer) -> ChainOperationState {
        self.previous.internal_start(asset_server)
    }

    fn internal_apply(
        self,
        state: &mut ChainOperationState,
        commands: &mut Commands,
        asset_server: &AssetServer,
    ) {
        self.previous.internal_apply(state, commands, asset_server);
        state.config.gravity_override = Some(self.acceleration);
    }
}

pub struct ChainOperationMesh<T> {
    previous: T,
    mesh: &'static str,
    radius: f32,
}

impl<T: ChainOperation> ChainOperation for ChainOperationMesh<T> {
    fn internal_start(&self, asset_server: &AssetServer) -> ChainOperationState {
        self.previous.internal_start(asset_server)
    }

    fn internal_apply(
        self,
        state: &mut ChainOperationState,
        commands: &mut Commands,
        asset_server: &AssetServer,
    ) {
        self.previous.internal_apply(state, commands, asset_server);
        state.radius = self.radius;
        state.mesh = asset_server.load(format!("{}/mesh.glb#Scene0", self.mesh));
    }
}

pub struct ChainOperationOne<T> {
    previous: T,
    direction: Vec3,
}

impl<T: ChainOperation> ChainOperation for ChainOperationOne<T> {
    fn internal_start(&self, asset_server: &AssetServer) -> ChainOperationState {
        self.previous.internal_start(asset_server)
    }

    fn internal_apply(
        self,
        state: &mut ChainOperationState,
        commands: &mut Commands,
        asset_server: &AssetServer,
    ) {
        self.previous.internal_apply(state, commands, asset_server);

        let last_final_translation = state.previous_translation;

        // radius gap radius?
        let point_translation =
            (state.previous_radius + state.config.gap_between_points + state.radius)
                * self.direction
                + last_final_translation;

        let saved_previous_entity = state.previous_entity;

        let mut previous_entity = commands.spawn_empty();
        state.insert_point_bundle(&mut previous_entity, point_translation, ());
        (state.config.alter)(&mut previous_entity);
        state.previous_entity = previous_entity.id();

        commands.spawn(
            DistanceJoint::new(saved_previous_entity, state.previous_entity).with_limits(
                0.,
                state.previous_radius + state.config.gap_between_points + state.radius,
            ),
        );

        state.previous_translation = point_translation;

        state.previous_radius = state.radius;
    }
}
