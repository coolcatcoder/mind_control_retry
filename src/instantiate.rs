use bevy::{
    ecs::query::{QueryData, ReadOnlyQueryData},
    prelude::*,
};

pub fn plugin(_: &mut App) {}

pub trait BundleThatCanBeQueried<const OVERLAP: bool = true>: Bundle {
    type Query<'a>: ReadOnlyQueryData;

    fn components_to_bundle(components: <Self::Query<'_> as QueryData>::Item<'_>) -> Self;
}

impl<A: Component + Clone> BundleThatCanBeQueried<false> for A {
    type Query<'a> = &'a A;

    fn components_to_bundle(components: <Self::Query<'_> as QueryData>::Item<'_>) -> Self {
        components.clone()
    }
}

impl<A: Component + Clone, B: Component + Clone> BundleThatCanBeQueried for (A, B) {
    type Query<'a> = (&'a A, &'a B);

    fn components_to_bundle(components: <Self::Query<'_> as QueryData>::Item<'_>) -> Self {
        (components.0.clone(), components.1.clone())
    }
}

pub trait GetOrInsert {
    fn get_or_insert<T: BundleThatCanBeQueried<OVERLAP>, const OVERLAP: bool>(
        &mut self,
        default_bundle: T,
    ) -> T;
}

impl GetOrInsert for EntityWorldMut<'_> {
    fn get_or_insert<T: BundleThatCanBeQueried<OVERLAP>, const OVERLAP: bool>(
        &mut self,
        default_bundle: T,
    ) -> T {
        let components = self
            .insert_if_new(default_bundle)
            .components::<T::Query<'_>>();
        T::components_to_bundle(components)
    }
}

pub trait Config {
    fn instantiate(self, world: &mut World, root_entity: Entity) -> Result;
}

pub trait InstantiateInto {
    fn instantiate<T: Config + Send + Sync + 'static>(&'_ mut self, config: T) -> &mut Self;
}

impl InstantiateInto for EntityCommands<'_> {
    fn instantiate<T: Config + Send + Sync + 'static>(&'_ mut self, config: T) -> &mut Self {
        let entity_move = self.id();

        self.commands_mut()
            .queue(move |world: &mut World| -> Result { config.instantiate(world, entity_move) });

        self
    }
}
