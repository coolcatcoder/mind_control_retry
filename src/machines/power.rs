use std::num::NonZero;

use bevy::prelude::*;

use crate::{
    error_handling::{ForEachFallible, ToFailure},
    machines::{
        cable::Plug,
        outlet::{OutletSensor, OutletSensorEntity},
    },
};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (drain, powered));
}

/// Energy as a percentage.
/// Everything has the same capacity.
#[derive(Component)]
pub struct Energy(pub u8);

/// How much energy is taken per second, and whether the demand is met.
#[derive(Component)]
pub struct TakesPower(pub u8);

/// A convenience component for entities that have `TakesPower` and want to know
/// if they do have power.
#[derive(Component)]
#[require(TakesPower = TakesPower(1))]
pub struct Powered(pub bool);

fn powered(
    mut powered: Query<(&mut Powered, &OutletSensorEntity)>,
    outlet_sensor: Query<&OutletSensor>,
    plug: Query<&Plug>,
    energy: Query<&Energy>,
) -> Result {
    powered
        .iter_mut()
        .for_each_fallible(|(mut powered, outlet_sensor_entity)| {
            let first_outlet_sensor = outlet_sensor
                .get(outlet_sensor_entity.0)
                .else_error("No outlet sensor.")?;

            if first_outlet_sensor.max_plugs != NonZero::<u8>::new(1) {
                error_once!("Max plugs must be one for those that need power.");
            }

            let Some(plug_entity) = first_outlet_sensor.plugs.first() else {
                powered.0 = false;
                return Ok(());
            };
            let first_plug = plug
                .get(*plug_entity)
                .else_error("Plug entity does not have plug.")?;
            let second_plug = plug
                .get(first_plug.other_end)
                .else_error("Second plug entity does not have plug.")?;

            let Some(second_outlet_sensor_entity) = second_plug.outlet_sensor_connected_to else {
                powered.0 = false;
                return Ok(());
            };
            let second_outlet_sensor = outlet_sensor
                .get(second_outlet_sensor_entity)
                .else_error("No outlet sensor.")?;
            if let Ok(energy) = energy.get(second_outlet_sensor.root)
                && energy.0 != 0
            {
                powered.0 = true;
            } else {
                powered.0 = false;
            }

            Ok(())
        })
}

fn drain(
    mut energy: Query<(&mut Energy, &OutletSensorEntity)>,
    outlet_sensors: Query<&OutletSensor>,
    plugs: Query<&Plug>,
    takes_power: Query<&TakesPower>,
    time: Res<Time>,
    mut time_left: Local<f32>,
) -> Result {
    let time_delta = time.delta_secs();
    *time_left -= time_delta;

    while *time_left <= 0. {
        *time_left += 1.;

        // For each fallible is okay, as if anything goes wrong, there will be an error.
        energy.iter_mut().for_each_fallible(
            |(mut energy, OutletSensorEntity(outlet_sensor_entity))| {
                let outlet_sensor = outlet_sensors
                    .get(*outlet_sensor_entity)
                    .else_error("Couldn't get outlet sensor entity.")?;

                let mut energy_to_remove = 0;
                for plug_entity in outlet_sensor.plugs.iter().copied() {
                    // Returning is okay, as it will error.
                    let plug = plugs
                        .get(plug_entity)
                        .else_error("No plug on plug entity?")?;
                    let other_end = plugs
                        .get(plug.other_end)
                        .else_error("No plug on plug entity?")?;

                    let Some(outlet_sensor_connected_to) = other_end.outlet_sensor_connected_to
                    else {
                        continue;
                    };

                    // Returning is okay, as it will error.
                    let outlet_sensor = outlet_sensors
                        .get(outlet_sensor_connected_to)
                        .else_error("No outlet sensor.")?;

                    let Ok(takes_power) = takes_power.get(outlet_sensor.root) else {
                        continue;
                    };

                    energy_to_remove += takes_power.0;
                }

                energy.0 = energy.0.saturating_sub(energy_to_remove);

                Ok(())
            },
        )?;
    }

    Ok(())
}
