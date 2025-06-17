use super::{BasicHorizontalControl, Creature, LandHandling, LandHandlingState};
use avian3d::prelude::*;
use bevy::{
    ecs::{component::HookContext, world::DeferredWorld},
    prelude::*,
};
use bevy_mod_outline::{AsyncSceneInheritOutline, OutlineMode, OutlineVolume};

pub fn plugin(_: &mut App) {}

#[derive(Component)]
#[component(on_add = Tester::on_add)]
#[require(Transform, Creature)]
pub struct Tester;

impl Tester {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let asset_server = world.resource::<AssetServer>();
        let scene = asset_server.load("creatures/tester.glb#Scene0");

        world.commands().entity(context.entity).insert((
            SceneRoot(scene),
            RigidBody::Dynamic,
            Collider::compound(vec![(
                Vec3::Y * 0.5,
                Quat::default(),
                Collider::cuboid(1., 2., 1.),
            )]),
            // Friction {
            //     dynamic_coefficient: 0.25,
            //     static_coefficient: 1.,
            //     ..default()
            // },
            LockedAxes::ROTATION_LOCKED,
            LandHandling {
                state: LandHandlingState::InControl,
                gain_control: 3.,
                lose_control: 13.,
                slowing: 0.05,
            },
            BasicHorizontalControl { speed: 10. },
            OutlineVolume {
                visible: false,
                colour: Color::srgb(0., 1., 1.),
                width: 3.,
            },
            OutlineMode::ExtrudeFlat,
            AsyncSceneInheritOutline::default(),
        ));
    }
}
