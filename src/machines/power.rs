use bevy::prelude::*;

use crate::{
    error_handling::{ForEachFallible, ToFailure},
    machines::{
        cable::Plug,
        outlet::{OutletSensor, OutletSensorEntity},
    },
};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, drain);
}

/// Energy as a percentage.
/// Everything has the same capacity.
#[derive(Component)]
pub struct Energy(pub u8);

/// How much energy is taken per second, and whether the demand is met.
#[derive(Component)]
pub struct TakesPower(pub u8);

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
