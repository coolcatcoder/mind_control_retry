use avian3d::prelude::{Collider, RigidBody, Sensor};
use bevy::{
    ecs::{component::HookContext, world::DeferredWorld},
    prelude::*,
};

pub fn plugin(_: &mut App) {}

#[derive(Component)]
#[require(Transform, RigidBody = RigidBody::Static)]
#[component(on_add = Self::on_add)]
pub struct LightBulb;

impl LightBulb {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        //let asset_server = world.resource::<AssetServer>();
        //let scene = asset_server.load("machines/battery.glb#Scene0");

        let mut commands = world.commands();

        commands.entity(context.entity).insert((
            //SceneRoot(scene),
            Collider::cuboid(1., 1., 1.),
            PointLight {
                intensity: 100_000.0,
                range: 15.,
                color: Color::WHITE,
                shadows_enabled: true,
                ..default()
            },
        ));
    }
}

#[derive(Component)]
#[require(Sensor)]
pub struct LightArea;
