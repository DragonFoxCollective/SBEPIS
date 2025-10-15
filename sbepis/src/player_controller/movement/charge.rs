use std::time::{Duration, Instant};

use bevy::prelude::*;
use bevy_butler::*;
use bevy_pretty_nice_input::{Action, JustPressed, JustReleased};

use crate::gravity::{AffectedByGravity, ComputedGravity};
use crate::player_controller::PlayerControllerPlugin;
use crate::player_controller::movement::dash::Dash;
use crate::player_controller::movement::trip::Trip;

use super::stand::Standing;
use super::trip::{PlayerTripSettings, Tripping};

#[derive(Action)]
pub struct Charge;

#[derive(Action)]
pub struct ChargeCrouch;

#[derive(Action)]
pub struct ChargeWalk;

#[derive(Action)]
pub struct ChargeDash;

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

#[add_observer(plugin = PlayerControllerPlugin)]
fn standing_to_charging(
    charge: On<JustPressed<Charge>>,
    mut commands: Commands,
    assets: Res<ChargeAssets>,
    settings: Res<PlayerChargeSettings>,
) {
    debug!("Charging!");

    let sound = commands
        .spawn((AudioPlayer(assets.sound.clone()), PlaybackSettings::DESPAWN))
        .id();

    commands
        .entity(charge.input)
        .insert(ChargeStanding::new(&settings))
        .insert(ChargingSound(sound))
        .remove::<Standing>();
}

#[add_observer(plugin = PlayerControllerPlugin)]
fn charging_to_standing(
    charge: On<JustReleased<Charge>>,
    sounds: Query<&ChargingSound>,
    mut commands: Commands,
) {
    commands
        .entity(charge.input)
        .remove::<ChargeStanding>()
        .remove::<ChargingSound>()
        .insert(Standing);

    if let Ok(charging_sound) = sounds.get(charge.input)
        && let Ok(mut sound) = commands.get_entity(charging_sound.0)
    {
        sound.despawn();
    }
}

#[add_observer(plugin = PlayerControllerPlugin)]
fn charging_to_charge_crouching(
    crouch: On<JustPressed<ChargeCrouch>>,
    players: Query<&ChargeStanding>,
    mut commands: Commands,
) -> Result {
    let charging = players.get(crouch.input)?;
    commands
        .entity(crouch.input)
        .remove::<ChargeStanding>()
        .insert(ChargeCrouching::from(charging.clone()));
    Ok(())
}

#[add_observer(plugin = PlayerControllerPlugin)]
fn charge_crouching_to_charging(
    crouch: On<JustReleased<ChargeCrouch>>,
    players: Query<&ChargeCrouching>,
    mut commands: Commands,
) -> Result {
    let charging = players.get(crouch.input)?;
    commands
        .entity(crouch.input)
        .remove::<ChargeCrouching>()
        .insert(ChargeStanding::from(charging.clone()));
    Ok(())
}

#[add_observer(plugin = PlayerControllerPlugin)]
fn charging_to_charge_walking(
    walk: On<JustPressed<ChargeWalk>>,
    players: Query<&ChargeStanding>,
    mut commands: Commands,
) -> Result {
    let charging = players.get(walk.input)?;
    commands
        .entity(walk.input)
        .remove::<ChargeStanding>()
        .insert(ChargeWalking::from(charging.clone()));
    Ok(())
}

#[add_observer(plugin = PlayerControllerPlugin)]
fn charge_walking_to_charging(
    walk: On<JustReleased<ChargeWalk>>,
    players: Query<&ChargeWalking>,
    mut commands: Commands,
) -> Result {
    let charging = players.get(walk.input)?;
    commands
        .entity(walk.input)
        .remove::<ChargeWalking>()
        .insert(ChargeStanding::from(charging.clone()));
    Ok(())
}

#[add_observer(plugin = PlayerControllerPlugin)]
pub fn charge_walking_to_trying_to_dash(dash: On<JustPressed<ChargeDash>>, mut commands: Commands) {
    // TODO: replace this with another event with params
    commands.trigger(JustPressed {
        input: dash.input,
        action: Dash,
        data: dash.data,
    });
}

#[add_observer(plugin = PlayerControllerPlugin)]
pub fn charge_crouching_to_tripping(
    sprint: On<JustPressed<Trip>>, // TODO: equivalent to ButtonRelease of ChargeDash
    players: Query<(Option<&ChargingSound>, &ComputedGravity)>,
    mut commands: Commands,
    trip_settings: Res<PlayerTripSettings>,
) -> Result {
    let (charging_sound, gravity) = players.get(sprint.input)?;
    commands
        .entity(sprint.input)
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

    Ok(())
}
