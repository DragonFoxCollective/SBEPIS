use std::marker::PhantomData;
use std::ops::Range;
use std::time::Duration;

use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_pretty_nice_input::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::player_controller::PlayerControllerPlugin;
use crate::player_controller::movement::MovingOptExt as _;
use crate::player_controller::movement::dash::Dash;
use crate::player_controller::movement::jump::JumpAssets;
use crate::player_controller::movement::trip::Trip;
use crate::player_controller::stamina::Stamina;
use crate::util::TransformExt;
use crate::{gravity::ComputedGravity, player_controller::movement::Moving};

use super::trip::{PlayerTripSettings, Tripping};

#[derive(Action)]
#[action(invalidate = false)]
pub struct ChargeDash;

#[derive(Action)]
#[action(invalidate = false)]
pub struct SpinDash;

#[auto_resource(plugin = PlayerControllerPlugin, derive, reflect, register, init)]
pub struct PlayerChargeSettings {
    pub max_time: Duration,
    pub spindash_speed: f32,
    pub spindash_stamina: f32,
}

impl Default for PlayerChargeSettings {
    fn default() -> Self {
        Self {
            max_time: Duration::from_secs_f32(1.0),
            spindash_speed: 10.0,
            spindash_stamina: 0.0,
        }
    }
}

#[auto_resource(plugin = PlayerControllerPlugin, derive, reflect, register)]
pub struct ChargeAssets {
    pub sound: Handle<AudioSource>,
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Startup)]
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(ChargeAssets {
        sound: asset_server.load("worms bazooka charge.mp3"),
    });
}

#[auto_component(plugin = PlayerControllerPlugin, derive(Debug, Default), reflect, register)]
pub struct Charging {
    pub charge_time: Duration,
}

impl Charging {
    /// Gets the maximum power and stamina cost possible from this charge and stamina.
    /// Returns None if not enough stamina to perform the charge.
    pub fn power_from_stamina(
        &self,
        settings: &PlayerChargeSettings,
        current_stamina: f32,
        stamina_cost: Range<f32>,
    ) -> Option<f32> {
        if current_stamina < stamina_cost.start {
            return None;
        }

        let spendable_stamina = current_stamina - stamina_cost.start;
        let stamina_per_charge_second =
            (stamina_cost.end - stamina_cost.start) / settings.max_time.as_secs_f32();
        let available_charge_time = spendable_stamina / stamina_per_charge_second;
        let charge_time = self
            .charge_time
            .as_secs_f32()
            .min(available_charge_time)
            .min(settings.max_time.as_secs_f32());
        let power = (charge_time / settings.max_time.as_secs_f32()).min(1.0);

        debug!(
            "Given max stamina {}, max power {}, min/max_stamina_cost {} {}, resulted in power {}",
            current_stamina,
            (self.charge_time.as_secs_f32() / settings.max_time.as_secs_f32()).min(1.0),
            stamina_cost.start,
            stamina_cost.end,
            power,
        );
        Some(power)
    }
}

#[auto_component(plugin = PlayerControllerPlugin, derive, reflect, register)]
struct ChargingSound(pub Entity);

#[auto_observer(plugin = PlayerControllerPlugin)]
fn spawn_charging_sound(add: On<Add, Charging>, mut commands: Commands, assets: Res<ChargeAssets>) {
    debug!("Charging!");

    let sound = commands
        .spawn((AudioPlayer(assets.sound.clone()), PlaybackSettings::DESPAWN))
        .id();

    commands.entity(add.entity).insert(ChargingSound(sound));
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn despawn_charging_sound(
    remove: On<Remove, Charging>,
    sounds: Query<&ChargingSound>,
    mut commands: Commands,
) {
    if let Ok(charging_sound) = sounds.get(remove.entity)
        && let Ok(mut sound) = commands.get_entity(charging_sound.0)
    {
        sound.despawn();
    }

    commands.entity(remove.entity).remove::<ChargingSound>();
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn charge_walking_to_trying_to_dash(dash: On<JustPressed<ChargeDash>>, mut commands: Commands) {
    // TODO: replace this with another event with params
    commands.trigger(JustPressed::<Dash> {
        input: dash.input,
        data: dash.data,
        _marker: PhantomData,
    });
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn charge_crouching_to_tripping(
    sprint: On<JustReleased<Trip>>,
    mut players: Query<(&ComputedGravity, &mut Velocity)>,
    mut commands: Commands,
    trip_settings: Res<PlayerTripSettings>,
) -> Result {
    let (gravity, mut velocity) = players.get_mut(sprint.input)?;
    velocity.linvel = gravity.up * trip_settings.upward_speed;
    commands
        .entity(sprint.input)
        .remove::<Charging>()
        .insert(Tripping::default());

    Ok(())
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Update)]
fn update_charge_time(mut players: Query<&mut Charging>, time: Res<Time>) {
    for mut charging_time in players.iter_mut() {
        charging_time.charge_time += time.delta();
    }
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn spindash(
    sprint: On<JustReleased<SpinDash>>,
    mut players: Query<(
        &mut Velocity,
        &Charging,
        &Moving,
        &Stamina,
        &GlobalTransform,
    )>,
    mut commands: Commands,
    charge_settings: Res<PlayerChargeSettings>,
    assets: Res<JumpAssets>,
) -> Result {
    let (mut velocity, charging, moving, stamina, transform) = players.get_mut(sprint.input)?;
    let input = Some(moving).as_input();
    let wish_dir = transform.transform_vector3(Vec3::new(input.x, 0.0, input.y));
    velocity.linvel = charging
        .power_from_stamina(
            &charge_settings,
            stamina.current,
            0.0..charge_settings.spindash_stamina,
        )
        .unwrap_or_default()
        * charge_settings.spindash_speed
        * wish_dir;
    commands.entity(sprint.input).remove::<Charging>();
    commands.spawn((
        AudioPlayer(assets.charge_jump_sound.clone()),
        PlaybackSettings::DESPAWN,
    ));
    Ok(())
}
