use crate::{
    error_handling::ToFailure, instantiate::Config, propagate::Propagate,
    render::ComesFromRootEntity,
};
use avian3d::prelude::{
    AngularVelocity, Collider, LinearVelocity, MassPropertiesBundle, RigidBody, ShapeCastConfig,
    SpatialQuery, SpatialQueryFilter,
};
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, hover);
}

pub struct RobotConfig;

impl Config for RobotConfig {
    fn instantiate(self, world: &mut World, root_entity: Entity) -> Result {
        let asset_server = world.resource::<AssetServer>();
        let scene = asset_server.load("machines/robot.glb#Scene0");

        world
            .get_entity_mut(root_entity)
            .else_error("No root entity.")?
            .insert((
                Robot,
                SceneRoot(scene),
                Propagate(ComesFromRootEntity(root_entity)),
                RigidBody::Dynamic,
                MassPropertiesBundle::from_shape(&Cuboid::new(1., 2., 2.), 20.),
                Collider::cuboid(1., 2., 2.),
            ));

        Ok(())
    }
}

#[derive(Component)]
pub struct Robot;

fn hover(
    mut robot: Query<
        (
            Entity,
            &mut LinearVelocity,
            &mut AngularVelocity,
            &Transform,
        ),
        With<Robot>,
    >,
    spatial_query: SpatialQuery,
    time: Res<Time>,
) {
    let time = time.delta_secs();
    robot.iter_mut().for_each(
        |(entity, mut linear_velocity, mut angular_velocity, transform)| {
            if let Some(hit) = spatial_query.cast_shape(
                &Collider::cuboid(1., 0.2, 2.),
                transform.translation,
                transform.rotation,
                Dir3::NEG_Y,
                &ShapeCastConfig::from_max_distance(5.),
                &SpatialQueryFilter::from_excluded_entities([entity]),
            ) {
                let y_desired = hit.point1.y + 2.;
                let y_change = y_desired - transform.translation.y;
                linear_velocity.y = y_change * time * 1000.;
                linear_velocity.y = linear_velocity.y.min(10.);
            }

            //angular_velocity.
        },
    );
}
