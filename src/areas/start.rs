use avian3d::prelude::*;

pub use crate::bevy_prelude::*;
use crate::{
    areas::{Area, AreaLoadedEntity},
    chain::ChainConfig,
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

    let first = ChainConfig {
        mesh: "fungus_purple_glow",
        gap_between_points: -0.01,
        radius: 0.05,
        gravity_scale: 0.,
        rigid_body: RigidBody::Static,
        mass: 0.05,
        alter: |_| {},
    };

    let middle = ChainConfig {
        rigid_body: RigidBody::Dynamic,
        alter: |commands| {
            commands.insert(PointLight {
                color: Color::srgb(0.937, 0.149, 0.941),
                intensity: 1000.,
                radius: 0.5,
                range: 5.,
                shadows_enabled: false,
                ..default()
            });
        },
        ..first
    };

    let last = ChainConfig {
        rigid_body: RigidBody::Dynamic,
        gravity_scale: -1.,
        ..first
    };

    let mut stem = first.start(
        &mut commands,
        &asset_server,
        Vec3::new(-1.6973115, 0.41313428, 0.23011196),
    );
    stem.configure(&middle)
        .to(Vec3::new(-1.6973115, 1., 0.23011196))
        .configure(&last)
        .one();
    let stem_state = stem.state.clone();

    let branch_1 = ChainConfig {
        rigid_body: RigidBody::Dynamic,
        alter: |commands| {
            commands.insert(Accelerate(Vec3::new(-1.5, 1., 0.)));
        },
        ..first
    };
    stem.configure(&branch_1)
    .to(Vec3::new(-1.6973115 + -0.25, 2., 0.23011196));

    stem.state = stem_state.clone();
    let branch_2 = ChainConfig {
        rigid_body: RigidBody::Dynamic,
        alter: |commands| {
            commands.insert(Accelerate(Vec3::new(-1., 1.5, 0.)));
        },
        ..first
    };
    stem.configure(&branch_2)
    .to(Vec3::new(-1.6973115 + -0.25, 2.2, 0.23011196));

    stem.state = stem_state.clone();
    let branch_3 = ChainConfig {
        rigid_body: RigidBody::Dynamic,
        alter: |commands| {
            commands.insert(Accelerate(Vec3::new(1., 1.5, 0.)));
        },
        ..first
    };
    stem.configure(&branch_3)
    .to(Vec3::new(-1.6973115 + 0.25, 2.2, 0.23011196));

    stem.state = stem_state;
    stem.configure(&ChainConfig {
        rigid_body: RigidBody::Dynamic,
        alter: |commands| {
            commands.insert(Accelerate(Vec3::new(1.5, 1., 0.)));
        },
        ..first
    })
    .to(Vec3::new(-1.6973115 + 0.25, 2., 0.23011196));

    //.to(Vec3::new(2., 2., 1.));
}

fn full_patch(
    on: On<AreaLoadedEntity>,
    mut loaded: Query<(&Name, &Transform)>,
    mut commands: Commands,
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
        "mushroom" => {
            info!("{}", transform.translation);
            commands.entity(on.loaded).insert(PointLight {
                color: Color::srgb(0.937, 0.149, 0.941),
                intensity: 5000.,
                radius: 0.5,
                range: 5.,
                shadows_enabled: false,
                ..default()
            });
        }
        _ => (),
    }
}
