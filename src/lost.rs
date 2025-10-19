// crime
#![allow(dead_code)]
#![allow(clippy::crate_in_macro_def)]
use std::{
    marker::PhantomData,
    ops::{Add, Deref, DerefMut, Div, Mul, Sub},
};

// TODO: Anything in here must be removed eventually.
use crate::physics::{CollisionLayer, common_properties::AIR_RESISTANCE};
use avian3d::prelude::{AngularVelocity, CollisionLayers, Mass, MassPropertiesBundle, RigidBody};
use bevy::{
    ecs::{
        lifecycle::HookContext,
        query::{QueryData, QueryFilter},
        system::SystemParam,
        world::DeferredWorld,
    },
    prelude::*,
};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, (testing, some_system));
}

fn testing(mut commands: Commands) {
    // commands.spawn((
    //     Tester,
    //     Controlled,
    //     CollisionLayers::new(
    //         [CollisionLayer::Default, CollisionLayer::Floor],
    //         [
    //             CollisionLayer::Default,
    //             CollisionLayer::Floor,
    //             CollisionLayer::Cable,
    //         ],
    //     ),
    //     Transform::from_xyz(10., 5., 0.),
    // ));
    // commands.spawn((
    //     Tester,
    //     Controlled,
    //     MassPropertiesBundle::from_shape(&Cuboid::new(0.3, 1., 0.3), 20.),
    //     CollisionLayers::new(
    //         [CollisionLayer::Default, CollisionLayer::Floor],
    //         [
    //             CollisionLayer::Default,
    //             CollisionLayer::Floor,
    //             CollisionLayer::Cable,
    //         ],
    //     ),
    //     Transform::from_xyz(-5., 0.5, 0.),
    // ));

    // let mut tester = commands.spawn(CollisionLayers::new(
    //     [CollisionLayer::Default, CollisionLayer::Floor],
    //     [
    //         CollisionLayer::Default,
    //         CollisionLayer::Floor,
    //         CollisionLayer::Cable,
    //     ],
    // ));
    // tester.join_group(PhysicsDynamic);

    //let cable = commands.spawn_ext(()).id();
    //let plug_1 = commands.spawn_ext(()).relegate_despawn(cable);

    let _plug_1 = commands.spawn(()).id();
    let _plug_2 = commands.spawn(()).id();
    let wire = commands.spawn(()).id();

    commands.spawn((Cable, ExternalComponents::<Wire, Cable>::on(wire)));
}

#[derive(Component)]
struct Cable;

#[derive(Component)]
struct Plug;

#[derive(Component)]
struct Wire;

fn cable_stuff(cables: Query<&ExternalComponents<Wire, Cable>>, wires: Query<&Wire>) {
    for wire in cables {
        //let wire = wire.get(&wires);
    }
}

trait Subset {}

//const TEST_TYPE: TypeId = TypeId::of::<i32>();

// You can not have a tuple struct nor unit struct with the same identifier as a
// constant, so this can only work with a regular struct.
// struct Something {
//     blah: bool,
// }
// // Once we have const traits, then you can hash a type id to get a unique
// // integer at compile time. (adt_const_params would allow the storage of
// TypeId // as a const directly, but it seems likely that const traits will
// stabilise // first.)
// const Something: u32 = 1;

// struct Other {}
// const Other: u32 = 2;

// // 0 is no
// struct WeirdTuple<
//     const A: u32 = 0,
//     const B: u32 = 0,
//     const C: u32 = 0,
//     const D: u32 = 0,
//     const E: u32 = 0,
//     const F: u32 = 0,
//     const G: u32 = 0,
// >;

// type Example = WeirdTuple<{ Something }, { Other }, { Something }>;

/// Signifies that a slot in the weird tuple is empty.
struct Empty;
struct EmptySlot;
/// A tuple struct of fixed arity, but with defaults for every generic, which
/// almost makes it seem like it can support any arity.
struct WeirdTuple<
    A = Empty,
    B = Empty,
    C = Empty,
    D = Empty,
    E = Empty,
    F = Empty,
    G = Empty,
    H = Empty,
>(A, B, C, D, E, F, G, H);

