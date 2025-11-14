use crate::plugin_modules;
use bevy::prelude::*;

pub mod battery;
pub mod cable;
pub mod light;
pub mod outlet;
pub mod power;
pub mod robot;

plugin_modules!(pub player);

pub fn plugin(app: &mut App) {
    app.add_plugins((
        battery::plugin,
        light::plugin,
        cable::plugin,
        outlet::plugin,
        power::plugin,
        robot::plugin,
        plugins_in_modules,
    ));
}
