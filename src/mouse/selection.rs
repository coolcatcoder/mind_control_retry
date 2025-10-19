use std::fmt::Debug;

pub use crate::bevy_prelude::*;
use bevy_mod_outline::{AsyncSceneInheritOutline, OutlineMode, OutlineVolume};

const DEVELOP_OVERRIDE: bool = true;

/// Allows the entity to be selected with the mouse.
#[derive(Component)]
#[component(on_add = Self::on_add)]
pub struct Selected<const DEVELOP: bool = false>(pub bool)
where
    Selected<DEVELOP>: ClickOrPress;

#[derive(Component)]
pub struct SelectOthers(pub Vec<Entity>);

impl<const DEVELOP: bool> Selected<DEVELOP>
where
    Selected<DEVELOP>: ClickOrPress,
{
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        world
            .commands()
            .entity(context.entity)
            .observe(select::<DEVELOP>);
    }
}

#[derive(Component)]
#[require(OutlineVolume, OutlineMode, AsyncSceneInheritOutline, Selected::<DEVELOP>(false))]
pub struct OutlineWhileSelected<const DEVELOP: bool = false>
where
    Selected<DEVELOP>: ClickOrPress,
{
    pub colour: Color,
    pub width: f32,
}

/// Handles all observers that need to know whether it was selected or
/// unselected. This is due to a lack of observer ordering.
fn select<const DEVELOP: bool>(
    on: On<Pointer<<Selected<DEVELOP> as ClickOrPress>::On>>,
    mut selecteds: Query<(
        &mut Selected<DEVELOP>,
        Option<&SelectOthers>,
        Option<(&OutlineWhileSelected<DEVELOP>, &mut OutlineVolume)>,
    )>,
) where
    Selected<DEVELOP>: ClickOrPress,
{
    info!("got");
    let (mut selected, select_others, outline_while_selected) = selecteds
        .get_mut(on.entity)
        .else_error("Unreachable. No Selected component found.")?;
    selected.0 = !selected.0;

    if DEVELOP_OVERRIDE || crate::DEVELOP {
        info!("Entity {:?} was selected.", on.entity);
    }

    let (outline_while_selected, mut outline) = outline_while_selected.else_return()?;

    outline.visible = selected.0;
    if selected.0 {
        outline.colour = outline_while_selected.colour;
        outline.width = outline_while_selected.width;
    }

    // Hacky editor only junk.
    if let Some(select_others) = select_others {
        let selected = selected.0;
        for other in select_others.0.clone() {
            let (mut other_selected, _, outline_while_selected) = selecteds
                .get_mut(other)
                .else_error("Unreachable. No Selected component found.")?;

            other_selected.0 = selected;

            let (outline_while_selected, mut outline) = outline_while_selected.else_return()?;

            outline.visible = selected;
            if selected {
                outline.colour = outline_while_selected.colour;
                outline.width = outline_while_selected.width;
            }
        }
    }
}

/// A hack to improve the editor.
/// We need Press for the editor, but Click for everything else.
pub trait ClickOrPress {
    type On: Debug + Clone + Reflect;
}

impl ClickOrPress for Selected<false> {
    type On = Click;
}
impl ClickOrPress for Selected<true> {
    type On = Press;
}
