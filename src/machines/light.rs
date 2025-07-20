use std::num::NonZero;

use avian3d::prelude::{Collider, RigidBody, Sensor};
use bevy::{
    ecs::{component::HookContext, world::DeferredWorld},
    prelude::*,
};

use crate::{
    machines::{outlet::OutletSensor, power::TakesPower},
    propagate::Propagate,
    render::SceneNotShadowCaster,
    sync::{SyncRotation, SyncTranslation},
};

pub fn plugin(_: &mut App) {}

#[derive(Component)]
#[require(Transform, RigidBody = RigidBody::Static)]
#[component(on_add = Self::on_add)]
pub struct LightBulb;

impl LightBulb {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let asset_server = world.resource::<AssetServer>();
        let scene = asset_server.load("machines/light.glb#Scene0");

        let mut commands = world.commands();

        commands.spawn((
            OutletSensor {
                root: context.entity,
                rest_length: 1.,
                plugs: vec![],
                max_plugs: NonZero::<u8>::new(1),
            },
            Collider::cuboid(2., 2., 2.),
            SyncTranslation {
                target: context.entity,
                offset: Vec3::ZERO,
            },
            SyncRotation {
                target: context.entity,
            },
        ));

        commands.entity(context.entity).insert((
            Propagate(SceneNotShadowCaster),
            SceneRoot(scene),
            Collider::cuboid(1., 1., 1.),
            PointLight {
                intensity: 100_000.0,
                range: 15.,
                color: Color::WHITE,
                shadows_enabled: true,
                ..default()
            },
            TakesPower(1),
        ));
    }
}

#[derive(Component)]
#[require(Sensor)]
pub struct LightArea;
