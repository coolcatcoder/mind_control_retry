// Stolen from bevy 0.17!
use core::marker::PhantomData;

use crate::{App, Plugin, Update};
use bevy::ecs::{
    component::Component,
    entity::Entity,
    hierarchy::ChildOf,
    query::{Changed, Or, QueryFilter, With, Without},
    relationship::{Relationship, RelationshipTarget},
    removal_detection::RemovedComponents,
    schedule::{IntoScheduleConfigs, SystemSet},
    system::{Commands, Local, Query},
};

/// Plugin to automatically propagate a component value to all direct and
/// transient relationship targets (e.g. [`bevy_ecs::hierarchy::Children`]) of
/// entities with a [`Propagate`] component.
///
/// The plugin Will maintain the target component over hierarchy changes, adding
/// or removing `C` when a relationship `R` (e.g. [`ChildOf`]) is added to or
/// removed from a relationship tree with a [`Propagate<C>`] source, or if the
/// [`Propagate<C>`] component is added, changed or removed.
///
/// Optionally you can include a query filter `F` to restrict the entities that
/// are updated. Note that the filter is not rechecked dynamically: changes to
/// the filter state will not be picked up until the  [`Propagate`] component is
/// touched, or the hierarchy is changed. All members of the tree between source
/// and target must match the filter for propagation to reach a given target.
/// Individual entities can be skipped or terminate the propagation with the
/// [`PropagateOver`] and [`PropagateStop`] components.
pub struct HierarchyPropagatePlugin<
    C: Component + Clone + PartialEq,
    F: QueryFilter = (),
    R: Relationship = ChildOf,
>(PhantomData<fn() -> (C, F, R)>);

/// Causes the inner component to be added to this entity and all direct and
/// transient relationship targets. A target with a [`Propagate<C>`] component
/// of its own will override propagation from that point in the tree.
#[derive(Component, Clone, PartialEq)]
pub struct Propagate<C: Component + Clone + PartialEq>(pub C);

/// Stops the output component being added to this entity.
/// Relationship targets will still inherit the component from this entity or
/// its parents.
#[derive(Component)]
pub struct PropagateOver<C>(PhantomData<fn() -> C>);

/// Stops the propagation at this entity. Children will not inherit the
/// component.
#[derive(Component)]
pub struct PropagateStop<C>(PhantomData<fn() -> C>);

/// The set in which propagation systems are added. You can schedule your logic
/// relative to this set.
#[derive(SystemSet, Clone, PartialEq, PartialOrd, Ord)]
pub struct PropagateSet<C: Component + Clone + PartialEq> {
    _p: PhantomData<fn() -> C>,
}

/// Internal struct for managing propagation
#[derive(Component, Clone, PartialEq)]
pub struct Inherited<C: Component + Clone + PartialEq>(pub C);

impl<C: Component + Clone + PartialEq, F: QueryFilter, R: Relationship> Default
    for HierarchyPropagatePlugin<C, F, R>
{
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<C> Default for PropagateOver<C> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<C> Default for PropagateStop<C> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<C: Component + Clone + PartialEq> core::fmt::Debug for PropagateSet<C> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PropagateSet")
            .field("_p", &self._p)
            .finish()
    }
}

impl<C: Component + Clone + PartialEq> Eq for PropagateSet<C> {}

impl<C: Component + Clone + PartialEq> core::hash::Hash for PropagateSet<C> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self._p.hash(state);
    }
}

impl<C: Component + Clone + PartialEq> Default for PropagateSet<C> {
    fn default() -> Self {
        Self { _p: PhantomData }
    }
}

impl<C: Component + Clone + PartialEq, F: QueryFilter + 'static, R: Relationship> Plugin
    for HierarchyPropagatePlugin<C, F, R>
{
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_source::<C>,
                update_stopped::<C, F>,
                update_reparented::<C, F, R>,
                propagate_inherited::<C, F, R>,
                propagate_output::<C, F>,
            )
                .chain()
                .in_set(PropagateSet::<C>::default()),
        );
    }
}

