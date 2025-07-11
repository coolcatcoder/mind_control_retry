use avian3d::prelude::*;
use bevy::{
    ecs::{component::HookContext, world::DeferredWorld},
    prelude::*,
};

use crate::{
    controls::{Action, Actions},
    mind_control::{Controlled, outline_on_over, remove_outline_on_out, take_control_on_click},
};

pub mod tester;

pub fn plugin(app: &mut App) {
    app.add_plugins(tester::plugin)
        .add_systems(Update, (land_handling, basic_horizontal_control).chain());
}

#[derive(Component, Default)]
#[component(on_add = Creature::on_add)]
pub struct Creature;

impl Creature {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        world
            .commands()
            .entity(context.entity)
            .observe(take_control_on_click)
            .observe(outline_on_over)
            .observe(remove_outline_on_out);
    }
}

pub enum LandHandlingState {
    Disabled,
    InControl,
    OutOfControl,
}

/// Handles the velocity of a creature on land.
#[derive(Component)]
pub struct LandHandling {
    pub state: LandHandlingState,
    pub gain_control: f32,
    pub lose_control: f32,
    pub slowing: f32,
}
fn land_handling(handling: Query<(&mut LinearVelocity, &mut LandHandling)>, time: Res<Time>) {
    let time_delta_seconds = time.delta_secs();

    for (mut velocity, mut handling) in handling {
        match handling.state {
            LandHandlingState::Disabled => (),
            LandHandlingState::InControl => {
                let velocity_length = velocity.xz().length();

                if velocity_length > 0.01 {
                    if velocity_length < handling.lose_control {
                        velocity.x *= handling.slowing * time_delta_seconds;
                        velocity.z *= handling.slowing * time_delta_seconds;
                    } else {
                        handling.state = LandHandlingState::OutOfControl;
                    }
                }
            }
            LandHandlingState::OutOfControl => {
                let velocity_length = velocity.length();

                if velocity_length <= handling.gain_control {
                    handling.state = LandHandlingState::InControl;
                }
            }
        }
    }
}

#[derive(Component)]
pub struct BasicHorizontalControl {
    pub speed: f32,
}
fn basic_horizontal_control(
    velocity: Query<(&mut LinearVelocity, &BasicHorizontalControl), With<Controlled>>,
    actions: Res<Actions>,
) {
    for (mut velocity, control) in velocity {
        //let horizontal = Vec2::from_angle(45.0_f32.to_radians()).rotate(
        let horizontal = actions
            .clamped_axis_pair(&Action::Horizontal)
            .normalize_or_zero()
            * Vec2::new(1., -1.)
            * control.speed;

        if horizontal.x != 0. {
            if horizontal.x.is_sign_positive() {
                velocity.x = velocity.x.max(horizontal.x);
            } else {
                velocity.x = velocity.x.min(horizontal.x);
            }
        }
        if horizontal.y != 0. {
            if horizontal.y.is_sign_positive() {
                velocity.z = velocity.z.max(horizontal.y);
            } else {
                velocity.z = velocity.z.min(horizontal.y);
            }
        }
    }
}
