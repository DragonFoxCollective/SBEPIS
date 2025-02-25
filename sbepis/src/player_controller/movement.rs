use std::time::Duration;

use bevy::prelude::*;
use bevy_butler::*;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::camera::PlayerCamera;
use crate::entity::Movement;
use crate::input::button_just_pressed;
use crate::player_controller::{PlayerAction, PlayerBody, PlayerControllerPlugin};

#[derive(Resource)]
#[resource(plugin = PlayerControllerPlugin, init = PlayerSpeed {
	speed: 6.0,
	sprint_modifier: 1.5,
	jump_speed: 5.0,
	friction: 6.0,
	air_friction: 0.0,
	acceleration: 8.0,
	air_acceleration: 2.0,
	dash_speed_addon: 6.0,
})]
pub struct PlayerSpeed {
	pub speed: f32,
	pub sprint_modifier: f32,
	pub jump_speed: f32,
	pub friction: f32,
	pub air_friction: f32,
	pub acceleration: f32,
	pub air_acceleration: f32,

	pub dash_speed_addon: f32,
}

#[derive(Component, Default)]
pub struct DirectionalInput {
	pub input: Vec2,
	pub local_space: Vec3,
	pub world_space: Vec3,
	pub forward: Vec3,
}

#[derive(Component, Default)]
pub struct Sprinting(pub Duration);

#[derive(Component)]
pub struct Dashing {
	pub duration: Duration,
	pub velocity: Vec3,
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
)]
fn update_di(
	input: Query<&ActionState<PlayerAction>>,
	mut players: Query<&mut DirectionalInput, With<PlayerBody>>,
	player_cameras: Query<&GlobalTransform, With<PlayerCamera>>,
) {
	let input = input.single();
	let mut di = players.single_mut();
	let transform = player_cameras.single();
	di.input = input.axis_pair(&PlayerAction::Move).clamp_length_max(1.0) * Vec2::new(1.0, -1.0);
	di.local_space = Vec3::new(di.input.x, 0.0, di.input.y);
	di.world_space = transform.rotation() * di.local_space;
	di.forward = transform.rotation() * -Vec3::Z;
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	after = update_di,
)]
fn update_sprinting_state(
	input: Query<&ActionState<PlayerAction>>,
	mut players: Query<(
		Entity,
		&PlayerBody,
		&Velocity,
		&DirectionalInput,
		Option<&mut Sprinting>,
		Option<&mut Dashing>,
	)>,
	speed_settings: Res<PlayerSpeed>,
	time: Res<Time>,
	mut commands: Commands,
) {
	let input = input.single();
	let (player, body, velocity, di, sprinting, dashing) = players.single_mut();

	if input.just_pressed(&PlayerAction::Sprint) {
		commands.entity(player).insert(Sprinting::default());

		if dashing.is_none() && body.is_grounded {
			commands.entity(player).insert(Dashing {
				duration: Duration::ZERO,
				velocity: di.world_space.normalize_or(di.forward)
					* (velocity.linvel.length() + speed_settings.dash_speed_addon),
			});
		}
	}
	if input.just_released(&PlayerAction::Sprint) {
		commands.entity(player).remove::<Sprinting>();
	}

	if let Some(mut sprinting) = sprinting {
		sprinting.0 += time.delta();
	}
	if let Some(mut dashing) = dashing {
		dashing.duration += time.delta();
	}
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	after = update_sprinting_state,
)]
fn update_walk_velocity(
	mut movement: Query<
		(
			&PlayerBody,
			&mut Movement,
			&Velocity,
			&Transform,
			&DirectionalInput,
			Has<Sprinting>,
		),
		Without<Dashing>,
	>,
	speed_settings: Res<PlayerSpeed>,
	time: Res<Time>,
) {
	for (body, mut movement, velocity, transform, di, sprinting) in movement.iter_mut() {
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
		let friction = if body.is_grounded {
			speed_settings.friction
		} else {
			speed_settings.air_friction
		};
		let acceleration = if body.is_grounded {
			speed_settings.acceleration
		} else {
			speed_settings.air_acceleration
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

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	after = update_sprinting_state,
)]
fn update_dash_velocity(
	mut movement: Query<(Entity, &mut Movement, &mut Velocity, &Dashing)>,
	speed_settings: Res<PlayerSpeed>,
	mut commands: Commands,
) {
	for (player, mut movement, mut velocity, dashing) in movement.iter_mut() {
		if dashing.duration < Duration::from_secs_f32(0.2) {
			velocity.linvel = dashing.velocity;
			movement.0 = dashing.velocity;
		} else {
			velocity.linvel = dashing.velocity.normalize_or_zero()
				* (dashing.velocity.length() - speed_settings.dash_speed_addon);
			movement.0 = velocity.linvel;
			commands.entity(player).remove::<Dashing>();
		}
	}
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_pressed(PlayerAction::Jump),
	after = update_dash_velocity,
)]
fn jump(
	mut player_bodies: Query<(Entity, &PlayerBody, &mut Velocity, &Transform)>,
	speed: Res<PlayerSpeed>,
	mut commands: Commands,
) {
	for (player, body, mut velocity, transform) in player_bodies.iter_mut() {
		if body.is_grounded {
			velocity.linvel += transform.up() * speed.jump_speed;
			commands.entity(player).remove::<Dashing>();
		}
	}
}
