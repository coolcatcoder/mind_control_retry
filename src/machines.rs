use bevy::prelude::*;

pub mod battery;
pub mod light;

pub fn plugin(app: &mut App) {
    app.add_plugins((battery::plugin, light::plugin));
}
