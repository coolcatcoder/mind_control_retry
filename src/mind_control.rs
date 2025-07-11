use bevy::{
    ecs::{component::HookContext, world::DeferredWorld},
    prelude::*,
};
use bevy_mod_outline::OutlineVolume;

use crate::error_handling::ToFailure;

pub fn plugin(app: &mut App) {
    app.init_resource::<ControlledEntity>()
        .add_plugins(MeshPickingPlugin);
}

#[derive(Component)]
#[component(on_add = controlled_on_add)]
pub struct Controlled;

#[derive(Resource, Default)]
struct ControlledEntity(Option<Entity>);
fn controlled_on_add(mut world: DeferredWorld, context: HookContext) {
    let mut controlled_entity = world.resource_mut::<ControlledEntity>();
    let Some(old_entity) = controlled_entity.0.replace(context.entity) else {
        return;
    };

    let mut commands = world.commands();

    let mut old_entity = commands.entity(old_entity);
    old_entity.remove::<Controlled>();
    old_entity
        .entry::<OutlineVolume>()
        .and_modify(|mut outline| {
            outline.visible = false;
        });

    let mut entity = commands.entity(context.entity);
    entity.entry::<OutlineVolume>().and_modify(|mut outline| {
        outline.visible = true;
        outline.colour = Color::srgb(1., 1., 0.);
        outline.width = 2.;
    });
}

pub fn take_control_on_click(click: Trigger<Pointer<Click>>, mut commands: Commands) {
    commands.entity(click.target()).insert(Controlled);
}

pub fn outline_on_over(
    over: Trigger<Pointer<Over>>,
    mut outline: Query<&mut OutlineVolume, Without<Controlled>>,
) -> Result {
    let mut outline = outline.get_mut(over.target()).else_return()?;
    outline.visible = true;
    outline.colour = Color::srgb(0., 1., 1.);
    outline.width = 3.;
    Ok(())
}

pub fn remove_outline_on_out(
    out: Trigger<Pointer<Out>>,
    mut outline: Query<&mut OutlineVolume, Without<Controlled>>,
) -> Result {
    outline.get_mut(out.target()).else_return()?.visible = false;
    Ok(())
}
