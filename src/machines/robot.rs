use crate::{
    error_handling::ToFailure,
    instantiate::Config,
    lost::{change_range, move_towards_single_axis},
    physics::common_properties::AIR_RESISTANCE,
    propagate::Propagate,
    render::ComesFromRootEntity,
};
use avian3d::prelude::*;
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
                AIR_RESISTANCE,
                GravityScale(0.),
                Mass(70.),
                LockedAxes::ROTATION_LOCKED.unlock_rotation_y(),
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
    let time_delta = time.delta_secs();

    robot.iter_mut().for_each(
        |(entity, mut linear_velocity, mut angular_velocity, transform)| {
            if let Some(hit) = spatial_query.cast_shape(
                &Collider::cuboid(1., 0.2, 2.),
                transform.translation,
                transform.rotation,
                Dir3::NEG_Y,
                &ShapeCastConfig {
                    max_distance: 20.,
                    ignore_origin_penetration: true,
                    ..default()
                },
                &SpatialQueryFilter::from_excluded_entities([entity]),
            ) {
                let desired_y = hit.point1.y + 2.;
                let current_y = transform.translation.y;

                let distance = (desired_y - current_y).abs();
                let speed = change_range((0., 0.5), (0., 2.), distance).unwrap_or(2.);
                info!(speed);

                move_towards_single_axis(
                    hit.point1.y + 2.,
                    transform.translation.y,
                    speed,
                    2.,
                    time_delta,
                    &mut linear_velocity.y,
                );
            }

            // rotate_towards(
            //     Quat::IDENTITY,
            //     transform.rotation,
            //     &mut angular_velocity,
            //     time_delta,
            // );
            // angular_velocity.0 = angular_velocity.0.min(Vec3::splat(3.));
            // rotate_towards_new(
            //     Quat::IDENTITY,
            //     1.,
            //     transform.rotation,
            //     &mut angular_velocity,
            //     time_delta,
            // );
        },
    );
}
