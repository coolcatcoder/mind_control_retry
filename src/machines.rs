use bevy::prelude::*;

pub mod battery;

pub fn plugin(app: &mut App) {
    app.add_plugins(battery::plugin);
}
