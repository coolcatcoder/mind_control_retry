// TODO: Anything in here must be removed eventually.
use crate::{creatures::tester::Tester, mind_control::Controlled};
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, testing);
}

fn testing(mut commands: Commands) {
    commands.spawn((Tester, Controlled, Transform::from_xyz(10., 5., 0.)));
    commands.spawn((Tester, Controlled, Transform::from_xyz(0., 0.5, 0.)));
}
