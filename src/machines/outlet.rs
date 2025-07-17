use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{
    error_handling::{ForEachFallible, ToFailure},
    machines::cable::Plug,
};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (within_range, out_of_range, connect));
}

#[derive(Component)]
#[require(Sensor, CollisionEventsEnabled)]
pub struct OutletSensor {
    pub root: Entity,
    pub rest_length: f32,
    pub plug: Option<Entity>,
}

fn within_range(
    mut outlet_sensor: Query<&OutletSensor>,
    mut plug: Query<&mut Plug>,
    mut collisions_started: EventReader<CollisionStarted>,
) {
    for CollisionStarted(entity_1, entity_2) in collisions_started.read() {
        let ((outlet_sensor_entity, outlet_sensor), (plug_entity, mut plug)) =
            match (outlet_sensor.get_mut(*entity_1), plug.get_mut(*entity_2)) {
                (Ok(outlet_sensor), Ok(plug)) => ((*entity_1, outlet_sensor), (*entity_2, plug)),
                (Err(_), Err(_)) => {
                    match (outlet_sensor.get_mut(*entity_2), plug.get_mut(*entity_1)) {
                        (Ok(outlet_sensor), Ok(plug)) => {
                            ((*entity_2, outlet_sensor), (*entity_1, plug))
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
    mut collisions_started: EventReader<CollisionEnded>,
) -> Result {
    collisions_started
        .read()
        .for_each_fallible(|CollisionEnded(entity_1, entity_2)| {
            let ((outlet_sensor_entity, outlet_sensor), (plug_entity, mut plug)) =
                match (outlet_sensor.get_mut(*entity_1), plug.get_mut(*entity_2)) {
                    (Ok(outlet_sensor), Ok(plug)) => {
                        ((*entity_1, outlet_sensor), (*entity_2, plug))
                    }
                    (Err(_), Err(_)) => {
                        match (outlet_sensor.get_mut(*entity_2), plug.get_mut(*entity_1)) {
                            (Ok(outlet_sensor), Ok(plug)) => {
                                ((*entity_2, outlet_sensor), (*entity_1, plug))
                            }
                            _ => return Ok(()),
                        }
                    }
                    _ => return Ok(()),
                };

            let index = plug
                .outlet_sensors_within_range
                .iter()
                .position(|entity| *entity == outlet_sensor_entity)
                .else_return()?;
            plug.outlet_sensors_within_range.swap_remove(index);
            Ok(())
        })
}

fn connect(
    mut plug: Query<(Entity, &mut Plug)>,
    mut outlet_sensor: Query<&mut OutletSensor>,
    mut commands: Commands,
) -> Result {
    plug.iter_mut()
        .for_each_fallible(|(plug_entity, mut plug)| {
            if plug.dragged || plug.outlet_sensor_connected_to.is_some() {
                return Ok(());
            }

            let outlet_sensor_entity = *plug.outlet_sensors_within_range.first().else_return()?;
            let mut outlet_sensor = outlet_sensor
                .get_mut(outlet_sensor_entity)
                .else_error("No outlet sensor.")?;

            outlet_sensor.plug = Some(plug_entity);
            plug.outlet_sensor_connected_to = Some(outlet_sensor_entity);
            commands.entity(plug.joint).insert(
                DistanceJoint::new(outlet_sensor.root, plug_entity)
                    .with_rest_length(outlet_sensor.rest_length)
                    .with_compliance(0.0025),
            );
            info!("Connected!");

            Ok(())
        })
}
