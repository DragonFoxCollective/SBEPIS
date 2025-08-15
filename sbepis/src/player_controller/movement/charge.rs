use std::time::{Duration, Instant};

use bevy::prelude::*;
use bevy_butler::*;

use crate::gravity::{AffectedByGravity, ComputedGravity};
use crate::input::{button_is_released, button_just_pressed, button_just_released};
use crate::player_controller::movement::MovementControlSet;
use crate::player_controller::movement::dash::add_trying_to_dash;
use crate::player_controller::{PlayerAction, PlayerControllerPlugin};

use super::dash::TryingToDash;
use super::grounded::EffectiveGrounded;
use super::stand::Standing;
use super::trip::{PlayerTripSettings, Tripping};

#[derive(Resource)]
#[insert_resource(plugin = PlayerControllerPlugin, init = PlayerChargeSettings {
	max_time: Duration::from_secs_f32(1.0),
})]
pub struct PlayerChargeSettings {
    pub max_time: Duration,
}

#[derive(Resource)]
pub struct ChargeAssets {
    pub sound: Handle<AudioSource>,
}

#[add_system(plugin = PlayerControllerPlugin, schedule = Startup)]
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(ChargeAssets {
        sound: asset_server.load("worms bazooka charge.mp3"),
    });
}

#[derive(Clone, Debug)]
pub struct ChargingInternal {
    pub start_time: Instant,
    pub max_time: Duration,
}

impl ChargingInternal {
    pub fn new(settings: &PlayerChargeSettings) -> Self {
        Self {
            start_time: Instant::now(),
            max_time: settings.max_time,
        }
    }

    pub fn power(&self) -> f32 {
        (self.start_time.elapsed().as_secs_f32() / self.max_time.as_secs_f32()).min(1.0)
    }

    pub fn power_and_stamina_cost_from_stamina(
        &self,
        current_stamina: f32,
        min_stamina_cost: f32,
        max_stamina_cost: f32,
    ) -> Option<(f32, f32)> {
        let stamina_cost = max_stamina_cost.min(current_stamina);
        if stamina_cost < min_stamina_cost {
            return None;
        }
        let power = self.power() * (stamina_cost - min_stamina_cost)
            / (max_stamina_cost - min_stamina_cost);
        debug!(
            "Given stamina {}, power {}, min/max_stamina_cost {} {}, resulted in power {} and stamina_cost {}",
            current_stamina,
            self.power(),
            min_stamina_cost,
            max_stamina_cost,
            power,
            stamina_cost
        );
        Some((power, stamina_cost))
    }
}

#[derive(Component, Clone, Debug)]
pub struct ChargeStanding(pub ChargingInternal);

impl ChargeStanding {
    pub fn new(settings: &PlayerChargeSettings) -> Self {
        Self(ChargingInternal::new(settings))
    }

    pub fn power_and_stamina_cost_from_stamina(
        &self,
        current_stamina: f32,
        min_stamina_cost: f32,
        max_stamina_cost: f32,
    ) -> Option<(f32, f32)> {
        self.0.power_and_stamina_cost_from_stamina(
            current_stamina,
            min_stamina_cost,
            max_stamina_cost,
        )
    }
}

impl From<ChargeCrouching> for ChargeStanding {
    fn from(charge_crouching: ChargeCrouching) -> Self {
        Self(charge_crouching.0)
    }
}

impl From<ChargeWalking> for ChargeStanding {
    fn from(charge_walking: ChargeWalking) -> Self {
        Self(charge_walking.0)
    }
}

#[derive(Component, Clone, Debug)]
pub struct ChargeCrouching(pub ChargingInternal);

impl ChargeCrouching {
    pub fn power_and_stamina_cost_from_stamina(
        &self,
        current_stamina: f32,
        min_stamina_cost: f32,
        max_stamina_cost: f32,
    ) -> Option<(f32, f32)> {
        self.0.power_and_stamina_cost_from_stamina(
            current_stamina,
            min_stamina_cost,
            max_stamina_cost,
        )
    }
}

impl From<ChargeStanding> for ChargeCrouching {
    fn from(charging: ChargeStanding) -> Self {
        Self(charging.0)
    }
}

#[derive(Component, Clone, Debug)]
pub struct ChargeWalking(pub ChargingInternal);

