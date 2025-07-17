// TODO: Anything in here must be removed eventually.
use crate::{creatures::tester::Tester, mind_control::Controlled, physics::CollisionLayer};
use avian3d::prelude::CollisionLayers;
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, testing);
}

fn testing(mut commands: Commands) {
    commands.spawn((
        Tester,
        Controlled,
        CollisionLayers::new(
            [CollisionLayer::Default, CollisionLayer::Floor],
            [
                CollisionLayer::Default,
                CollisionLayer::Floor,
                CollisionLayer::Cable,
            ],
        ),
        Transform::from_xyz(10., 5., 0.),
    ));
    commands.spawn((
        Tester,
        Controlled,
        CollisionLayers::new(
            [CollisionLayer::Default, CollisionLayer::Floor],
            [
                CollisionLayer::Default,
                CollisionLayer::Floor,
                CollisionLayer::Cable,
            ],
        ),
        Transform::from_xyz(-5., 0.5, 0.),
    ));
}

// trait TwoQueriesGet {
//     type OutputOne;
//     type OutputTwo;
//     fn get() -> Result<((Entity, Self::OutputOne), (Entity, Self::OutputTwo)), ()>;
// }

// impl<'a, D1: QueryData, F1: QueryFilter, D2: QueryData, F2: QueryFilter> TwoQueriesGet for (&'a Query<'_, '_, D, F>) {
//     type Data = D;
//     type Filter = F;
//     type Output = ROQueryItem<'a, D>;
//     fn get(self, entity: Entity) -> Result<Self::Output, QueryEntityError> {
//         self.get(entity)
//     }
// }

// trait QueryReference {
//     type Data: QueryData;
//     type Filter: QueryFilter;
//     type Output;
//     fn get(self, entity: Entity) -> Result<Self::Output, QueryEntityError>;
// }

// impl<'a, D: QueryData, F: QueryFilter> QueryReference for &'a Query<'_, '_, D, F> {
//     type Data = D;
//     type Filter = F;
//     type Output = ROQueryItem<'a, D>;
//     fn get(self, entity: Entity) -> Result<Self::Output, QueryEntityError> {
//         self.get(entity)
//     }
// }

// impl<'a, D: QueryData, F: QueryFilter> QueryReference for &'a mut Query<'_, '_, D, F> {
//     type Data = D;
//     type Filter = F;
//     type Output = D::Item<'a>;
//     fn get(self, entity: Entity) -> Result<Self::Output, QueryEntityError> {
//         self.get_mut(entity)
//     }
// }

// fn which_is_which<Q1: QueryReference, Q2: QueryReference>(
//     entity_one: Entity,
//     entity_two: Entity,
//     query_one: Q1,
//     query_two: Q2,
// ) {
//     let ((output_one_entity, output_one), (output_two_entity, output_two)) =
//         match (query_one.get(entity_one), query_two.get(entity_two)) {
//             (Ok(output_one), Ok(output_two)) => ((entity_one, output_one), (entity_two, output_two)),
//             (Err(_), Err(_)) => match (query_one.get(entity_two), query_two.get(entity_one)) {
//                     (Ok(output_one), Ok(output_two)) => ((entity_two, output_one), (entity_one, output_two)),
//                     _ => return,
//                 }

//             _ => return,
//         };
// }
