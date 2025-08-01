use crate::{
    error_handling::ToFailure,
    instantiate::{Config, GetOrInsert},
    machines::outlet::OutletSensor,
    mouse::{drag, Interactable},
    physics::CollisionLayer,
    propagate::Propagate,
    render::ComesFromRootEntity,
};
use avian3d::prelude::*;
use bevy::prelude::*;

pub fn plugin(_: &mut App) {
    //app.add_systems(Update, load);
}

pub struct CableConfig {
    pub length: u8,
    pub force_other_head: Option<Vec3>,
}

impl CableConfig {
    const PLUG_DENSITY: f32 = 25.;
    const PLUG_COMPLIANCE: f32 = 0.0001;

    const CABLE_RADIUS: f32 = 0.25 * 0.5;
    const CABLE_DENSITY: f32 = 10.;
    const CABLE_COMPLIANCE: f32 = 0.01;

    const MAX_DISTANCE: f32 = 0.2;
}

impl Config for CableConfig {
    fn instantiate<'a>(self, world: &mut World, root_entity: Entity) -> Result {
        let asset_server = world.resource::<AssetServer>();
        let plug_scene = asset_server.load("machines/plug.glb#Scene0");
        let cable_scene = asset_server.load("machines/cable.glb#Scene0");

        let transform = world
            .entity_mut(root_entity)
            .get_or_insert(Transform::default());

        let mut commands = world.commands();

        let collision_layers = CollisionLayers::new(
            [CollisionLayer::Cable, CollisionLayer::Floor],
            [CollisionLayer::Default, CollisionLayer::Floor],
        );

        let head_joint = commands.spawn_empty().id();
        let tail = commands.spawn_empty().id();

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
                MassPropertiesBundle::from_shape(&Cuboid::new(0.8, 0.4, 0.8), Self::PLUG_DENSITY),
                Collider::cuboid(0.8, 0.4, 0.8),
                collision_layers,
                SceneRoot(plug_scene.clone()),
                Propagate(ComesFromRootEntity(root_entity)),
                Interactable,
            ))
            .observe(drag)
            .observe(drag_start)
            .observe(drag_end)
            .id();

        let mut previous_transform = transform;
        previous_transform.translation.y -= 0.2 + Self::CABLE_RADIUS;
        let mut previous = commands
            .spawn((
                RigidBody::Dynamic,
                LockedAxes::ROTATION_LOCKED,
                MassPropertiesBundle::from_shape(
                    &Sphere::new(Self::CABLE_RADIUS),
                    Self::CABLE_DENSITY,
                ),
                Collider::sphere(Self::CABLE_RADIUS),
                collision_layers,
                SceneRoot(cable_scene.clone()),
                Propagate(ComesFromRootEntity(root_entity)),
                previous_transform,
            ))
            .id();

        commands.spawn(
            SphericalJoint::new(head, previous)
                .with_local_anchor_1(Vec3::NEG_Y * 0.2)
                .with_local_anchor_2(Vec3::Y * Self::CABLE_RADIUS)
                .with_compliance(Self::PLUG_COMPLIANCE),
        );
        commands.spawn(
            DistanceJoint::new(head, previous)
                .with_limits(0., Self::CABLE_RADIUS + 0.2 + Self::MAX_DISTANCE),
        );

        for i in 1..self.length {
            let mut transform = transform;
            transform.translation.y -= 0.2 + Self::CABLE_RADIUS;
            transform.translation.x += f32::from(i) * Self::CABLE_RADIUS * 2.;

            let mut cable = commands.spawn((
                RigidBody::Dynamic,
                LockedAxes::ROTATION_LOCKED,
                MassPropertiesBundle::from_shape(
                    &Sphere::new(Self::CABLE_RADIUS),
                    Self::CABLE_DENSITY,
                ),
                SceneRoot(cable_scene.clone()),
                Propagate(ComesFromRootEntity(root_entity)),
                transform,
            ));
            let current = cable.id();

            if i % 6 == 0 {
                cable.insert((Collider::sphere(Self::CABLE_RADIUS), collision_layers));
            } else {
                cable.insert(GravityScale(-0.01));
            }

            commands.spawn(
                SphericalJoint::new(previous, current)
                    .with_local_anchor_1(Vec3::NEG_Y * Self::CABLE_RADIUS)
                    .with_local_anchor_2(Vec3::Y * Self::CABLE_RADIUS)
                    .with_compliance(Self::CABLE_COMPLIANCE),
            );
            commands.spawn(
                DistanceJoint::new(previous, current)
                    .with_limits(0., Self::CABLE_RADIUS * 2. + Self::MAX_DISTANCE),
            );

            previous = current;
        }

        let tail_joint = commands.spawn_empty().id();

        let mut tail_transform = transform;
        tail_transform.translation.x += f32::from(self.length - 1) * Self::CABLE_RADIUS * 2.;

        if let Some(tail_translation) = self.force_other_head {
            tail_transform.translation = tail_translation;
        }

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
                MassPropertiesBundle::from_shape(&Cuboid::new(0.8, 0.4, 0.8), Self::PLUG_DENSITY),
                Collider::cuboid(0.8, 0.4, 0.8),
                collision_layers,
                SceneRoot(plug_scene.clone()),
                Propagate(ComesFromRootEntity(root_entity)),
                tail_transform,
                Interactable,
            ))
            .observe(drag)
            .observe(drag_start)
            .observe(drag_end)
            .id();

        commands.spawn(
            SphericalJoint::new(previous, tail)
                .with_local_anchor_1(Vec3::Y * Self::CABLE_RADIUS)
                .with_local_anchor_2(Vec3::NEG_Y * 0.2)
                .with_compliance(Self::PLUG_COMPLIANCE),
        );
        commands.spawn(
            DistanceJoint::new(previous, tail)
                .with_limits(0., Self::CABLE_RADIUS + 0.2 + Self::MAX_DISTANCE),
        );

        Ok(())
    }
}

#[derive(Component)]
pub struct Plug {
    pub outlet_sensors_within_range: Vec<Entity>,
    pub dragged: bool,
    pub outlet_sensor_connected_to: Option<Entity>,
    pub joint: Entity,
    pub other_end: Entity,
}

pub fn drag_start(
    drag_start: Trigger<Pointer<DragStart>>,
    mut plug: Query<&mut Plug>,
    mut outlet_sensor: Query<&mut OutletSensor>,
    mut commands: Commands,
) -> Result {
    let target = drag_start.target();
    let mut plug = plug
        .get_mut(target)
        .else_warn("Plug doesn't have a Plug.")?;

    plug.dragged = true;
    commands.entity(plug.joint).remove::<DistanceJoint>();

    let outlet_sensor_entity = plug.outlet_sensor_connected_to.take().else_return()?;
    let mut outlet_sensor = outlet_sensor
        .get_mut(outlet_sensor_entity)
        .else_error("No outlet sensor.")?;
    let position = outlet_sensor
        .plugs
        .iter()
        .position(|plug| *plug == target)
        .else_return()?;
    outlet_sensor.plugs.swap_remove(position);

    Ok(())
}

pub fn drag_end(drag_end: Trigger<Pointer<DragEnd>>, mut plug: Query<&mut Plug>) -> Result {
    let mut plug = plug
        .get_mut(drag_end.target())
        .else_warn("Plug doesn't have a Plug.")?;

    plug.dragged = false;

    Ok(())
}
