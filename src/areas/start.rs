use avian3d::prelude::*;

pub use crate::bevy_prelude::*;
use crate::{
    areas::{Area, AreaLoadedEntity},
    chain::{ChainOperation, FUNGUS_PURPLE_GLOW, SEAWEED, fungus_a},
    editor::editor,
    physics::Accelerate,
};

pub const DEVELOP_OVERRIDE: bool = false;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, load);

    if DEVELOP_OVERRIDE || crate::DEVELOP {
        editor(file!())(app);
    }
}

fn load(asset_server: Res<AssetServer>, mut commands: Commands) {
    let scene = asset_server.load("map/room_1.glb#Scene0");
    commands.spawn((SceneRoot(scene), Area)).observe(full_patch);

    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.937, 0.149, 0.941),
        brightness: 60.,
        ..default()
    });
}

fn full_patch(
    on: On<AreaLoadedEntity>,
    mut loaded: Query<(&Name, &Transform)>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let (name, transform) = loaded
        .get_mut(on.loaded)
        .else_error("Could not get components on loaded entity.")?;

    //patch(name, transform);

    #[allow(clippy::match_same_arms)]
    #[allow(clippy::unreadable_literal)]
    #[allow(clippy::single_match)]
    match name.as_str() {
        "container" => {
            commands.entity(on.loaded).insert((
                Collider::cylinder(0.3, 0.6),
                Mass(8.1),
                RigidBody::Dynamic,
            ));
        }
        "mushroom pot" => {
            commands.entity(on.loaded).insert(PointLight {
                //color: Color::srgb(0.937, 0.149, 0.941),
                color: Color::srgb(1., 1., 1.),
                intensity: 10000.,
                //intensity: 5000.,
                radius: 0.5,
                range: 5.,
                shadows_enabled: false,
                ..default()
            });

            let translation = transform.translation + Vec3::new(0., 0.1, 0.);

            let base = SEAWEED
                .rigid_body(RigidBody::Static)
                .start(translation)
                .rigid_body(RigidBody::Dynamic)
                .gravity_override(Vec3::ZERO)
                .run(&asset_server, &mut commands);

            base.clone()
                .to(translation + Vec3::Y * 1.5)
                .gravity_override(Vec3::Y * 1.5)
                .one(Vec3::Y)
                .run(&asset_server, &mut commands);

            base.clone()
                .to(translation + Vec3::Y * 1.5)
                .gravity_override(Vec3::Y * 1.5)
                .one(Vec3::Y)
                .run(&asset_server, &mut commands);

            base.clone()
                .to(translation + Vec3::Y * 1.5)
                .gravity_override(Vec3::Y * 1.5)
                .one(Vec3::Y)
                .run(&asset_server, &mut commands);

            // let stem = FUNGUS_PURPLE_GLOW
            //     .rigid_body(RigidBody::Static)
            //     .start(translation)
            //     .rigid_body(RigidBody::Dynamic)
            //     .gravity_override(Vec3::ZERO)
            //     .to(translation + Vec3::Y * 0.5)
            //     .gravity_override(Vec3::Y * 1.5)
            //     .one(Vec3::Y)
            //     .gravity_override(Vec3::ZERO)
            //     .run(&asset_server, &mut commands);

            // stem.clone()
            //     .to(translation + Vec3::new(0., 1., 0.5))
            //     .gravity_override(Vec3::new(-0.5, 1., 1.5))
            //     .mesh(fungus_a::CAP)
            //     .one(Vec3::Y)
            //     .run(&asset_server, &mut commands);

            // stem.clone().to(translation + Vec3::new(0.5, 1.3, 0.))
            //     .gravity_override(Vec3::new(1.5, 1., 0.))
            //     .mesh(fungus_a::CAP)
            //     .one(Vec3::Y)
            //     .run(&asset_server, &mut commands);

            // stem.to(translation + Vec3::Y * 1.5)
            //     .gravity_override(Vec3::new(0., 1.5, 0.))
            //     .mesh(fungus_a::CAP)
            //     .one(Vec3::Y)
            //     .run(&asset_server, &mut commands);
        }
        _ => (),
    }
}
