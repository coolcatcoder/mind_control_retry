use crate::{
    areas::AreaLoadedEntity,
    controls::{Action, Actions},
    creatures::{BasicHorizontalControl, LandHandling, LandHandlingState},
};
use avian3d::prelude::*;
pub use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_observer(load).add_systems(Update, control);
}

#[derive(Component)]
pub struct Player;

fn load(
    on: On<AreaLoadedEntity>,
    name: Query<&Name>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    // This code needs to be completely re-written.
    return;
    let name = name
        .get(on.loaded)
        .else_error("Couldn't get loaded entity's name.")?;
    name.starts_with("player").else_return()?;

    let scene = asset_server.load("machines/player.glb#Scene0");

    commands.entity(on.loaded).insert((
        Player,
        SceneRoot(scene),
        RigidBody::Dynamic,
        Collider::cuboid(0.3, 1., 0.3),
        // Friction {
        //     dynamic_coefficient: 0.25,
        //     static_coefficient: 1.,
        //     ..default()
        // },
        LockedAxes::ROTATION_LOCKED,
    ));

    info!("player");
}

fn control(
    velocity: Query<&mut LinearVelocity, With<Player>>,
    actions: Res<Actions>,
    time: Res<Time>,
) {
    for mut velocity in velocity {
        //let horizontal = Vec2::from_angle(45.0_f32.to_radians()).rotate(
        let horizontal = actions
            .clamped_axis_pair(&Action::Horizontal)
            .normalize_or_zero();

        let direction = Vec2::new(1., -1.);

        // speed = distance / time
        // TO DO: Why is 1 centimetre a second so fast? It actually causes us to move at
        // 0.5 metres a second, which can't be right.
        let speed = 0.01 / time.delta_secs();
        let velocity_xz = horizontal * speed * direction;

        velocity.x = velocity_xz.x;
        velocity.z = velocity_xz.y;

        // TO DO: Have some acceleration, instead of this instant velocity
        // control.
    }
}
