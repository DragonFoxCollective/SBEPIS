use std::time::Duration;

use bevy::prelude::*;
use bevy_butler::*;
use bevy_rapier3d::prelude::*;

use crate::input::button_just_pressed;
use crate::player_controller::movement::MovementControlSet;
use crate::player_controller::movement::stand::Standing;
use crate::player_controller::movement::walk::Walking;
use crate::player_controller::stamina::Stamina;
use crate::player_controller::{PlayerAction, PlayerBody, PlayerControllerPlugin};
use crate::util::MapRangeBetween;

use super::CoyoteTimeSettings;
use super::charge::{ChargeCrouching, ChargeStanding, ChargeWalking, ChargingSound};
use super::crouch::Crouching;
use super::dash::Dashing;
use super::grounded::EffectiveGrounded;
use super::slide::Sliding;

#[derive(Resource)]
#[resource(plugin = PlayerControllerPlugin, init = PlayerJumpSettings {
	jump_speed: 5.0,
	jump_stamina_cost: 0.0,

	high_jump_speed: 7.0,
	high_jump_stamina_cost: 0.0,

	charge_jump_min_speed: 5.0,
	charge_jump_max_speed: 10.0,
	charge_jump_min_stamina_cost: 0.0,
	charge_jump_max_stamina_cost: 0.33,

	unreal_air_jump_min_speed: 7.0,
	unreal_air_jump_max_speed: 15.0,
	unreal_air_jump_min_stamina_cost: 0.0,
	unreal_air_jump_max_stamina_cost: 0.66,
})]
pub struct PlayerJumpSettings {
	pub jump_speed: f32,
	pub jump_stamina_cost: f32,

	pub high_jump_speed: f32,
	pub high_jump_stamina_cost: f32,

	pub charge_jump_min_speed: f32,
	pub charge_jump_max_speed: f32,
	pub charge_jump_min_stamina_cost: f32,
	pub charge_jump_max_stamina_cost: f32,

	pub unreal_air_jump_min_speed: f32,
	pub unreal_air_jump_max_speed: f32,
	pub unreal_air_jump_min_stamina_cost: f32,
	pub unreal_air_jump_max_stamina_cost: f32,
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
		debug!("Trying to jump: {:.2?}", trying_to_jump.0.as_secs_f32());
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
			&mut Stamina,
			Has<Crouching>,
			Option<&ChargeStanding>,
			Option<&ChargeCrouching>,
			Has<ChargeWalking>,
			Option<&ChargingSound>,
		),
		(
			With<EffectiveGrounded>,
			With<TryingToJump>,
			Or<(
				With<Standing>,
				With<Walking>,
				With<Sliding>,
				With<ChargeStanding>,
				With<ChargeWalking>,
				With<ChargeCrouching>,
			)>,
		),
	>,
	settings: Res<PlayerJumpSettings>,
	assets: Res<JumpAssets>,
	mut commands: Commands,
) {
	for (
		player,
		mut velocity,
		transform,
		mut stamina,
		crouching,
		charge_standing,
		charge_crouching,
		charge_walking,
		charging_sound,
	) in player_bodies.iter_mut()
	{
		let (min_stamina_cost, result) = if crouching {
			(
				settings.high_jump_stamina_cost,
				if stamina.current > settings.high_jump_stamina_cost {
					Some((settings.high_jump_speed, settings.high_jump_stamina_cost))
				} else {
					None
				},
			)
		} else if let Some(charging) = charge_standing {
			(
				settings.charge_jump_min_stamina_cost,
				charging
					.power_and_stamina_cost_from_stamina(
						stamina.current,
						settings.charge_jump_min_stamina_cost,
						settings.charge_jump_max_stamina_cost,
					)
					.map(|(power, stamina_cost)| {
						(
							power.map_from_01(
								settings.charge_jump_min_speed..settings.charge_jump_max_speed,
							),
							stamina_cost,
						)
					}),
			)
		} else if let Some(charge_crouching) = charge_crouching {
			(
				settings.unreal_air_jump_min_stamina_cost,
				charge_crouching
					.power_and_stamina_cost_from_stamina(
						stamina.current,
						settings.unreal_air_jump_min_stamina_cost,
						settings.unreal_air_jump_max_stamina_cost,
					)
					.map(|(power, stamina_cost)| {
						(
							power.map_from_01(
								settings.unreal_air_jump_min_speed
									..settings.unreal_air_jump_max_speed,
							),
							stamina_cost,
						)
					}),
			)
		} else {
			(
				settings.jump_stamina_cost,
				if stamina.current > settings.jump_stamina_cost {
					Some((settings.jump_speed, settings.jump_stamina_cost))
				} else {
					None
				},
			)
		};

		if let Some((speed, stamina_cost)) = result {
			debug!("Jumping!");

			stamina.current -= stamina_cost;

			if transform.up().dot(velocity.linvel) < 0.0 {
				velocity.linvel = velocity.linvel.reject_from(transform.up().into());
			}
			velocity.linvel += transform.up() * speed;
		} else {
			debug!(
				"Not enough stamina to jump! Have {}, need {}",
				stamina.current, min_stamina_cost
			);
		}

		commands
			.entity(player)
			.remove::<Dashing>()
			.remove::<ChargeStanding>()
			.remove::<ChargeCrouching>()
			.remove::<ChargeWalking>()
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
				commands.entity(player).insert(Crouching);
			} else if charge_walking {
				commands.entity(player).insert(Walking);
			} else {
				commands.entity(player).insert(Standing);
			}
		}
	}
}