impl ChargeWalking {
    pub fn power_and_stamina_cost_from_stamina(
        &self,
        current_stamina: f32,
        min_stamina_cost: f32,
        max_stamina_cost: f32,
    ) -> Option<(f32, f32)> {
        self.0.power_and_stamina_cost_from_stamina(
            current_stamina,
            min_stamina_cost,
            max_stamina_cost,
        )
    }
}

impl From<ChargeStanding> for ChargeWalking {
    fn from(charging: ChargeStanding) -> Self {
        Self(charging.0)
    }
}

#[derive(Component)]
pub struct ChargingSound(pub Entity);

#[add_system(
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
            Without<ChargeStanding>,
        ),
    >,
    mut commands: Commands,
    assets: Res<ChargeAssets>,
    settings: Res<PlayerChargeSettings>,
) {
    for player in players.iter() {
        debug!("Charging!");

        let sound = commands
            .spawn((AudioPlayer(assets.sound.clone()), PlaybackSettings::DESPAWN))
            .id();

        commands
            .entity(player)
            .insert(ChargeStanding::new(&settings))
            .insert(ChargingSound(sound))
            .remove::<TryingToDash>()
            .remove::<Standing>();
    }
}

#[add_system(
    plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_is_released(PlayerAction::Sprint),
	in_set = MovementControlSet::UpdateState,
)]
fn charging_to_standing(
    players: Query<(Entity, &ChargingSound), With<ChargeStanding>>,
    mut commands: Commands,
) {
    for (player, charging_sound) in players.iter() {
        commands
            .entity(player)
            .remove::<ChargeStanding>()
            .remove::<ChargingSound>()
            .insert(Standing);

        if let Ok(mut sound) = commands.get_entity(charging_sound.0) {
            sound.despawn();
        }
    }
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_pressed(PlayerAction::Crouch),
	in_set = MovementControlSet::UpdateState,
)]
fn charching_to_charge_crouching(
    players: Query<(Entity, &ChargeStanding)>,
    mut commands: Commands,
) {
    for (player, charging) in players.iter() {
        commands
            .entity(player)
            .remove::<ChargeStanding>()
            .insert(ChargeCrouching::from(charging.clone()));
    }
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_released(PlayerAction::Crouch),
	in_set = MovementControlSet::UpdateState,
)]
fn charge_crouching_to_charging(
    players: Query<(Entity, &ChargeCrouching)>,
    mut commands: Commands,
) {
    for (player, charge_crouching) in players.iter() {
        commands
            .entity(player)
            .remove::<ChargeCrouching>()
            .insert(ChargeStanding::from(charge_crouching.clone()));
    }
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_pressed(PlayerAction::Move),
	in_set = MovementControlSet::UpdateState,
)]
fn charching_to_charge_walking(players: Query<(Entity, &ChargeStanding)>, mut commands: Commands) {
    for (player, charging) in players.iter() {
        commands
            .entity(player)
            .remove::<ChargeStanding>()
            .insert(ChargeWalking::from(charging.clone()));
    }
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_released(PlayerAction::Move),
	in_set = MovementControlSet::UpdateState,
)]
fn charge_walking_to_charging(players: Query<(Entity, &ChargeWalking)>, mut commands: Commands) {
    for (player, charge_walking) in players.iter() {
        commands
            .entity(player)
            .remove::<ChargeWalking>()
            .insert(ChargeStanding::from(charge_walking.clone()));
    }
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_released(PlayerAction::Sprint),
	in_set = MovementControlSet::UpdateState,
)]
pub fn charge_walking_to_trying_to_dash(
    players: Query<Entity, With<ChargeWalking>>,
    mut commands: Commands,
) {
    for player in players.iter() {
        commands.entity(player).insert(TryingToDash::default());
    }
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_released(PlayerAction::Sprint),
	in_set = MovementControlSet::UpdateState,
)]
pub fn charge_crouching_to_tripping(
    players: Query<(Entity, Option<&ChargingSound>, &ComputedGravity), With<ChargeCrouching>>,
    mut commands: Commands,
    trip_settings: Res<PlayerTripSettings>,
) {
    for (player, charging_sound, gravity) in players.iter() {
        commands
            .entity(player)
            .remove::<ChargeCrouching>()
            .remove::<AffectedByGravity>()
            .insert(Tripping::new(
                gravity.up,
                gravity.up * trip_settings.upward_speed,
            ));

        if let Some(charging_sound) = charging_sound
            && let Ok(mut sound) = commands.get_entity(charging_sound.0)
        {
            sound.despawn();
        }
    }
}
