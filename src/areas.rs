use avian3d::prelude::Sensor;
use bevy::prelude::*;

macro_rules! areas {
    ($($areas:ident),*) => {
        $(
            pub mod $areas;
        )*

        fn area_plugins(app: &mut App) {
            app.add_plugins(($($areas::plugin),*));
        }

        const AREAS: &[(&str, fn(&mut Commands))] = &[
            $(
                (const_str::concat!("map/", std::stringify!($areas), ".glb#Scene0"), $areas::load),
            )*
        ];
    };
}

areas!(test_area);

pub fn plugin(app: &mut App) {
    area_plugins(app);
    app.add_systems(Startup, temp_load_all);
}

fn temp_load_all(asset_server: Res<AssetServer>, mut commands: Commands) {
    for (path, load) in AREAS {
        let scene = asset_server.load(*path);
        commands.spawn(SceneRoot(scene));
        load(&mut commands);
    }
}

#[derive(Component)]
#[require(Sensor)]
pub struct LoadArea;


