use bevy::prelude::*;
use bevy_butler::*;

use crate::entity::EntityPlugin;
use crate::entity::movement::ExecuteMovementSet;
use crate::gravity::ComputedGravity;

#[derive(Component, Default)]
pub struct GravityOrientation;

#[add_system(
	plugin = EntityPlugin, schedule = Update,
	after = ExecuteMovementSet,
)]
fn orient(mut rigidbodies: Query<(&mut Transform, &ComputedGravity), With<GravityOrientation>>) {
    for (mut transform, gravity) in rigidbodies.iter_mut() {
        transform.rotation =
            Quat::from_rotation_arc(transform.up().into(), gravity.up) * transform.rotation;
    }
}