macro_rules! variadic {
    (
        struct $identifier:ident
        <$($generic:ident,)* ...Variadic>
        (
            $(
                $(PhantomData<(...$variadic_wrapped:ident)>)?
                $(...$variadic_field:ident)?
                $($field_type:ident)?
                ,
            )*
            $(,)?
        );
    ) => {
        struct $identifier<$($generic,)*
        Variadic=crate::lost::EmptySlot,
        VariadicB=crate::lost::EmptySlot,
        VariadicC=crate::lost::EmptySlot,
        VariadicD=crate::lost::EmptySlot,
        VariadicE=crate::lost::EmptySlot,
        VariadicF=crate::lost::EmptySlot,
        VariadicG=crate::lost::EmptySlot,
        VariadicH=crate::lost::EmptySlot,
        >(
            $(
                $(Variadic<(
                    $variadic_wrapped,
                    VariadicB,
                    VariadicC,
                    VariadicD,
                    VariadicE,
                    VariadicF,
                    VariadicG,
                    VariadicH,
                )>)?
                $(
                    $variadic_field,
                    VariadicB,
                    VariadicC,
                    VariadicD,
                    VariadicE,
                    VariadicF,
                    VariadicG,
                    VariadicH,
                )?
                $($field_type)?
            ),*
            // $($wrapper:ident<
            //     $variadic_wrapped,
            //     VariadicB,
            //     VariadicC,
            //     VariadicD,
            //     VariadicE,
            //     VariadicF,
            //     VariadicG,
            //     VariadicH,
            // >)?
        );
    };
}

#[derive(Component)]
struct TestComponent {
    bad: i32,
}

//variadic! {struct TestTuple<...Variadic>(PhantomData<...Variadic>,);}
//#[variadic]
//struct TestTuple<A, B>(A, B);

trait ComponentOrEmpty {}
impl ComponentOrEmpty for Empty {}
impl<T: Component> ComponentOrEmpty for T {}

/// Implemented on weird tuples where all non-empty slots implement Component.
trait WeirdBundle {}
impl<T: Component> WeirdBundle for T {}
#[procedural_macros::variadic]
impl<VariadicBad: ComponentOrEmpty + ComponentOrEmpty> WeirdBundle
    for WeirdTuple<Bad0, Bad1, Bad2, Bad3, Bad4, Bad5, Bad6, Bad7>
{
}

// Asserting that this all works.
const fn is_weird_bundle<T: WeirdBundle>() {}
const _: () = is_weird_bundle::<Transform>();
const _: () = is_weird_bundle::<WeirdTuple<Transform, Transform, Transform>>();

struct Data<A = Empty, B = Empty, C = Empty, D = Empty, E = Empty, F = Empty, G = Empty, H = Empty>(
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
);
struct Filter<
    A = Empty,
    B = Empty,
    C = Empty,
    D = Empty,
    E = Empty,
    F = Empty,
    G = Empty,
    H = Empty,
>(A, B, C, D, E, F, G, H);

trait ToData {
    type Output: QueryData;
}
impl<A: QueryData> ToData for Data<A> {
    type Output = A;
}
impl<A: QueryData, B: QueryData> ToData for Data<A, B> {
    type Output = (A, B);
}

trait ToFilter {
    type Output: QueryFilter;
}
impl ToFilter for Filter {
    type Output = ();
}

#[derive(SystemParam)]
struct WeirdQuery<'w, 's, D: ToData + 'static, F: ToFilter + 'static = Filter>(
    Query<'w, 's, <D as ToData>::Output, <F as ToFilter>::Output>,
)
where
    D::Output: 'static,
    F::Output: 'static;
impl<'w, 's, D: ToData + 'static, F: ToFilter + 'static> Deref for WeirdQuery<'w, 's, D, F> {
    type Target = Query<'w, 's, <D as ToData>::Output, <F as ToFilter>::Output>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<D: ToData + 'static, F: ToFilter + 'static> DerefMut for WeirdQuery<'_, '_, D, F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn some_system(query: WeirdQuery<Data<Entity, &Transform>>) {
    query.iter().for_each(|(entity, transform)| todo!());
}

// struct SomeComponentManager;

// impl Deref for SomeComponentManager {
//     type Target = fn(u32)->SomeComponent;
//     fn deref(&self) -> &Self::Target {
//         &((|zero| SomeComponent { zero, }) as fn(u32)->SomeComponent)
//     }
// }

// const SomeComponent: SomeComponentManager = SomeComponentManager;
// // Showing you can use it like a normal tuple struct:
// fn testing_some_component() {
//     let _: SomeComponent = SomeComponent(0);
// }

struct Weird<const A: u32 = 0, const B: u32 = 0>;

impl Weird<0, 0> {}

#[derive(Component)]
#[component(on_remove = Self::on_remove)]
pub struct ExternalComponents<C: Component, M> {
    target: Entity,
    //observer: Entity,
    phantom: PhantomData<(C, M)>,
}
impl<C: Component, M> ExternalComponents<C, M> {
    fn on(target: Entity) -> Self {
        Self {
            target,
            phantom: PhantomData,
        }
    }

