use bevy::prelude::*;
use bevy_butler::*;

use crate::input::{button_just_pressed, button_just_released};
use crate::player_controller::movement::MovementControlSet;
use crate::player_controller::movement::dash::add_trying_to_dash;
use crate::player_controller::{PlayerAction, PlayerControllerPlugin};
use crate::prelude::PlayerBody;

use super::crouch::{CrouchingAssets, StandingAssets, to_crouching_assets, to_standing_assets};
use super::dash::TryingToDash;
use super::grounded::EffectiveGrounded;
use super::stand::Standing;

#[derive(Resource)]
pub struct ChargeAssets {
	pub sound: Handle<AudioSource>,
}

#[system(plugin = PlayerControllerPlugin, schedule = Startup)]
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
	commands.insert_resource(ChargeAssets {
		sound: asset_server.load("worms bazooka charge.mp3"),
	});
}

#[derive(Component)]
pub struct Charging;

#[derive(Component)]
pub struct ChargeCrouching;

#[derive(Component)]
pub struct ChargingSound(pub Entity);

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	after = MovementControlSet::UpdateDi,
	after = MovementControlSet::UpdateGrounded,
	after = add_trying_to_dash,
	in_set = MovementControlSet::UpdateState,
)]
fn standing_to_charging(
	players: Query<
		Entity,
		(
			With<EffectiveGrounded>,
			With<TryingToDash>,
			With<Standing>,
			Without<Charging>,
		),
	>,
	mut commands: Commands,
	assets: Res<ChargeAssets>,
) {
	for player in players.iter() {
		println!("Charging!");

		let sound = commands
			.spawn((AudioPlayer(assets.sound.clone()), PlaybackSettings::DESPAWN))
			.id();

		commands
			.entity(player)
			.insert(Charging)
			.insert(ChargingSound(sound))
			.remove::<TryingToDash>()
			.remove::<Standing>();
	}
}

#[system(
    plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_released(PlayerAction::Sprint),
	in_set = MovementControlSet::UpdateState,
)]
fn charging_to_standing(
	players: Query<(Entity, &ChargingSound), With<Charging>>,
	mut commands: Commands,
) {
	for (player, charging_sound) in players.iter() {
		commands
			.entity(player)
			.remove::<Charging>()
			.insert(Standing);

		if let Some(sound) = commands.get_entity(charging_sound.0) {
			sound.despawn_recursive();
		}
	}
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_pressed(PlayerAction::Crouch),
	in_set = MovementControlSet::UpdateState,
)]
fn charching_to_charge_crouching(
	players: Query<(Entity, &PlayerBody), With<Charging>>,
	assets: Res<CrouchingAssets>,
	mut commands: Commands,
) {
	for (player, body) in players.iter() {
		commands
			.entity(player)
			.remove::<Charging>()
			.insert(ChargeCrouching);
		to_crouching_assets(body, &mut commands, &assets);
	}
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_released(PlayerAction::Crouch),
	in_set = MovementControlSet::UpdateState,
)]
fn charge_crouching_to_charging(
	players: Query<(Entity, &PlayerBody), With<ChargeCrouching>>,
	assets: Res<StandingAssets>,
	mut commands: Commands,
) {
	for (player, body) in players.iter() {
		commands
			.entity(player)
			.remove::<ChargeCrouching>()
			.insert(Charging);
		to_standing_assets(body, &mut commands, &assets);
	}
}
