use crate::{
    areas::LoadedFromArea,
    machines::outlet::OutletSensor,
    mouse::{Interactable, drag::Dragged, selection::SelectOthers},
    physics::CollisionLayer,
    render::ComesFromRootEntity,
};
use avian3d::prelude::*;
use bevy::{app::Propagate, prelude::*};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, load);
}

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

#[derive(Component)]
pub struct Plug {
    pub outlet_sensors_within_range: Vec<Entity>,
    pub dragged: bool,
    pub outlet_sensor_connected_to: Option<Entity>,
    pub joint: Entity,
    pub other_end: Entity,
}

pub fn drag_start(
    drag_start: On<Pointer<DragStart>>,
    mut plug: Query<&mut Plug>,
    mut outlet_sensor: Query<&mut OutletSensor>,
    mut commands: Commands,
) {
    let target = drag_start.event().event_target();
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
}

pub fn drag_end(drag_end: On<Pointer<DragEnd>>, mut plug: Query<&mut Plug>) {
    let mut plug = plug
        .get_mut(drag_end.event().event_target())
        .else_warn("Plug doesn't have a Plug.")?;

    plug.dragged = false;
}
