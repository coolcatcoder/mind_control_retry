use crate::{
    error_handling::{ForEachFallible, ToFailure},
    mouse::Interactable,
    sync::SyncTranslation,
};
use avian3d::prelude::{Collider, RigidBody};
use bevy::{
    ecs::{component::HookContext, world::DeferredWorld},
    prelude::*,
};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, load);
}

#[derive(Component)]
#[require(Interactable, Transform, RigidBody = RigidBody::Static)]
#[component(on_add = Self::on_add)]
pub struct Battery {
    charge: u8,
}

impl Default for Battery {
    fn default() -> Self {
        Self { charge: 50 }
    }
}

impl Battery {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let asset_server = world.resource::<AssetServer>();
        let scene = asset_server.load("machines/battery.glb#Scene0");

        let mut commands = world.commands();

        commands
            .entity(context.entity)
            .insert((SceneRoot(scene), Collider::cuboid(1., 1., 1.)));

        let light = PointLight {
            intensity: 500.0, // lumens
            range: 0.25,
            color: Srgba::rgb(0., 1., 0.).into(),
            shadows_enabled: false,
            ..default()
        };

        commands.spawn((
            light,
            SyncTranslation {
                target: context.entity,
                offset: Vec3::new(0., 1. / 3., 0.4),
            },
        ));

        commands.spawn((
            light,
            SyncTranslation {
                target: context.entity,
                offset: Vec3::new(0., 0., 0.4),
            },
        ));

        commands.spawn((
            light,
            SyncTranslation {
                target: context.entity,
                offset: Vec3::new(0., -(1. / 3.), 0.4),
            },
        ));
    }
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

        commands.entity(entity).insert(Battery { charge });

        Ok(())
    })
}
