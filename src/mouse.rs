use crate::error_handling::ToFailure;
use avian3d::prelude::LinearVelocity;
use bevy::{
    ecs::{component::HookContext, world::DeferredWorld},
    prelude::*,
};
use bevy_mod_outline::{AsyncSceneInheritOutline, OutlineMode, OutlineVolume};

pub fn plugin(_: &mut App) {}

#[derive(Component, Default)]
#[component(on_add = Self::on_add)]
pub struct Interactable;

impl Interactable {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        world
            .commands()
            .entity(context.entity)
            .insert((
                OutlineVolume {
                    visible: false,
                    colour: Color::srgb(0., 1., 1.),
                    width: 3.,
                },
                OutlineMode::ExtrudeFlat,
                AsyncSceneInheritOutline::default(),
            ))
            .observe(outline_on_over)
            .observe(remove_outline_on_out);
    }
}

pub fn outline_on_over(
    over: Trigger<Pointer<Over>>,
    mut outline: Query<&mut OutlineVolume, With<Interactable>>,
) -> Result {
    let mut outline = outline.get_mut(over.target()).else_return()?;
    outline.visible = true;
    outline.colour = Color::srgb(0., 1., 1.);
    outline.width = 3.;
    Ok(())
}

pub fn remove_outline_on_out(
    out: Trigger<Pointer<Out>>,
    mut outline: Query<&mut OutlineVolume, With<Interactable>>,
) -> Result {
    outline.get_mut(out.target()).else_return()?.visible = false;
    Ok(())
}

pub fn drag(drag: Trigger<Pointer<Drag>>, mut velocity: Query<&mut LinearVelocity>) -> Result {
    let mut velocity = velocity
        .get_mut(drag.target())
        .else_error("No linear velocity when dragging entity.")?;
    let drag_delta = drag.delta * 0.1;
    velocity.x += drag_delta.x;
    velocity.z += drag_delta.y;
    Ok(())
}