    fn on_add(_world: DeferredWorld, _context: HookContext) {
        //let mut observer = Observer::new(|On: On<OnRemove, C>|
        // {}).watch_entity(entity); let mut commands =
        // world.commands();
    }
    fn on_remove(_world: DeferredWorld, _context: HookContext) {
        //world.commands()
    }
}

trait SpawnExt {
    fn spawn_ext<T: Bundle>(&mut self, bundle: T) -> EntityCommandsExt<'_, true>;
}

impl SpawnExt for Commands<'_, '_> {
    fn spawn_ext<T: Bundle>(&mut self, bundle: T) -> EntityCommandsExt<'_, true> {
        EntityCommandsExt(self.spawn(bundle))
    }
}

struct EntityCommandsExt<'a, const CAN_DESPAWN: bool>(EntityCommands<'a>);

impl<'a> EntityCommandsExt<'a, true> {
    // CAN_DESPAWN should be true or false, as both are correct!!!!
    fn relegate_despawn(self, bearer: EntityWithPermissions<true>) -> EntityCommandsExt<'a, false> {
        todo!("Give bearer the responsibility to despawn this entity.");
        EntityCommandsExt(self.0)
    }
}

// Wrong way? Create entities from bottom up instead perhaps?
//fn create_sub_entity(&mut EntityCommands) -> &mut EntityCommands {}

struct EntityWithPermissions<const CAN_DESPAWN: bool>(Entity);

trait JoinGroup {
    fn join_group<Group: ComponentGroup>(&mut self, group: Group);
}

impl JoinGroup for EntityCommands<'_> {
    fn join_group<Group: ComponentGroup>(&mut self, group: Group) {
        group.apply(self);
    }
}

trait ComponentGroup {
    fn apply(self, entity_commands: &mut EntityCommands<'_>);
}

struct PhysicsDynamic;
impl ComponentGroup for PhysicsDynamic {
    fn apply(self, entity_commands: &mut EntityCommands<'_>) {
        entity_commands.insert_if_new((RigidBody::Dynamic, AIR_RESISTANCE, Mass(10.)));
    }
}

pub fn change_range<
    T: Copy + PartialOrd + Sub<Output = T> + Div<Output = T> + Mul<Output = T> + Add<Output = T>,
>(
    from: (T, T),
    to: (T, T),
    value: T,
) -> Option<T> {
    if value < from.0 || value > from.1 {
        return None;
    }
    // From zero to (from.1 - from.0).
    let value_from_zero = value - from.0;
    // From zero to one.
    let value_from_zero_to_one = value_from_zero / (from.1 - from.0);
    // From zero to (to.1 - to.0).
    let value_from_zero = value_from_zero_to_one * (to.1 - to.0);
    // From to.0 to to.1.
    Some(value_from_zero + to.0)
}

pub fn move_towards_single_axis(
    desired_translation: f32,
    current_translation: f32,
    speed: f32,
    acceleration: f32,
    time_delta: f32,
    linear_velocity: &mut f32,
) {
    let desired_velocity = (desired_translation - current_translation).signum() * speed;
    let added_acceleration =
        (desired_velocity - *linear_velocity).signum() * acceleration * time_delta;
    //info!("added_acceleration: {added_acceleration}");
    *linear_velocity += added_acceleration;
}

pub fn rotate_towards_weird(
    desired_rotation_axis: Vec3,
    current_rotation_axis: Vec3,
    speed: f32,
    acceleration: f32,
    time_delta: f32,
    angular_velocity: &mut AngularVelocity,
) {
    let desired_velocity =
        (desired_rotation_axis - current_rotation_axis).normalize_or_zero() * speed;
    let added_acceleration =
        (desired_velocity - angular_velocity.0).normalize_or_zero() * acceleration * time_delta;
    //*angular_velocity
}

// All functions below taken from unity.
// https://discussions.unity.com/t/how-to-rotate-towards-a-direction-with-physicsvelocity/787239

pub fn estimate_angles_between(from: Quat, to: Quat) -> Vec3 {
    let from_imag = from.xyz();
    let to_imag = to.xyz();

    let mut angle = from_imag.cross(to_imag);
    angle -= to.w * from_imag;
    angle += from.w * to_imag;
    angle += angle;
    if to_imag.dot(from_imag).is_sign_negative() {
        -angle
    } else {
        angle
    }
}

pub fn rotate_towards(
    desired_rotation: Quat,
    current_rotation: Quat,
    angular_velocity: &mut AngularVelocity,
    time_delta: f32,
) {
    angular_velocity.0 =
        estimate_angles_between(current_rotation, desired_rotation) * time_delta.recip();
}