/// add/remove `Inherited::<C>` and `C` for entities with a direct
/// `Propagate::<C>`
pub fn update_source<C: Component + Clone + PartialEq>(
    mut commands: Commands,
    changed: Query<
        (Entity, &Propagate<C>),
        (
            Or<(Changed<Propagate<C>>, Without<Inherited<C>>)>,
            Without<PropagateStop<C>>,
        ),
    >,
    mut removed: RemovedComponents<Propagate<C>>,
) {
    for (entity, source) in &changed {
        commands
            .entity(entity)
            .try_insert(Inherited(source.0.clone()));
    }

    for removed in removed.read() {
        if let Ok(mut commands) = commands.get_entity(removed) {
            commands.remove::<(Inherited<C>, C)>();
        }
    }
}

/// remove `Inherited::<C>` and `C` for entities with a `PropagateStop::<C>`
pub fn update_stopped<C: Component + Clone + PartialEq, F: QueryFilter>(
    mut commands: Commands,
    q: Query<Entity, (With<Inherited<C>>, With<PropagateStop<C>>, F)>,
) {
    for entity in q.iter() {
        let mut cmds = commands.entity(entity);
        cmds.remove::<(Inherited<C>, C)>();
    }
}

/// add/remove `Inherited::<C>` and `C` for entities which have changed
/// relationship
pub fn update_reparented<C: Component + Clone + PartialEq, F: QueryFilter, R: Relationship>(
    mut commands: Commands,
    moved: Query<
        (Entity, &R, Option<&Inherited<C>>),
        (
            Changed<R>,
            Without<Propagate<C>>,
            Without<PropagateStop<C>>,
            F,
        ),
    >,
    relations: Query<&Inherited<C>>,
    orphaned: Query<Entity, (With<Inherited<C>>, Without<Propagate<C>>, Without<R>, F)>,
) {
    for (entity, relation, maybe_inherited) in &moved {
        if let Ok(inherited) = relations.get(relation.get()) {
            commands.entity(entity).try_insert(inherited.clone());
        } else if maybe_inherited.is_some() {
            commands.entity(entity).remove::<(Inherited<C>, C)>();
        }
    }

    for orphan in &orphaned {
        commands.entity(orphan).remove::<(Inherited<C>, C)>();
    }
}

/// add/remove `Inherited::<C>` for targets of entities with modified
/// `Inherited::<C>`
pub fn propagate_inherited<C: Component + Clone + PartialEq, F: QueryFilter, R: Relationship>(
    mut commands: Commands,
    changed: Query<
        (&Inherited<C>, &R::RelationshipTarget),
        (Changed<Inherited<C>>, Without<PropagateStop<C>>, F),
    >,
    recurse: Query<
        (Option<&R::RelationshipTarget>, Option<&Inherited<C>>),
        (Without<Propagate<C>>, Without<PropagateStop<C>>, F),
    >,
    mut removed: RemovedComponents<Inherited<C>>,
    mut to_process: Local<Vec<(Entity, Option<Inherited<C>>)>>,
) {
    // gather changed
    for (inherited, targets) in &changed {
        to_process.extend(
            targets
                .iter()
                .map(|target| (target, Some(inherited.clone()))),
        );
    }

    // and removed
    for entity in removed.read() {
        if let Ok((Some(targets), _)) = recurse.get(entity) {
            to_process.extend(targets.iter().map(|target| (target, None)));
        }
    }

    // propagate
    while let Some((entity, maybe_inherited)) = (*to_process).pop() {
        let Ok((maybe_targets, maybe_current)) = recurse.get(entity) else {
            continue;
        };

        if maybe_current == maybe_inherited.as_ref() {
            continue;
        }

        if let Some(targets) = maybe_targets {
            to_process.extend(
                targets
                    .iter()
                    .map(|target| (target, maybe_inherited.clone())),
            );
        }

        if let Some(inherited) = maybe_inherited {
            commands.entity(entity).try_insert(inherited.clone());
        } else {
            commands.entity(entity).remove::<(Inherited<C>, C)>();
        }
    }
}

/// add `C` to entities with `Inherited::<C>`
pub fn propagate_output<C: Component + Clone + PartialEq, F: QueryFilter>(
    mut commands: Commands,
    changed: Query<
        (Entity, &Inherited<C>, Option<&C>),
        (Changed<Inherited<C>>, Without<PropagateOver<C>>, F),
    >,
) {
    for (entity, inherited, maybe_current) in &changed {
        if maybe_current.is_some_and(|c| &inherited.0 == c) {
            continue;
        }

        commands.entity(entity).try_insert(inherited.0.clone());
    }
}
