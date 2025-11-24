pub use bevy::prelude::*;
use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    scene::SceneInstanceReady,
};

mod feathers;

plugin_modules!(pub start, pub grave_yard);

pub fn plugin(app: &mut App) {
    app.add_plugins(plugins_in_modules);
}

#[derive(Component)]
#[component(on_add = Self::on_add)]
pub struct LoadedFromArea(pub Entity);

impl LoadedFromArea {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let area = world
            .entity(context.entity)
            .get::<Self>()
            .else_error("It just got added, how on earth is there not a LoadedFromArea?")?
            .0;
        world.commands().trigger(AreaLoadedEntity {
            area,
            loaded: context.entity,
        });
    }
}

#[derive(Component)]
#[component(on_add = Self::on_add)]
pub struct Area;

impl Area {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        world.commands().entity(context.entity).observe(Self::load);
    }

    fn load(on: On<SceneInstanceReady>, children: Query<&Children>, mut commands: Commands) {
        info!("Done!");
        let scene_children = children
            .get(on.entity)
            .else_error("Failed to get scene children.")?;
        if scene_children.len() != 1 {
            error!("There should only be one child for SceneInstance entities.");
            return;
        }
        let scene_child = scene_children.iter().next().else_return()?;

        children.iter_descendants(scene_child).for_each(|child| {
            commands.entity(child).try_insert(LoadedFromArea(on.entity));
        });
    }
}

/// Make an area observe this in order to patch entities that are loaded.
#[derive(EntityEvent)]
pub struct AreaLoadedEntity {
    #[event_target]
    pub area: Entity,
    pub loaded: Entity,
}
