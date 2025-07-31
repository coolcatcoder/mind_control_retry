use std::ops::{Add, Div, Mul, Range, RangeBounds, RangeInclusive, Sub};

// TODO: Anything in here must be removed eventually.
use crate::{creatures::tester::Tester, mind_control::Controlled, physics::CollisionLayer};
use avian3d::prelude::{AngularVelocity, CollisionLayers, LinearVelocity, MassPropertiesBundle};
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, testing);
}

fn testing(mut commands: Commands) {
    // commands.spawn((
    //     Tester,
    //     Controlled,
    //     CollisionLayers::new(
    //         [CollisionLayer::Default, CollisionLayer::Floor],
    //         [
    //             CollisionLayer::Default,
    //             CollisionLayer::Floor,
    //             CollisionLayer::Cable,
    //         ],
    //     ),
    //     Transform::from_xyz(10., 5., 0.),
    // ));
    commands.spawn((
        Tester,
        Controlled,
        MassPropertiesBundle::from_shape(&Cuboid::new(1., 2., 1.), 20.),
        CollisionLayers::new(
            [CollisionLayer::Default, CollisionLayer::Floor],
            [
                CollisionLayer::Default,
                CollisionLayer::Floor,
                CollisionLayer::Cable,
            ],
        ),
        Transform::from_xyz(-5., 0.5, 0.),
    ));
}

pub fn change_range<T: Copy + PartialOrd + Sub<Output = T> + Div<Output = T> + Mul<Output = T> + Add<Output = T>>(from: (T, T), to: (T, T), value: T) -> Option<T> {
    if value < from.0 || value > from.1 {
        return None;
    }
    // From zero to (from.1 - from.0).
    let value_from_zero = value - from.0;
    // From zero to one.
    let value_from_zero_to_one = value_from_zero / (from.1 - from.0);
    // From zero to (to.1 - to.0).
    let value_from_zero = value_from_zero_to_one * (to.1 - to.0);
    // From to.0 to to.1.
    Some(value_from_zero + to.0)
}

pub fn move_towards_single_axis(
    desired_translation: f32,
    current_translation: f32,
    speed: f32,
    acceleration: f32,
    time_delta: f32,
    linear_velocity: &mut f32,
) {
    let desired_velocity = (desired_translation - current_translation).signum() * speed;
    let added_acceleration = (desired_velocity - *linear_velocity).signum() * acceleration * time_delta;
    //info!("added_acceleration: {added_acceleration}");
    *linear_velocity += added_acceleration;
}

pub fn rotate_towards_weird(
    desired_rotation_axis: Vec3,
    current_rotation_axis: Vec3,
    speed: f32,
    acceleration: f32,
    time_delta: f32,
    angular_velocity: &mut AngularVelocity,
) {
    let desired_velocity = (desired_rotation_axis - current_rotation_axis).normalize_or_zero() * speed;
    let added_acceleration = (desired_velocity - angular_velocity.0).normalize_or_zero() * acceleration * time_delta;
    //*angular_velocity 
}

// All functions below taken from unity.
// https://discussions.unity.com/t/how-to-rotate-towards-a-direction-with-physicsvelocity/787239

pub fn estimate_angles_between(from: Quat, to: Quat) -> Vec3 {
    let from_imag = from.xyz();
    let to_imag = to.xyz();

    let mut angle = from_imag.cross(to_imag);
    angle -= to.w * from_imag;
    angle += from.w * to_imag;
    angle += angle;
    if to_imag.dot(from_imag).is_sign_negative() {
        -angle
    } else {
        angle
    }
}

pub fn rotate_towards(
    desired_rotation: Quat,
    current_rotation: Quat,
    angular_velocity: &mut AngularVelocity,
    time_delta: f32,
) {
    angular_velocity.0 =
        estimate_angles_between(current_rotation, desired_rotation) * time_delta.recip();
}
