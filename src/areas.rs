use avian3d::prelude::{
    CollisionEnded, CollisionEventsEnabled, CollisionLayers, CollisionStarted, RigidBody, Sensor,
};
use bevy::prelude::*;

use crate::{error_handling::ToFailure, physics::CollisionLayer};

macro_rules! areas {
    ($($areas:ident),*) => {
        $(
            pub mod $areas;
        )*

        fn area_plugins(app: &mut App) {
            app.add_plugins(($($areas::plugin),*));
        }

        const AREAS: &[(&str, fn(&mut Commands))] = &[
            $(
                (const_str::concat!("map/", std::stringify!($areas), ".glb#Scene0"), $areas::load),
            )*
        ];
    };
}

areas!(test_area);

pub fn plugin(app: &mut App) {
    area_plugins(app);
    app.add_systems(Startup, temp_load_all)
        .add_systems(Update, (on_enter, on_exit));
}

fn temp_load_all(asset_server: Res<AssetServer>, mut commands: Commands) {
    for (path, load) in AREAS {
        let scene = asset_server.load(*path);
        commands.spawn(SceneRoot(scene));
        load(&mut commands);
    }
}

#[derive(Component)]
#[require(Sensor, CollisionEventsEnabled, RigidBody = RigidBody::Static)]
pub struct LoadArea;

fn on_enter(
    areas: Query<(), With<LoadArea>>,
    mut collision_layers: Query<&mut CollisionLayers>,
    mut collisions_started: EventReader<CollisionStarted>,
) -> Result {
    for CollisionStarted(entity_1, entity_2) in collisions_started.read() {
        let (_, collider) = match (areas.get(*entity_1), areas.get(*entity_2)) {
            (Ok(()), Err(_)) => (*entity_1, *entity_2),
            (Err(_), Ok(())) => (*entity_2, *entity_1),
            _ => continue,
        };

        let mut collision_layers = collision_layers.get_mut(collider).else_return()?;
        collision_layers.memberships.remove(CollisionLayer::Floor);
        collision_layers.filters.remove(CollisionLayer::Floor);
    }
    Ok(())
}

fn on_exit(
    areas: Query<(), With<LoadArea>>,
    mut collision_layers: Query<&mut CollisionLayers>,
    mut collisions_ended: EventReader<CollisionEnded>,
) -> Result {
    for CollisionEnded(entity_1, entity_2) in collisions_ended.read() {
        let (_, collider) = match (areas.get(*entity_1), areas.get(*entity_2)) {
            (Ok(()), Err(_)) => (*entity_1, *entity_2),
            (Err(_), Ok(())) => (*entity_2, *entity_1),
            _ => continue,
        };

        let mut collision_layers = collision_layers.get_mut(collider).else_return()?;
        collision_layers.memberships.add(CollisionLayer::Floor);
        collision_layers.filters.add(CollisionLayer::Floor);
    }
    Ok(())
}
