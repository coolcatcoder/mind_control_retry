use crate::areas::LoadedFromArea;
use avian3d::prelude::*;
pub use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    //app.add_systems(Update, load);
}

fn load(names: Query<(Entity, &Name), Added<LoadedFromArea>>, mut commands: Commands) {
    for (entity, name) in names {
        if name.starts_with("collider") {
            commands.entity(entity).insert((
                ColliderConstructorHierarchy::new(ColliderConstructor::ConvexDecompositionFromMesh),
                RigidBody::Static,
            ));
        } else if name.starts_with("dynamic") {
            commands.entity(entity).insert((
                ColliderConstructorHierarchy::new(ColliderConstructor::ConvexDecompositionFromMesh),
                RigidBody::Dynamic,
            ));

            warn!("Automatic dynamic meshes are for temporary testing only. Name: {name}");
        }
    }
}
