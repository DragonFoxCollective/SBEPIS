use std::time::Duration;

use bevy::prelude::*;
use bevy_butler::*;
use bevy_rapier3d::prelude::*;

use crate::input::button_just_pressed;
use crate::player_controller::movement::MovementControlSet;
use crate::player_controller::movement::sneak::Sneaking;
use crate::player_controller::movement::stand::Standing;
use crate::player_controller::movement::walk::Walking;
use crate::player_controller::{PlayerAction, PlayerBody, PlayerControllerPlugin};
use crate::util::MapRangeBetween;

use super::CoyoteTimeSettings;
use super::charge::{ChargeCrouching, Charging, ChargingSound};
use super::crouch::Crouching;
use super::dash::Dashing;
use super::di::DirectionalInput;
use super::grounded::EffectiveGrounded;

#[derive(Resource)]
#[resource(plugin = PlayerControllerPlugin, init = PlayerJumpSettings {
	jump_speed: 5.0,
	high_jump_speed: 7.0,
	charge_jump_speed: 10.0,
	unreal_air_jump_speed: 15.0,
})]
pub struct PlayerJumpSettings {
	pub jump_speed: f32,
	pub high_jump_speed: f32,
	pub charge_jump_speed: f32,
	pub unreal_air_jump_speed: f32,
}

#[derive(Resource)]
pub struct JumpAssets {
	pub charge_jump_sound: Handle<AudioSource>,
}

#[system(plugin = PlayerControllerPlugin, schedule = Startup)]
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
	commands.insert_resource(JumpAssets {
		charge_jump_sound: asset_server.load("worms bazooka shoot.mp3"),
	});
}

#[derive(Component, Default)]
pub struct TryingToJump(Duration);

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_pressed(PlayerAction::Jump),
	in_set = MovementControlSet::UpdateState,
)]
fn add_trying_to_jump(players: Query<Entity, With<PlayerBody>>, mut commands: Commands) {
	for player in players.iter() {
		commands.entity(player).insert(TryingToJump::default());
	}
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSet::UpdateState,
	before = add_trying_to_jump,
)]
fn update_trying_to_jump(
	mut players: Query<(Entity, &mut TryingToJump)>,
	time: Res<Time>,
	cotote_time_settings: Res<CoyoteTimeSettings>,
	mut commands: Commands,
) {
	for (player, mut trying_to_jump) in players.iter_mut() {
		trying_to_jump.0 += time.delta();
		println!("Trying to jump: {:.2?}", trying_to_jump.0.as_secs_f32());
		if trying_to_jump.0 >= cotote_time_settings.input_buffer_time {
			commands.entity(player).remove::<TryingToJump>();
		}
	}
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	after = MovementControlSet::DoHorizontalMovement,
	in_set = MovementControlSet::DoVerticalMovement,
)]
fn jump(
	mut player_bodies: Query<
		(
			Entity,
			&mut Velocity,
			&Transform,
			&DirectionalInput,
			Has<Crouching>,
			Option<&Charging>,
			Option<&ChargeCrouching>,
			Option<&ChargingSound>,
		),
		(With<EffectiveGrounded>, With<TryingToJump>),
	>,
	speed: Res<PlayerJumpSettings>,
	assets: Res<JumpAssets>,
	mut commands: Commands,
) {
	for (
		player,
		mut velocity,
		transform,
		di,
		crouching,
		charging,
		charge_crouching,
		charging_sound,
	) in player_bodies.iter_mut()
	{
		println!("Jumping!");
		if transform.up().dot(velocity.linvel) < 0.0 {
			velocity.linvel = velocity.linvel.reject_from(transform.up().into());
		}
		velocity.linvel += transform.up()
			* if crouching {
				speed.high_jump_speed
			} else if let Some(charging) = charging {
				charging
					.power()
					.map_from_01(speed.jump_speed..speed.charge_jump_speed)
			} else if let Some(charge_crouching) = charge_crouching {
				charge_crouching
					.power()
					.map_from_01(speed.jump_speed..speed.unreal_air_jump_speed)
			} else {
				speed.jump_speed
			};
		commands
			.entity(player)
			.remove::<Dashing>()
			.remove::<Charging>()
			.remove::<ChargeCrouching>()
			.remove::<ChargingSound>()
			.remove::<TryingToJump>();

		if let Some(charging_sound) = charging_sound {
			commands.spawn((
				AudioPlayer(assets.charge_jump_sound.clone()),
				PlaybackSettings::DESPAWN,
			));

			if let Some(sound) = commands.get_entity(charging_sound.0) {
				sound.despawn_recursive();
			}

			if charge_crouching.is_some() {
				if di.world_space.length() > 0.0 {
					commands.entity(player).insert(Sneaking);
				} else {
					commands.entity(player).insert(Crouching);
				}
			} else if di.world_space.length() > 0.0 {
				commands.entity(player).insert(Walking);
			} else {
				commands.entity(player).insert(Standing);
			}
		}
	}
}
