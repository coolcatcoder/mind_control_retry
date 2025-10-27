pub use crate::bevy_prelude::*;
use crate::physics::{Accelerate, CollisionLayer};
use avian3d::prelude::*;

pub const FUNGUS_PURPLE_GLOW: ChainPointConfigured =
    ChainPointConfigured::new("fungus_purple_glow", 0.05);

pub const SEAWEED: ChainPointConfigured = ChainPointConfigured::new("seaweed", 0.05);

pub mod fungus_a {
    use super::ChainPointConfigured;

    pub const CAP: ChainPointConfigured = ChainPointConfigured::new("fungus_a/cap", 0.5); // * 0.1);
}

pub mod fungus_small_pot {
    use super::ChainPointConfigured;

    pub const CAP: ChainPointConfigured = ChainPointConfigured::new("fungus_small_pot/cap", 0.04);
    pub const STEM: ChainPointConfigured = ChainPointConfigured::new("fungus_small_pot/stem", 0.02);
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
pub struct State {
    previous_entity: Entity,
    previous_translation: Vec3,
    previous_radius: f32,

    mesh_handle: Handle<Scene>,

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

macro_rules! operation {
    ([$name:ident] $operation:ident, $($parameter:ident: $parameter_type:ty),* {$($body:tt)*}) => {
        fn $name(self, $($parameter: $parameter_type),*) -> impl ChainOperation {
            pub struct Operation<T> {
                previous: T,
                $($parameter: $parameter_type),*
            }

            impl<T: ChainOperation> ChainOperation for Operation<T> {
                fn internal_start(&self, asset_server: &AssetServer) -> ChainOperationState {
                    self.previous.internal_start(asset_server)
                }

                fn internal_apply(
                    self,
                    state: &mut ChainOperationState,
                    commands: &mut Commands,
                    asset_server: &AssetServer,
                ) {
                    struct CurrentOperation<'sr, 'ar, 'cr, 'w, 's> {
                        state: &'sr mut ChainOperationState,
                        #[allow(dead_code)]
                        asset_server: &'ar AssetServer,
                        #[allow(dead_code)]
                        commands: &'cr mut Commands<'w, 's>,
                        $($parameter: $parameter_type),*
                    }

                    self.previous.internal_apply(state, commands, asset_server);

                    let $operation = CurrentOperation {
                        state,
                        commands,
                        asset_server,
                        $($parameter: self.$parameter),*
                    };
                    $($body)*
                }
            }

            Operation {
                previous: self,
                $($parameter),*
            }
        }
    };

    ($name:ident, $($parameter:ident: $parameter_type:ty),* {$($body:tt)*}) => {
        fn $name(self, $($parameter: $parameter_type),*) -> impl ChainOperation {
            pub struct Operation<T> {
                previous: T,
                $($parameter: $parameter_type),*
            }

            impl<T: ChainOperation> ChainOperation for Operation<T> {
                fn internal_start(&self, asset_server: &AssetServer) -> ChainOperationState {
                    self.previous.internal_start(asset_server)
                }

                fn internal_apply(
                    self,
                    state: &mut ChainOperationState,
                    commands: &mut Commands,
                    asset_server: &AssetServer,
                ) {
                    struct CurrentOperation<'sr, 'ar, 'cr, 'w, 's> {
                        state: &'sr mut ChainOperationState,
                        #[allow(dead_code)]
                        asset_server: &'ar AssetServer,
                        #[allow(dead_code)]
                        commands: &'cr mut Commands<'w, 's>,
                        $($parameter: $parameter_type),*
                    }

                    self.previous.internal_apply(state, commands, asset_server);

                    let $name = CurrentOperation {
                        state,
                        commands,
                        asset_server,
                        $($parameter: self.$parameter),*
                    };
                    $($body)*
                }
            }

            Operation {
                previous: self,
                $($parameter),*
            }
        }
    };
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

    operation!([to] state, pos: Vec3 {
        let direction = (state.pos - state.state.previous_translation).normalize_or_zero();

        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        let quantity = (state.pos.distance(state.state.previous_translation)
            / (state.state.radius + state.state.config.gap_between_points + state.state.radius))
            .floor() as u16;

        let last_final_translation = state.state.previous_translation;

        for i in 0..quantity {
            // radius gap radius?
            let point_translation = (state.state.previous_radius
                + state.state.config.gap_between_points
                + state.state.radius
                + (f32::from(i) * (state.state.radius + state.state.config.gap_between_points + state.state.radius)))
                * direction
                + last_final_translation;

            let saved_previous_entity = state.state.previous_entity;

            let mut previous_entity = state.commands.spawn_empty();
            state.state.insert_point_bundle(&mut previous_entity, point_translation, ());
            (state.state.config.alter)(&mut previous_entity);
            state.state.previous_entity = previous_entity.id();

            // if i % 6 == 0 {
            //     cable.insert((Collider::sphere(CABLE_RADIUS), collision_layers));
            // } else {
            //     cable.insert(GravityScale(-0.01));
            // }

            // If i is 0, then we need to account for the previous radius.
            // This is a bad way of accounting for it. All the maths in this function should
            // be re-worked out.
            if i == 0 {
                state.commands.spawn(
                    DistanceJoint::new(saved_previous_entity, state.state.previous_entity).with_limits(
                        0.,
                        state.state.previous_radius + state.state.config.gap_between_points + state.state.radius,
                    ),
                );
            } else {
                state.commands.spawn(
                    DistanceJoint::new(saved_previous_entity, state.state.previous_entity)
                        .with_limits(0., state.state.radius * 2. + state.state.config.gap_between_points),
                );
            }

            state.state.previous_translation = point_translation;
        }

        state.state.previous_radius = state.state.radius;
    });

    operation!([one] state, dir: Vec3 {
        let last_final_translation = state.state.previous_translation;

        // radius gap radius?
        let point_translation =
            (state.state.previous_radius + state.state.config.gap_between_points + state.state.radius)
                * state.dir
                + last_final_translation;

        let saved_previous_entity = state.state.previous_entity;

        let mut previous_entity = state.commands.spawn_empty();
        state.state.insert_point_bundle(&mut previous_entity, point_translation, ());
        (state.state.config.alter)(&mut previous_entity);
        state.state.previous_entity = previous_entity.id();

        state.commands.spawn(
            DistanceJoint::new(saved_previous_entity, state.state.previous_entity).with_limits(
                0.,
                state.state.previous_radius + state.state.config.gap_between_points + state.state.radius,
            ),
        );

        state.state.previous_translation = point_translation;

        state.state.previous_radius = state.state.radius;
    });

    operation!(rigid_body, rigid_body: RigidBody {
        rigid_body.state.config.rigid_body = rigid_body.rigid_body;
    });

    operation!([gravity_override] operation, gravity_override: Vec3 {
        // TO DO: Should this operation take in an Option<Vec3>?
        operation.state.config.gravity_override = Some(operation.gravity_override);
    });

    operation!([mesh] operation, mesh: ChainPointConfigured {
        operation.state.radius = operation.mesh.radius;
        operation.state.mesh = operation.asset_server.load(format!("{}/mesh.glb#Scene0", operation.mesh.mesh));
    });

    operation!([gap_between_points] operation, gap_between_points: f32 {
        operation.state.config.gap_between_points = operation.gap_between_points;
    });

    operation!([alter] operation, alter: fn(&mut EntityCommands<'_>) {
        operation.state.config.alter = operation.alter;
    });
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
