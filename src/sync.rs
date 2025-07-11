use crate::error_handling::ToFailure;
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(
        PostUpdate,
        sync_translation.before(TransformSystem::TransformPropagate),
    );
}

/// Syncs a translation to another entity, if it exists and has the component.
/// Will not sync to a translation which is also syncing.
#[derive(Component)]
#[require(Transform)]
pub struct SyncTranslation {
    pub target: Entity,
    pub offset: Vec3,
}

fn sync_translation(
    sync: Query<(&SyncTranslation, &mut Transform)>,
    target: Query<&Transform, Without<SyncTranslation>>,
) -> Result {
    for (sync, mut transform) in sync {
        let target = target.get(sync.target).else_return()?;
        transform.translation = target.translation + sync.offset;
    }
    Ok(())
}
