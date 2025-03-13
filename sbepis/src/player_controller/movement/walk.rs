use bevy::prelude::*;
use bevy_butler::*;
use bevy_rapier3d::prelude::*;

use crate::entity::Movement;
use crate::entity::movement::ExecuteMovementSet;
use crate::input::{button_just_pressed, button_just_released};
use crate::player_controller::movement::MovementControlSet;
use crate::player_controller::{PlayerAction, PlayerControllerPlugin};
use crate::prelude::PlayerBody;

use super::crouch::Crouching;
use super::dash::Dashing;
use super::di::DirectionalInput;
use super::grounded::Grounded;

#[derive(Resource)]
#[resource(plugin = PlayerControllerPlugin, init = PlayerWalkSettings {
	speed: 6.0,
	sneak_speed: 3.0,
	sprint_speed: 9.0,

	friction: 6.0,
	air_friction: 0.0,
	acceleration: 8.0,
	air_acceleration: 2.0,
})]
pub struct PlayerWalkSettings {
	pub speed: f32,
	pub sneak_speed: f32,
	pub sprint_speed: f32,

	pub friction: f32,
	pub air_friction: f32,
	pub acceleration: f32,
	pub air_acceleration: f32,
}

#[derive(Component, Default)]
pub struct Sprinting;

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
	before = ExecuteMovementSet,
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
			Has<Crouching>,
		),
		Without<Dashing>,
	>,
	walk_settings: Res<PlayerWalkSettings>,
	time: Res<Time>,
) {
	for (mut movement, velocity, transform, di, sprinting, grounded, crouching) in
		movement.iter_mut()
	{
		// Set up vectors
		let velocity = (transform.rotation.inverse() * velocity.linvel).xz();
		let wish_velocity = di.input
			* match (sprinting, crouching) {
				(true, false) => walk_settings.sprint_speed,
				(false, true) => walk_settings.sneak_speed,
				(_, _) => walk_settings.speed,
			};
		let wish_speed = wish_velocity.length();
		let wish_direction = wish_velocity.normalize_or_zero();
		let friction = if grounded {
			walk_settings.friction
		} else {
			walk_settings.air_friction
		};
		let acceleration = if grounded {
			walk_settings.acceleration
		} else {
			walk_settings.air_acceleration
		};

		// Apply friction
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
