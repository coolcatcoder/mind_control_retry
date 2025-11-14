use bevy::ecs::{lifecycle::HookContext, world::DeferredWorld};
use bevy::prelude::*;
use bevy_mod_outline::{AsyncSceneInheritOutline, OutlineMode, OutlineVolume};

plugin_modules!(pub selection, pub drag);

pub fn plugin(app: &mut App) {
    plugins_in_modules(app);
}

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
    over: On<Pointer<Over>>,
    mut outline: Query<&mut OutlineVolume, With<Interactable>>,
) {
    let mut outline = outline.get_mut(over.event().event_target()).else_return()?;
    outline.visible = true;
    outline.colour = Color::srgb(0., 1., 1.);
    outline.width = 3.;
}

pub fn remove_outline_on_out(
    out: On<Pointer<Out>>,
    mut outline: Query<&mut OutlineVolume, With<Interactable>>,
) {
    outline
        .get_mut(out.event().event_target())
        .else_return()?
        .visible = false;
}
