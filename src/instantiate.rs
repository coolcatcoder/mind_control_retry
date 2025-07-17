use bevy::prelude::*;

pub fn plugin(_: &mut App) {}

pub trait Config {
    fn instantiate(self, world: &mut World, root_entity: Entity) -> Result;
}

pub trait Instantiate {
    fn instantiate<T: Config + Send + Sync + 'static>(
        &'_ mut self,
        config: T,
    ) -> EntityCommands<'_>;
}

impl Instantiate for Commands<'_, '_> {
    fn instantiate<T: Config + Send + Sync + 'static>(
        &'_ mut self,
        config: T,
    ) -> EntityCommands<'_> {
        let entity = self.spawn_empty().id();
        let entity_move = entity;

        self.queue(move |world: &mut World| -> Result { config.instantiate(world, entity_move) });

        self.entity(entity)
    }
}
