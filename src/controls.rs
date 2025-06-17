use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_plugins(InputManagerPlugin::<Action>::default())
        .init_resource::<ActionState<Action>>()
        .insert_resource(input_map());
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum Action {
    #[actionlike(DualAxis)]
    Horizontal,
}

fn input_map() -> InputMap<Action> {
    InputMap::default().with_dual_axis(Action::Horizontal, VirtualDPad::wasd())
}

pub type Actions = ActionState<Action>;
