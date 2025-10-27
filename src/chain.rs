use std::ops::{Deref, DerefMut};

pub use crate::bevy_prelude::*;
use crate::physics::{Accelerate, CollisionLayer};
use avian3d::prelude::*;

pub const FUNGUS_PURPLE_GLOW: Point = Point::new("fungus_purple_glow", 0.05);

pub const SEAWEED: Point = Point::new("seaweed", 0.05);

pub mod fungus_a {
    use super::Point;

    pub const CAP: Point = Point::new("fungus_a/cap", 0.5); // * 0.1);
}

pub mod fungus_small_pot {
    use super::Point;

    pub const CAP: Point = Point::new("fungus_small_pot/cap", 0.04);
    pub const STEM: Point = Point::new("fungus_small_pot/stem", 0.02);
}

pub struct Point {
    mesh: &'static str,
    radius: f32,
}

impl Point {
    const fn new(mesh: &'static str, radius: f32) -> Self {
        Self { mesh, radius }
    }
}

impl ChainOperation for Point {
    fn internal_start(&self, asset_server: &AssetServer) -> State {
        State {
            mesh_handle: asset_server.load(format!("{}/mesh.glb#Scene0", self.mesh)),
            cheap_state: CheapState {
                previous_entity: Entity::PLACEHOLDER,
                previous_translation: Vec3::NAN,
                previous_radius: self.radius,

                mesh_path: self.mesh,

                radius: self.radius,
                gap_between_points: 0.,
                gravity_override: None,
                rigid_body: RigidBody::Dynamic,
                mass: 0.05,
                alter: |_| {},
            },
        }
    }

    fn internal_apply(self, _: &mut State, _: &mut Commands, _: &AssetServer) {}
}

/// The current state while running.
pub struct State {
    cheap_state: CheapState,
    mesh_handle: Handle<Scene>,
}

impl Deref for State {
    type Target = CheapState;

    fn deref(&self) -> &Self::Target {
        &self.cheap_state
    }
}

impl DerefMut for State {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cheap_state
    }
}

/// The state that is cheap to copy around.
#[derive(Clone, Copy)]
pub struct CheapState {
    previous_entity: Entity,
    previous_translation: Vec3,
    previous_radius: f32,

    mesh_path: &'static str,

