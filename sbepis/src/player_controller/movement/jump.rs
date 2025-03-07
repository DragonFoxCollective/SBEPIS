use std::time::Duration;

use bevy::prelude::*;
use bevy_butler::*;
use bevy_rapier3d::prelude::*;

use crate::input::button_just_pressed;
use crate::player_controller::movement::MovementControlSet;
use crate::player_controller::{PlayerAction, PlayerBody, PlayerControllerPlugin};

use super::PlayerSpeed;
use super::crouch::Crouching;
use super::dash::Dashing;
use super::grounded::EffectiveGrounded;

#[derive(Component, Default)]
pub struct TryingToJump(Duration);

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_pressed(PlayerAction::Jump),
	in_set = MovementControlSet::UpdateJumping,
)]
fn add_trying_to_jump(players: Query<Entity, With<PlayerBody>>, mut commands: Commands) {
	for player in players.iter() {
		commands.entity(player).insert(TryingToJump::default());
	}
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSet::UpdateJumping,
	before = add_trying_to_jump,
)]
fn update_trying_to_jump(
	mut players: Query<(Entity, &mut TryingToJump)>,
	time: Res<Time>,
	speed_settings: Res<PlayerSpeed>,
	mut commands: Commands,
) {
	for (player, mut trying_to_jump) in players.iter_mut() {
		trying_to_jump.0 += time.delta();
		println!("Trying to jump: {:.2?}", trying_to_jump.0.as_secs_f32());
		if trying_to_jump.0 >= speed_settings.input_buffer_time {
			commands.entity(player).remove::<TryingToJump>();
		}
	}
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	after = MovementControlSet::DoHorizontalMovement,
	after = MovementControlSet::UpdateGrounded,
	after = MovementControlSet::UpdateJumping,
	in_set = MovementControlSet::DoVerticalMovement,
)]
fn jump(
	mut player_bodies: Query<
		(Entity, &mut Velocity, &Transform, Has<Crouching>),
		(With<EffectiveGrounded>, With<TryingToJump>),
	>,
	speed: Res<PlayerSpeed>,
	mut commands: Commands,
) {
	for (player, mut velocity, transform, crouching) in player_bodies.iter_mut() {
		println!("Jumping!");
		if transform.up().dot(velocity.linvel) < 0.0 {
			velocity.linvel = velocity.linvel.reject_from(transform.up().into());
		}
		velocity.linvel += transform.up()
			* match crouching {
				false => speed.jump_speed,
				true => speed.high_jump_speed,
			};
		commands
			.entity(player)
			.remove::<Dashing>()
			.remove::<TryingToJump>();
	}
}
