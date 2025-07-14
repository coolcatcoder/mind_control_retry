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

pub fn drag(
    drag: Trigger<Pointer<Drag>>,
    mut velocity: Query<(&mut LinearVelocity, &Transform, &Children)>,
    camera: Query<(&Camera, &GlobalTransform)>,
    window: Query<&Window>,
    mut ray_cast: MeshRayCast,
    time: Res<Time>,
) -> Result {
    let (mut velocity, transform, children) = velocity
        .get_mut(drag.target())
        .else_error("No linear velocity when dragging entity.")?;

    let target = drag.target();

    let window = window.single().else_error("Not a single window.")?;
    let cursor_translation = window
        .cursor_position()
        .else_error("Dragging not on screen?")?;

    let (camera, camera_transform) = camera.single().else_error("Not a single camera.")?;
    let cursor_ray = camera
        .viewport_to_world(camera_transform, cursor_translation)
        .else_error("Viewport to world failed.")?;

    let (entity, hit) = ray_cast
        .cast_ray(
            cursor_ray,
            &MeshRayCastSettings {
                visibility: RayCastVisibility::VisibleInView,
                early_exit_test: &|entity| entity != target && !children.contains(&entity),
                filter: &|entity| entity != target && !children.contains(&entity),
            },
        )
        .first()
        .else_warn("I need to handle hitting nothing!")?;

    info!("entity: {}", entity);
    info!("entity: {target}");
    info!("hit translation: {}", hit.point);
    let cursor_translation = hit.point;

    let displacement = cursor_translation - transform.translation;
    let time = time.delta_secs();
    **velocity = displacement * time * 1000.;
    **velocity = velocity.min(Vec3::splat(10.));

    info!("displacement: {displacement}");
    info!("velocity: {}", **velocity);

    Ok(())
}
