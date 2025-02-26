use std::time::Duration;

use bevy::prelude::*;
use bevy_butler::*;
use bevy_rapier3d::prelude::*;

use crate::player_controller::movement::MovementControlSet;
use crate::player_controller::{PlayerBody, PlayerControllerPlugin};

use super::PlayerSpeed;

#[derive(Component, Default)]
pub struct Grounded;

#[derive(Component, Default)]
pub struct EffectiveGrounded(pub Duration);

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSet::UpdateGrounded,
)]
fn update_is_grounded(
	mut bodies: Query<(Entity, &GlobalTransform), With<PlayerBody>>,
	rapier_context: Query<&RapierContext>,
	mut commands: Commands,
) {
	let rapier_context = rapier_context.single();
	for (entity, transform) in bodies.iter_mut() {
		let mut grounded = false;
		rapier_context.intersections_with_shape(
			transform.translation() - transform.rotation() * Vec3::Y * 0.5,
			Quat::IDENTITY,
			&Collider::ball(0.25),
			QueryFilter::default(),
			|collided_entity| {
				if collided_entity == entity {
					true
				} else {
					grounded = true;
					false
				}
			},
		);
		if grounded {
			commands.entity(entity).insert(Grounded);
		} else {
			commands.entity(entity).remove::<Grounded>();
		}
	}
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	after = update_is_grounded,
	in_set = MovementControlSet::UpdateGrounded,
)]
fn add_effective_grounded(
	players: Query<Entity, (With<Grounded>, Without<EffectiveGrounded>)>,
	mut commands: Commands,
) {
	for player in players.iter() {
		commands.entity(player).insert(EffectiveGrounded::default());
	}
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	after = update_is_grounded,
	before = add_effective_grounded,
	in_set = MovementControlSet::UpdateGrounded,
)]
fn update_effective_grounded_time(
	mut players: Query<(Entity, &mut EffectiveGrounded), Without<Grounded>>,
	time: Res<Time>,
	mut commands: Commands,
	speed_settings: Res<PlayerSpeed>,
) {
	for (player, mut grounded) in players.iter_mut() {
		grounded.0 += time.delta();
		if grounded.0 >= speed_settings.coyote_time {
			commands.entity(player).remove::<EffectiveGrounded>();
		}
	}
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	after = update_is_grounded,
	before = add_effective_grounded,
	in_set = MovementControlSet::UpdateGrounded,
)]
fn reset_effective_grounded_time(mut players: Query<&mut EffectiveGrounded, With<Grounded>>) {
	for mut grounded in players.iter_mut() {
		grounded.0 = Duration::ZERO;
	}
}
