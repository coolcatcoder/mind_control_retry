use crate::{
    error_handling::{ForEachFallible, ToFailure},
    instantiate::{Config, InstantiateInto},
    machines::{
        outlet::{OutletSensor, OutletSensorEntity},
        power::Energy,
    },
    mouse::Interactable,
    propagate::Propagate,
    render::ComesFromRootEntity,
    sync::{SyncRotation, SyncTranslation},
};
use avian3d::prelude::{Collider, MassPropertiesBundle, RigidBody};
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (load, charge_indicator));
}

pub struct BatteryConfig {
    pub charge: u8,
}

impl Default for BatteryConfig {
    fn default() -> Self {
        Self { charge: 50 }
    }
}

impl Config for BatteryConfig {
    fn instantiate(self, world: &mut World, root_entity: Entity) -> Result {
        let asset_server = world.resource::<AssetServer>();
        let scene = asset_server.load("machines/battery.glb#Scene0");

        let mut commands = world.commands();

        // outlet connected
        let outlet_sensor_entity = commands
            .spawn((
                OutletSensor {
                    root: root_entity,
                    rest_length: 1.,
                    plugs: vec![],
                    max_plugs: None,
                },
                Collider::cuboid(2., 2., 2.),
                SyncTranslation {
                    target: root_entity,
                    offset: Vec3::ZERO,
                },
                SyncRotation {
                    target: root_entity,
                },
            ))
            .id();

        let light = PointLight {
            intensity: 500.0, // lumens
            range: 0.25,
            color: Srgba::rgb(0., 1., 0.).into(),
            shadows_enabled: false,
            ..default()
        };

        let top = commands
            .spawn((
                light,
                SyncTranslation {
                    target: root_entity,
                    offset: Vec3::new(0., 1. / 3., 0.4),
                },
            ))
            .id();

        let middle = commands
            .spawn((
                light,
                SyncTranslation {
                    target: root_entity,
                    offset: Vec3::new(0., 0., 0.4),
                },
            ))
            .id();

        let bottom = commands
            .spawn((
                light,
                SyncTranslation {
                    target: root_entity,
                    offset: Vec3::new(0., -(1. / 3.), 0.4),
                },
            ))
            .id();

        // battery
        commands.entity(root_entity).insert((
            BatteryLights {
                top,
                middle,
                bottom,
            },
            SceneRoot(scene),
            Propagate(ComesFromRootEntity(root_entity)),
            MassPropertiesBundle::from_shape(&Cuboid::new(1., 1., 1.), 10.),
            Collider::cuboid(1., 1., 1.),
            Battery,
            OutletSensorEntity(outlet_sensor_entity),
            Energy(self.charge),
        ));

        Ok(())
    }
}

#[derive(Component)]
#[require(Interactable, Transform, RigidBody = RigidBody::Static)]
pub struct Battery;

#[derive(Component)]
pub struct BatteryLights {
    top: Entity,
    middle: Entity,
    bottom: Entity,
}

fn load(extras: Query<(&GltfExtras, Entity), Added<GltfExtras>>, mut commands: Commands) -> Result {
    extras.iter().for_each_fallible(|(extras, entity)| {
        let extras_json = serde_json::from_str::<serde_json::Value>(&extras.value)
            .else_error("Gltf extras was not json.")?;
        let charge = u8::try_from(
            extras_json
                .get("battery")
                .else_return()?
                .as_u64()
                .else_return()?,
        )
        .else_error("Too much charge in battery.")?;

        info!("Spawned battery with charge: {charge}");

        commands
            .entity(entity)
            .instantiate(BatteryConfig { charge });

        Ok(())
    })
}

fn charge_indicator(
    mut battery: Query<(&Energy, &BatteryLights)>,
    mut visibility: Query<&mut Visibility>,
) -> Result {
    battery
        .iter_mut()
        .for_each_fallible(|(energy, battery_lights)| {
            let mut top_visibility = visibility
                .get_mut(battery_lights.top)
                .else_error("No visibility on light.")?;
            *top_visibility = if energy.0 >= 66 {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };

            let mut middle_visibility = visibility
                .get_mut(battery_lights.middle)
                .else_error("No visibility on light.")?;
            *middle_visibility = if energy.0 >= 33 {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };

            let mut bottom_visibility = visibility
                .get_mut(battery_lights.bottom)
                .else_error("No visibility on light.")?;
            *bottom_visibility = if energy.0 > 0 {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };

            Ok(())
        })
}
