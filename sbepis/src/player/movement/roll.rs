use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::entity::Movement;
use crate::gravity::AffectedByGravity;
use crate::player::PlayerControllerPlugin;
use crate::player::camera::third_person::PlayerCameraPositionEntities;
use crate::prelude::Player;

#[auto_resource(plugin = PlayerControllerPlugin, derive, init)]
pub struct RollingAssets {
    pub collider: Collider,
    pub collider_transform: Transform,
    pub camera_position: Vec3,
}

impl Default for RollingAssets {
    fn default() -> Self {
        let ball_radius = 0.5;
        RollingAssets {
            collider: Collider::ball(ball_radius),
            collider_transform: Transform::from_translation(Vec3::Y * ball_radius),
            camera_position: Vec3::Y * ball_radius,
        }
    }
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn to_rolling_assets(
    add: On<Add, Rolling>,
    players: Query<&Player>,
    cameras: Query<&PlayerCameraPositionEntities>,
    mut camera_transforms: Query<&mut Transform>,
    assets: Res<RollingAssets>,
    mut commands: Commands,
) -> Result {
    let body = players.get(add.entity)?;
    commands
        .entity(body.collider)
        .insert((assets.collider.clone(), assets.collider_transform));
    camera_transforms
        .get_mut(cameras.get(body.camera)?.first_person)?
        .translation = assets.camera_position;
    Ok(())
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn remove_movement(add: On<Add, Rolling>, mut commands: Commands) {
    commands
        .entity(add.entity)
        .remove::<Movement>()
        .insert(AffectedByGravity);
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn readd_movement(
    add: On<Remove, Rolling>,
    velocities: Query<&Velocity>,
    mut commands: Commands,
) -> Result {
    let velocity = velocities.get(add.entity)?;
    commands
        .entity(add.entity)
        .insert(Movement(velocity.linvel));
    Ok(())
}

#[auto_component(plugin = PlayerControllerPlugin, derive(Default), reflect, register)]
pub struct Rolling;
