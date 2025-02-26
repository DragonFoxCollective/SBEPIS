use bevy::prelude::*;
use bevy_butler::*;
use bevy_rapier3d::prelude::*;

use crate::entity::Movement;
use crate::input::{button_just_pressed, button_just_released};
use crate::player_controller::movement::MovementControlSet;
use crate::player_controller::{PlayerAction, PlayerControllerPlugin};
use crate::prelude::PlayerBody;

use super::dash::{Dashing, Sprinting};
use super::di::DirectionalInput;
use super::grounded::Grounded;
use super::PlayerSpeed;

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSet::UpdateSprinting,
	run_if = button_just_pressed(PlayerAction::Sprint),
)]
fn add_sprinting(players: Query<Entity, With<PlayerBody>>, mut commands: Commands) {
	for player in players.iter() {
		commands.entity(player).insert(Sprinting);
	}
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSet::UpdateSprinting,
	run_if = button_just_released(PlayerAction::Sprint),
)]
fn remove_sprinting(players: Query<Entity, With<PlayerBody>>, mut commands: Commands) {
	for player in players.iter() {
		commands.entity(player).remove::<Sprinting>();
	}
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	after = MovementControlSet::UpdateDashing,
	after = MovementControlSet::UpdateSprinting,
	in_set = MovementControlSet::DoHorizontalMovement,
)]
fn update_walk_velocity(
	mut movement: Query<
		(
			&mut Movement,
			&Velocity,
			&Transform,
			&DirectionalInput,
			Has<Sprinting>,
			Has<Grounded>,
		),
		Without<Dashing>,
	>,
	speed_settings: Res<PlayerSpeed>,
	time: Res<Time>,
) {
	for (mut movement, velocity, transform, di, sprinting, grounded) in movement.iter_mut() {
		// Set up vectors
		let velocity = (transform.rotation.inverse() * velocity.linvel).xz();
		let wish_velocity = di.input
			* speed_settings.speed
			* if sprinting {
				speed_settings.sprint_modifier
			} else {
				1.0
			};
		let wish_speed = wish_velocity.length();
		let wish_direction = wish_velocity.normalize_or_zero();
		let friction = if grounded {
			speed_settings.friction
		} else {
			speed_settings.air_friction
		};
		let acceleration = if grounded {
			speed_settings.acceleration
		} else {
			speed_settings.air_acceleration
		};

		// Apply friction // FIXME: using the the physics velocity applies friction, but it's not the same as the friction in the movement system
		let friction = -time.delta_secs() * friction * velocity;
		let velocity = velocity + friction;

		// Do funny quake movement
		let funny_quake_speed = velocity.dot(wish_direction);
		let add_speed = (wish_speed - funny_quake_speed)
			.clamp(0.0, acceleration * wish_speed * time.delta_secs()); // TODO: In absolute units, ignores relativity
		let new_velocity = velocity + wish_direction * add_speed;

		movement.0 = transform.rotation * Vec3::new(new_velocity.x, 0.0, new_velocity.y);
	}
}
