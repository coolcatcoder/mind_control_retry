use std::{
    fs::OpenOptions,
    io::{Read, Seek, Write},
};

use avian3d::prelude::{Physics, PhysicsTime};
use bevy::{
    feathers::{
        FeathersPlugins,
        controls::{ButtonProps, button},
        dark_theme::create_dark_theme,
        theme::{ThemeBackgroundColor, ThemedText, UiTheme},
        tokens,
    },
    input_focus::tab_navigation::TabGroup,
    scene::SceneInstance,
    ui_widgets::{Activate, observe},
};
use bevy_mod_outline::OutlineVolume;

pub use crate::bevy_prelude::*;
use crate::{
    areas::{Area, AreaLoadedEntity},
    mouse::{
        drag::Dragged,
        selection::{OutlineWhileSelected, SelectOthers, Selected},
    },
    render::ComesFromRootEntity,
};

const DEVELOP_OVERRIDE: bool = false;

pub fn editor(file_path: &'static str) -> impl Fn(&mut App) {
    move |app: &mut App| {
        app.add_plugins(FeathersPlugins)
            .insert_resource(UiTheme(create_dark_theme()))
            .add_observer(make_develop_selectable)
            .add_systems(Startup, create_load(file_path));

        if crate::DEVELOP || DEVELOP_OVERRIDE {
            app.add_systems(Update, log_system);
        }
    }
}

fn log_system(query: Query<(Entity, &Transform, Ref<Transform>, &Name), Changed<Transform>>) {
    for (entity, transform, t, name) in &query {
        info!(
            ?entity,
            ?name,
            ?transform,
            changed_by = ?t.changed_by(),
            "changed"
        );
    }
}

fn make_develop_selectable(on: On<AreaLoadedEntity>, mut commands: Commands) {
    commands
        .entity(on.loaded)
        .insert((
            Selected::<true>(false),
            OutlineWhileSelected::<true> {
                colour: Color::srgb(1., 0., 0.),
                width: 3.,
            },
        ))
        .observe(drag);

    info!("Did it.");
}

fn create_load(file_path: &'static str) -> impl Fn(Commands) {
    move |mut commands: Commands| {
        commands.spawn((
            Node {
                width: percent(20),
                height: percent(100),
                align_items: AlignItems::Start,
                justify_content: JustifyContent::Start,
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                ..default()
            },
            TabGroup::default(),
            ThemeBackgroundColor(tokens::WINDOW_BG),
            children![(
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    justify_content: JustifyContent::Start,
                    padding: UiRect::all(px(8)),
                    row_gap: px(8),
                    width: percent(100),
                    ..default()
                },
                children![
                    (
                        button(
                            ButtonProps::default(),
                            (),
                            Spawn((Text::new("Highlight Selected"), ThemedText))
                        ),
                        observe(
                            |_: On<Activate>,
                             selected: Query<(
                                &mut OutlineVolume,
                                &Selected<true>,
                                &OutlineWhileSelected<true>
                            )>| {
                                for (mut outline, selected, outline_while_selected) in selected {
                                    if selected.0 {
                                        outline.visible = true;
                                        outline.colour = outline_while_selected.colour;
                                        outline.width = outline_while_selected.width;
                                    }
                                }
                            }
                        )
                    ),
                    (
                        button(
                            ButtonProps::default(),
                            (),
                            Spawn((Text::new("Patch Selected"), ThemedText))
                        ),
                        observe(create_apply_patches(file_path))
                    ),
                    (
                        button(
                            ButtonProps::default(),
                            (),
                            Spawn((Text::new("toggle physics"), ThemedText))
                        ),
                        observe(|_: On<Activate>, mut time: ResMut<Time<Physics>>| {
                            if time.is_paused() {
                                time.unpause();
                            } else {
                                time.pause();
                            }
                        })
                    ),
                ]
            ),],
        ));
    }
}

