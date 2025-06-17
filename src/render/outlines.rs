use bevy::prelude::*;
use bevy_mod_outline::{AutoGenerateOutlineNormalsPlugin, OutlinePlugin};

pub fn plugin(app: &mut App) {
    app.add_plugins((OutlinePlugin, AutoGenerateOutlineNormalsPlugin::default()));
}
