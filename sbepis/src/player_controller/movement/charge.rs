use std::marker::PhantomData;
use std::time::Duration;

use bevy::prelude::*;
use bevy_butler::*;
use bevy_pretty_nice_input::{Action, JustPressed, Off};

use crate::gravity::{AffectedByGravity, ComputedGravity};
use crate::player_controller::PlayerControllerPlugin;
use crate::player_controller::movement::dash::Dash;
use crate::player_controller::movement::trip::Trip;

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

#[derive(Component, Debug, Default)]
pub struct ChargingTime {
    pub charge_time: Duration,
}

impl ChargingTime {
    pub fn power(&self, settings: &PlayerChargeSettings) -> f32 {
        (self.charge_time.as_secs_f32() / settings.max_time.as_secs_f32()).min(1.0)
    }

    pub fn power_and_stamina_cost_from_stamina(
        &self,
        settings: &PlayerChargeSettings,
        current_stamina: f32,
        min_stamina_cost: f32,
        max_stamina_cost: f32,
    ) -> Option<(f32, f32)> {
        let stamina_cost = max_stamina_cost.min(current_stamina);
        if stamina_cost < min_stamina_cost {
            return None;
        }
        let power = self.power(settings) * (stamina_cost - min_stamina_cost)
            / (max_stamina_cost - min_stamina_cost);
        debug!(
            "Given stamina {}, power {}, min/max_stamina_cost {} {}, resulted in power {} and stamina_cost {}",
            current_stamina,
            self.power(settings),
            min_stamina_cost,
            max_stamina_cost,
            power,
            stamina_cost
        );
        Some((power, stamina_cost))
    }
}

#[derive(Component, Default)]
pub struct ChargeStanding;

#[derive(Component, Default)]
pub struct ChargeCrouching;

#[derive(Component, Default)]
pub struct ChargeWalking;

#[derive(Component)]
pub struct ChargingSound(pub Entity);

#[add_observer(plugin = PlayerControllerPlugin)]
fn spawn_charging_sound(
    charge: On<JustPressed<Charge>>,
    mut commands: Commands,
    assets: Res<ChargeAssets>,
) {
    debug!("Charging!");

    let sound = commands
        .spawn((AudioPlayer(assets.sound.clone()), PlaybackSettings::DESPAWN))
        .id();

    commands.entity(charge.input).insert(ChargingSound(sound));
}

#[add_observer(plugin = PlayerControllerPlugin)]
fn despawn_charging_sound(
    charge: On<JustPressed<Off<Charge>>>,
    sounds: Query<&ChargingSound>,
    mut commands: Commands,
) {
    if let Ok(charging_sound) = sounds.get(charge.input)
        && let Ok(mut sound) = commands.get_entity(charging_sound.0)
    {
        sound.despawn();
    }

    commands.entity(charge.input).remove::<ChargingSound>();
}

#[add_observer(plugin = PlayerControllerPlugin)]
pub fn charge_walking_to_trying_to_dash(dash: On<JustPressed<ChargeDash>>, mut commands: Commands) {
    // TODO: replace this with another event with params
    commands.trigger(JustPressed::<Dash> {
        input: dash.input,
        data: dash.data,
        _marker: PhantomData,
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

#[add_system(plugin = PlayerControllerPlugin, schedule = Update)]
fn update_charge_time(mut players: Query<&mut ChargingTime>, time: Res<Time>) {
    for mut charging_time in players.iter_mut() {
        charging_time.charge_time += time.delta();
    }
}
