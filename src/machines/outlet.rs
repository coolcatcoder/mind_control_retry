use std::num::NonZero;

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::machines::cable::Plug;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (within_range, out_of_range, connect));
}

#[derive(Component)]
pub struct OutletSensorEntity(pub Entity);

#[derive(Component)]
#[require(Sensor, CollisionEventsEnabled)]
pub struct OutletSensor {
    pub root: Entity,
    pub rest_length: f32,
    pub plugs: Vec<Entity>,
    pub max_plugs: Option<NonZero<u8>>,
}

fn within_range(
    mut outlet_sensor: Query<&OutletSensor>,
    mut plug: Query<&mut Plug>,
    mut collisions_started: MessageReader<CollisionStart>,
) {
    for CollisionStart {
        collider1,
        collider2,
        ..
    } in collisions_started.read()
    {
        let ((outlet_sensor_entity, _), (_, mut plug)) =
            match (outlet_sensor.get_mut(*collider1), plug.get_mut(*collider2)) {
                (Ok(outlet_sensor), Ok(plug)) => ((*collider1, outlet_sensor), (*collider2, plug)),
                (Err(_), Err(_)) => {
                    match (outlet_sensor.get_mut(*collider2), plug.get_mut(*collider1)) {
                        (Ok(outlet_sensor), Ok(plug)) => {
                            ((*collider2, outlet_sensor), (*collider1, plug))
                        }
                        _ => continue,
                    }
                }
                _ => continue,
            };

        // This shouldn't ever fail, but just in case, we check.
        if !plug
            .outlet_sensors_within_range
            .contains(&outlet_sensor_entity)
        {
            plug.outlet_sensors_within_range.push(outlet_sensor_entity);
        }
    }
}

fn out_of_range(
    mut outlet_sensor: Query<&OutletSensor>,
    mut plug: Query<&mut Plug>,
    mut collisions_started: MessageReader<CollisionEnd>,
) {
    collisions_started.read().for_each(
        |CollisionEnd {
             collider1,
             collider2,
             ..
         }| {
            let ((outlet_sensor_entity, _), (_, mut plug)) =
                match (outlet_sensor.get_mut(*collider1), plug.get_mut(*collider2)) {
                    (Ok(outlet_sensor), Ok(plug)) => {
                        ((*collider1, outlet_sensor), (*collider2, plug))
                    }
                    (Err(_), Err(_)) => {
                        match (outlet_sensor.get_mut(*collider2), plug.get_mut(*collider1)) {
                            (Ok(outlet_sensor), Ok(plug)) => {
                                ((*collider2, outlet_sensor), (*collider1, plug))
                            }
                            _ => return,
                        }
                    }
                    _ => return,
                };

            let index = plug
                .outlet_sensors_within_range
                .iter()
                .position(|entity| *entity == outlet_sensor_entity)
                .else_return()?;
            plug.outlet_sensors_within_range.swap_remove(index);
        },
    );
}

fn connect(
    mut plug: Query<(Entity, &mut Plug)>,
    mut outlet_sensor: Query<&mut OutletSensor>,
    mut commands: Commands,
) {
    plug.iter_mut().for_each(|(plug_entity, mut plug)| {
        if plug.dragged || plug.outlet_sensor_connected_to.is_some() {
            return;
        }

        let outlet_sensor_entity = *plug.outlet_sensors_within_range.first().else_return()?;
        let mut outlet_sensor = outlet_sensor
            .get_mut(outlet_sensor_entity)
            .else_error("No outlet sensor.")?;

        if let Some(max_plugs) = outlet_sensor.max_plugs
            && u8::from(max_plugs) as usize == outlet_sensor.plugs.len()
        {
            return;
        }

        outlet_sensor.plugs.push(plug_entity);
        plug.outlet_sensor_connected_to = Some(outlet_sensor_entity);
        commands.entity(plug.joint).insert(
            DistanceJoint::new(outlet_sensor.root, plug_entity)
                .with_limits(outlet_sensor.rest_length, outlet_sensor.rest_length)
                .with_compliance(0.),
        );
        info!("Connected!");
    });
}