    radius: f32,
    gap_between_points: f32,
    gravity_override: Option<Vec3>,
    rigid_body: RigidBody,
    mass: f32,
    // TO DO: This is both too flexible, and too inflexible. Replace when possible.
    alter: fn(&mut EntityCommands<'_>),
}

impl State {
    fn insert_point_bundle(
        &self,
        entity_commands: &mut EntityCommands,
        translation: Vec3,
        extra: impl Bundle,
    ) {
        entity_commands.insert((
            SceneRoot(self.mesh_handle.clone()),
            Transform::from_translation(translation),
            self.rigid_body,
            LockedAxes::ROTATION_LOCKED,
            Mass(self.mass),
            Collider::sphere(self.radius),
            SleepThreshold {
                linear: 0.01,
                ..default()
            },
            CollisionLayers::new(CollisionLayer::Cable, CollisionLayer::Default),
            LinearDamping(1.),
            extra,
        ));

        if let Some(gravity_override) = self.gravity_override {
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
                fn internal_start(&self, asset_server: &AssetServer) -> State {
                    self.previous.internal_start(asset_server)
                }

                fn internal_apply(
                    self,
                    state: &mut State,
                    commands: &mut Commands,
                    asset_server: &AssetServer,
                ) {
                    struct CurrentOperation<'sr, 'ar, 'cr, 'w, 's> {
                        state: &'sr mut State,
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
        operation!([$name] $name, $($parameter: $parameter_type),* {$($body)*});
    };
}

pub trait ChainOperation: Sized {
    fn internal_start(&self, asset_server: &AssetServer) -> State;

    fn internal_apply(self, state: &mut State, commands: &mut Commands, asset_server: &AssetServer);

    fn start(self, pos: Vec3) -> ChainOperationStart<Self> {
        ChainOperationStart {
            previous: self,
            translation: pos,
        }
    }

    fn run(self, asset_server: &AssetServer, commands: &mut Commands) -> ChainOperationFinished {
        let mut state = self.internal_start(asset_server);
        self.internal_apply(&mut state, commands, asset_server);

        ChainOperationFinished {
            state: state.cheap_state,
        }
    }

    operation!([to] state, pos: Vec3 {
        let direction = (state.pos - state.state.previous_translation).normalize_or_zero();

        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        let quantity = (state.pos.distance(state.state.previous_translation)
            / (state.state.radius + state.state.gap_between_points + state.state.radius))
            .floor() as u16;

        let last_final_translation = state.state.previous_translation;

        for i in 0..quantity {
            // radius gap radius?
            let point_translation = (state.state.previous_radius
                + state.state.gap_between_points
                + state.state.radius
                + (f32::from(i) * (state.state.radius + state.state.gap_between_points + state.state.radius)))
                * direction
                + last_final_translation;

            let saved_previous_entity = state.state.previous_entity;

            let mut previous_entity = state.commands.spawn_empty();
            state.state.insert_point_bundle(&mut previous_entity, point_translation, ());
            (state.state.alter)(&mut previous_entity);
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
                        state.state.previous_radius + state.state.gap_between_points + state.state.radius,
                    ),
                );
            } else {
                state.commands.spawn(
                    DistanceJoint::new(saved_previous_entity, state.state.previous_entity)
                        .with_limits(0., state.state.radius * 2. + state.state.gap_between_points),
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
            (state.state.previous_radius + state.state.gap_between_points + state.state.radius)
                * state.dir
                + last_final_translation;

        let saved_previous_entity = state.state.previous_entity;

        let mut previous_entity = state.commands.spawn_empty();
        state.state.insert_point_bundle(&mut previous_entity, point_translation, ());
        (state.state.alter)(&mut previous_entity);
        state.state.previous_entity = previous_entity.id();

        state.commands.spawn(
            DistanceJoint::new(saved_previous_entity, state.state.previous_entity).with_limits(
                0.,
                state.state.previous_radius + state.state.gap_between_points + state.state.radius,
            ),
        );

        state.state.previous_translation = point_translation;

        state.state.previous_radius = state.state.radius;
    });

    operation!(rigid_body, rigid_body: RigidBody {
        rigid_body.state.rigid_body = rigid_body.rigid_body;
    });

    operation!([gravity_override] operation, gravity_override: Vec3 {
        // TO DO: Should this operation take in an Option<Vec3>?
        operation.state.gravity_override = Some(operation.gravity_override);
    });

    operation!([mesh] operation, mesh: Point {
        operation.state.radius = operation.mesh.radius;
        operation.state.mesh_handle = operation.asset_server.load(format!("{}/mesh.glb#Scene0", operation.mesh.mesh));
    });

    operation!([gap_between_points] operation, gap_between_points: f32 {
        operation.state.gap_between_points = operation.gap_between_points;
    });

    operation!([alter] operation, alter: fn(&mut EntityCommands<'_>) {
        operation.state.alter = operation.alter;
    });
}

pub struct ChainOperationStart<P> {
    previous: P,
    translation: Vec3,
}

impl<P: ChainOperation> ChainOperation for ChainOperationStart<P> {
    fn internal_start(&self, asset_server: &AssetServer) -> State {
        self.previous.internal_start(asset_server)
    }

    fn internal_apply(self, state: &mut State, commands: &mut Commands, _: &AssetServer) {
        state.previous_translation = self.translation;
        let mut previous_entity = commands.spawn_empty();
        state.insert_point_bundle(&mut previous_entity, self.translation, ());
        (state.alter)(&mut previous_entity);
        state.previous_entity = previous_entity.id();
    }
}

#[derive(Clone, Copy)]
pub struct ChainOperationFinished {
    state: CheapState,
}

impl ChainOperation for ChainOperationFinished {
    fn internal_start(&self, asset_server: &AssetServer) -> State {
        State {
            cheap_state: self.state,
            mesh_handle: asset_server.load(format!("{}/mesh.glb#Scene0", self.state.mesh_path)),
        }
    }

    fn internal_apply(self, _: &mut State, _: &mut Commands, _: &AssetServer) {}
}
