use bevy::prelude::*;

pub mod battery;
pub mod cable;
pub mod light;
pub mod outlet;
pub mod power;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        battery::plugin,
        light::plugin,
        cable::plugin,
        outlet::plugin,
        power::plugin,
    ));
}
