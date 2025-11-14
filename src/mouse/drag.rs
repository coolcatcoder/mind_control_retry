use crate::render::ComesFromRootEntity;
use avian3d::prelude::*;
pub use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, move_to_drag);
}

#[derive(Component, Default)]
#[component(on_add = Self::on_add)]
pub struct Dragged(pub Option<Vec3>);

impl Dragged {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        world
            .commands()
            .entity(context.entity)
            .observe(drag_start)
            .observe(drag_end)
            .observe(drag);
    }
}

pub fn drag_start(on: On<Pointer<DragStart>>, mut dragged: Query<(&mut Dragged, &Transform)>) {
    let (mut dragged, transform) = dragged
        .get_mut(on.entity)
        .else_error("No Dragged component.")?;
    dragged.0 = Some(transform.translation);
}

pub fn drag_end(on: On<Pointer<DragEnd>>, mut dragged: Query<&mut Dragged>) {
    dragged
        .get_mut(on.entity)
        .else_error("No Dragged component.")?
        .0 = None;
}

pub fn move_to_drag(
    mut dragged: Query<(&Dragged, &mut LinearVelocity, &Transform)>,
    time: Res<Time>,
) {
    dragged
        .iter_mut()
        .for_each(|(dragged, mut velocity, transform)| {
            let dragged = dragged.0.else_return()?;

            let desired_translation = dragged + Vec3::new(0., 1., 0.);

            let displacement = desired_translation - transform.translation;
            let time = time.delta_secs();
            velocity.0 = displacement * time * 1000.;
            velocity.0 = velocity.min(Vec3::splat(10.));
        });
}

pub fn drag(
    on: On<Pointer<Drag>>,
    mut dragged: Query<&mut Dragged>,
    camera: Query<(&Camera, &GlobalTransform)>,
    window: Query<&Window>,
    mut ray_cast: MeshRayCast,
    comes_from_root_entity: Query<&ComesFromRootEntity>,
) {
    let mut dragged = dragged
        .get_mut(on.entity)
        .else_error("No linear velocity when dragging entity.")?;
    let dragged = dragged
        .0
        .as_mut()
        .else_error("Not being dragged somehow.")?;

    let window = window.single().else_error("Not a single window.")?;
    let cursor_translation = window.cursor_position().else_return()?;

    let (camera, camera_transform) = camera.single().else_error("Not a single camera.")?;
    let cursor_ray = camera
        .viewport_to_world(camera_transform, cursor_translation)
        .else_error("Viewport to world failed.")?;

    let (_, hit) = ray_cast
        .cast_ray(
            cursor_ray,
            &MeshRayCastSettings {
                visibility: RayCastVisibility::VisibleInView,
                filter: &|entity| {
                    if entity == on.entity {
                        return false;
                    }
                    let Ok(comes_from_root_entity) = comes_from_root_entity.get(entity) else {
                        return true;
                    };

                    comes_from_root_entity.0 != on.entity
                },
                ..default()
            },
        )
        .first()
        .else_return()?;

    *dragged = hit.point;
}
