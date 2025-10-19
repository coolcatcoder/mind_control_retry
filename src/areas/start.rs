use avian3d::prelude::*;

pub use crate::bevy_prelude::*;
use crate::{
    areas::{Area, AreaLoadedEntity},
    editor::editor,
};

pub const DEVELOP_OVERRIDE: bool = true;

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
        color: Color::srgb(0.46, 0.26, 0.63),
        brightness: 60.,
        ..default()
    });
}

fn full_patch(on: On<AreaLoadedEntity>, mut loaded: Query<&Name>, mut commands: Commands) {
    let name = loaded
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
        _ => (),
    }
}
