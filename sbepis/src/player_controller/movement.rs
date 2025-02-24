use bevy::prelude::*;
use bevy_butler::*;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::entity::Movement;
use crate::input::button_just_pressed;
use crate::player_controller::{PlayerAction, PlayerBody, PlayerControllerPlugin};

#[derive(Resource)]
#[resource(plugin = PlayerControllerPlugin, init = PlayerSpeed {
	speed: 7.0,
	sprint_modifier: 2.0,
	jump_speed: 5.0,
	friction: 6.0,
	air_friction: 0.0,
	acceleration: 8.0,
	air_acceleration: 6.0,
})]
pub struct PlayerSpeed {
	pub speed: f32,
	pub sprint_modifier: f32,
	pub jump_speed: f32,
	pub friction: f32,
	pub air_friction: f32,
	pub acceleration: f32,
	pub air_acceleration: f32,
}

#[derive(Component)]
pub struct Sprinting;

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
)]
fn update_sprinting(
	input: Query<&ActionState<PlayerAction>>,
	players: Query<Entity, With<PlayerBody>>,
	mut commands: Commands,
) {
	let input = input.single();
	let player = players.single();

	if input.just_pressed(&PlayerAction::Sprint) {
		commands.entity(player).insert(Sprinting);
	}
	if input.just_released(&PlayerAction::Sprint) {
		commands.entity(player).remove::<Sprinting>();
	}
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	after = update_sprinting,
)]
fn axes_to_ground_velocity(
	input: Query<&ActionState<PlayerAction>>,
	mut movement: Query<(&PlayerBody, &mut Movement, &Transform, Has<Sprinting>)>,
	speed: Res<PlayerSpeed>,
	time: Res<Time>,
) {
	let input = input.single();
	let (body, mut movement, transform, sprinting) = movement.single_mut();
	let input_dir = input.clamped_axis_pair(&PlayerAction::Move) * Vec2::new(1.0, -1.0);

	// Set up vectors
	let velocity = (transform.rotation.inverse() * movement.0).xz();
	let wish_velocity = input_dir
		* speed.speed
		* if sprinting {
			speed.sprint_modifier
		} else {
			1.0
		};
	let wish_speed = wish_velocity.length();
	let wish_direction = wish_velocity.normalize_or_zero();
	let friction = if body.is_grounded {
		speed.friction
	} else {
		speed.air_friction
	};
	let acceleration = if body.is_grounded {
		speed.acceleration
	} else {
		speed.air_acceleration
	};

	// Apply friction
	let friction = -time.delta_secs() * friction * velocity;
	let velocity = velocity + friction;

	// Do funny quake movement
	let funny_quake_speed = velocity.dot(wish_direction);
	let add_speed =
		(wish_speed - funny_quake_speed).clamp(0.0, acceleration * wish_speed * time.delta_secs()); // TODO: In absolute units, ignores relativity
	let new_velocity = velocity + wish_direction * add_speed;

	movement.0 = transform.rotation * Vec3::new(new_velocity.x, 0.0, new_velocity.y);
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_pressed(PlayerAction::Jump),
)]
fn jump(
	mut player_bodies: Query<(&PlayerBody, &mut Velocity, &Transform)>,
	speed: Res<PlayerSpeed>,
) {
	for (body, mut velocity, transform) in player_bodies.iter_mut() {
		if body.is_grounded {
			velocity.linvel += transform.up() * speed.jump_speed;
		}
	}
}
