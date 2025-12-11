use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;

use crate::entity::EntityPlugin;
use crate::gravity::ComputedGravity;
use crate::player_controller::movement::MovementControlSystems;

#[auto_component(plugin = EntityPlugin, derive(Default), reflect, register)]
pub struct GravityOrientation;

#[auto_system(plugin = EntityPlugin, schedule = Update, config(
	after = MovementControlSystems::ExecuteMovement,
))]
fn orient(mut rigidbodies: Query<(&mut Transform, &ComputedGravity), With<GravityOrientation>>) {
    for (mut transform, gravity) in rigidbodies.iter_mut() {
        transform.rotation =
            Quat::from_rotation_arc(transform.up().into(), gravity.up) * transform.rotation;
    }
}
