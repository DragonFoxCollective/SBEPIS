use std::time::{Duration, Instant};

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
#[resource(plugin = PlayerControllerPlugin, init = PlayerChargeSettings {
	max_time: Duration::from_secs_f32(1.0),
	max_stamina_cost: 1.0,
})]
pub struct PlayerChargeSettings {
	pub max_time: Duration,
	pub max_stamina_cost: f32,
}

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

#[derive(Clone, Debug)]
pub struct ChargingInternal {
	pub start_time: Instant,
	pub max_time: Duration,
	pub max_stamina_cost: f32,
}

impl ChargingInternal {
	pub fn new(settings: &PlayerChargeSettings) -> Self {
		Self {
			start_time: Instant::now(),
			max_time: settings.max_time,
			max_stamina_cost: settings.max_stamina_cost,
		}
	}

	pub fn power(&self) -> f32 {
		(self.start_time.elapsed().as_secs_f32() / self.max_time.as_secs_f32()).min(1.0)
	}

	pub fn stamina_cost(&self) -> f32 {
		self.power() * self.max_stamina_cost
	}

	pub fn power_and_stamina_cost_from_stamina(&self, stamina: f32) -> (f32, f32) {
		let power = self.power() * (stamina / self.stamina_cost()).min(1.0);
		let stamina_cost = self.stamina_cost().min(stamina);
		debug!(
			"Given stamina {}, power {}, and stamina_cost {}, resulted in power {} and stamina_cost {}",
			stamina,
			self.power(),
			self.stamina_cost(),
			power,
			stamina_cost
		);
		(power, stamina_cost)
	}
}

#[derive(Component, Clone, Debug)]
pub struct Charging(pub ChargingInternal);

impl Charging {
	pub fn new(settings: &PlayerChargeSettings) -> Self {
		Self(ChargingInternal::new(settings))
	}

	pub fn power_and_stamina_cost_from_stamina(&self, stamina: f32) -> (f32, f32) {
		self.0.power_and_stamina_cost_from_stamina(stamina)
	}
}

impl From<ChargeCrouching> for Charging {
	fn from(charge_crouching: ChargeCrouching) -> Self {
		Self(charge_crouching.0)
	}
}

#[derive(Component, Clone, Debug)]
pub struct ChargeCrouching(pub ChargingInternal);

impl ChargeCrouching {
	pub fn power_and_stamina_cost_from_stamina(&self, stamina: f32) -> (f32, f32) {
		self.0.power_and_stamina_cost_from_stamina(stamina)
	}
}

impl From<Charging> for ChargeCrouching {
	fn from(charging: Charging) -> Self {
		Self(charging.0)
	}
}

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
	settings: Res<PlayerChargeSettings>,
) {
	for player in players.iter() {
		println!("Charging!");

		let sound = commands
			.spawn((AudioPlayer(assets.sound.clone()), PlaybackSettings::DESPAWN))
			.id();

		commands
			.entity(player)
			.insert(Charging::new(&settings))
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
			.remove::<ChargingSound>()
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
	players: Query<(Entity, &PlayerBody, &Charging)>,
	assets: Res<CrouchingAssets>,
	mut commands: Commands,
) {
	for (player, body, charging) in players.iter() {
		commands
			.entity(player)
			.remove::<Charging>()
			.insert(ChargeCrouching::from(charging.clone()));
		to_crouching_assets(body, &mut commands, &assets);
	}
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_released(PlayerAction::Crouch),
	in_set = MovementControlSet::UpdateState,
)]
fn charge_crouching_to_charging(
	players: Query<(Entity, &PlayerBody, &ChargeCrouching)>,
	assets: Res<StandingAssets>,
	mut commands: Commands,
) {
	for (player, body, charge_crouching) in players.iter() {
		commands
			.entity(player)
			.remove::<ChargeCrouching>()
			.remove::<ChargingSound>()
			.insert(Charging::from(charge_crouching.clone()));
		to_standing_assets(body, &mut commands, &assets);
	}
}