#[rustfmt::skip]
pub fn create_apply_patches(
    module_path: &'static str,
) -> impl Fn(On<Activate>, Query<(&Selected<true>, &Name, &Transform)>) {
    move |_: On<Activate>, selected: Query<(&Selected<true>, &Name, &Transform)>| {
        let mut patches = vec![];
        for (selected, name, transform) in selected {
            if selected.0 {
                //info!("{}", name);
                let patch = (
                    name.to_string(),
                    format!(
                        "            {:?} => {{
                *transform = Transform {{
                    translation: Vec3::new({:?}, {:?}, {:?}),
                    rotation: Quat::from_array([{:?}, {:?}, {:?}, {:?}]),
                    scale: Vec3::new({:?}, {:?}, {:?}),
                }}
            }}",
                        name.as_str(),
                        transform.translation.x,
                        transform.translation.y,
                        transform.translation.z,
                        transform.rotation.x,
                        transform.rotation.y,
                        transform.rotation.z,
                        transform.rotation.w,
                        transform.scale.x,
                        transform.scale.y,
                        transform.scale.z,
                    ),
                );
                patches.push(patch);
            }
        }

        let mut file = OpenOptions::new()
            .write(true)
            .read(true)
            .append(false)
            .open(module_path)
            .else_error(format!("Could not open file. Path: {module_path}"))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .else_error("Could not read file.")?;

        while !patches.is_empty() {
            let patch_function_start = contents.find("fn patch");

            match patch_function_start {
                None => {
                    contents.push_str(
                        "
#[rustfmt::skip]
pub fn patch(name: &str, mut transform: Mut<Transform>) {
        #[allow(clippy::match_same_arms)]
        #[allow(clippy::unreadable_literal)]
        #[allow(clippy::single_match)]
        match name {",
                    );

                    let patches = std::mem::take(&mut patches);
                    for (_, patch) in patches {
                        use std::fmt::Write;
                        write!(&mut contents, "\n{patch}")
                            .else_error("Failed to write to contents")?;
                    }

                    contents.push_str(
                        "
            _ => (),
        }
}
",
                    );
                }
                Some(start_of_function) => {
                    let mut brackets_open = 1;
                    let mut index = start_of_function + 52;
                    while brackets_open != 0 {
                        index += 1;
                        let character = *contents
                            .as_bytes()
                            .get(index)
                            .else_error("Closing bracket not found.")?
                            as char;
                        //print!("{character}");
                        match character {
                            '{' => {
                                brackets_open += 1;
                            }
                            '}' => {
                                brackets_open -= 1;
                            }
                            _ => (),
                        }
                    }

                    let patch_function = contents
                        .get(start_of_function..index)
                        .else_error("Patch function range is broken.")?
                        .to_owned();

                    //info!("{patch_function}");

                    let (name, patch) = patches.pop().else_error("The while loop failed.")?;

                    match patch_function.find(&format!("\"{name}\"")) {
                        None => {
                            //info!("Some None");
                            let last_match = patch_function
                                .find("_ =>")
                                .else_error("Could not find last match.")?;
                            contents.insert_str(
                                last_match + start_of_function - 12,
                                &format!("{patch}\n"),
                            );
                        }
                        Some(start_of_arm) => {
                            //info!("Some Some");
                            let mut brackets_open = 1;
                            let mut index = start_of_function + start_of_arm + name.len() + 7;
                            //info!("Begin!");
                            while brackets_open != 0 {
                                index += 1;
                                let character = *contents
                                    .as_bytes()
                                    .get(index)
                                    .else_error("Closing bracket not found.")?
                                    as char;
                                //print!("{character}");
                                match character {
                                    '{' => {
                                        brackets_open += 1;
                                    }
                                    '}' => {
                                        brackets_open -= 1;
                                    }
                                    _ => (),
                                }
                            }

                            // let arm = contents
                            //     .get((start_of_function + start_of_arm)..=index)
                            //     .else_error("Patch function range is broken.")?
                            //     .to_owned();

                            //info!("{arm}");

                            contents.replace_range(
                                (start_of_function + start_of_arm - 12)..=index,
                                &patch,
                            );
                        }
                    }
                }
            }
        }

        //info!("{contents}");
        file.rewind().else_error("Could not rewind.")?;
        file.write_all(contents.as_bytes())
            .else_error("Could not save patches.")?;
        file.set_len(contents.len() as u64)
            .else_error("Could not set length.")?;
    }
}

pub fn drag(
    drag: On<Pointer<Drag>>,
    mut transforms: Query<&mut Transform, Without<Dragged>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    window: Query<&Window>,
    mut ray_cast: MeshRayCast,
    comes_from_root_entity: Query<&ComesFromRootEntity>,
) {
    // Return, rather than error. If the entity can already be dragged, then we
    // don't want to double dip.
    let mut transform = transforms.get_mut(drag.entity).else_return()?;

    let target = drag.event().event_target();

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
                    if entity == target {
                        return false;
                    }
                    let Ok(comes_from_root_entity) = comes_from_root_entity.get(entity) else {
                        return true;
                    };

                    comes_from_root_entity.0 != target
                },
                ..default()
            },
        )
        .first()
        .else_return()?;

    let cursor_translation = hit.point;
    let desired_translation = cursor_translation + Vec3::new(0., 0.5, 0.);

    transform.translation = desired_translation;
}
