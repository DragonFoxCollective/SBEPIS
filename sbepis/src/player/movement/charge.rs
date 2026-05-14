use std::marker::PhantomData;
use std::ops::Range;
use std::time::Duration;

use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_pretty_nice_input::prelude::*;
use bevy_rapier3d::prelude::*;
use sbepistats::{DataTypeOp, OrderStatBeforeHook, Stat, StatSystems, StatType, StatTypeHook};

use super::trip::{PlayerTripSettings, Tripping};
use crate::gravity::ComputedGravity;
use crate::player::PlayerControllerPlugin;
use crate::player::movement::crouch::Crouching;
use crate::player::movement::dash::Dash;
use crate::player::movement::jump::{JumpAssets, JumpHeight, JumpStaminaCost};
use crate::player::movement::trip::Trip;
use crate::player::movement::{Moving, MovingOptExt as _};
use crate::player::stamina::Stamina;

#[derive(Action)]
#[action(invalidate = false)]
pub struct ChargeDash;

#[derive(Action)]
#[action(invalidate = false)]
pub struct SpinDash;

#[auto_resource(plugin = PlayerControllerPlugin, derive, reflect, register)]
pub struct ChargeAssets {
    pub sound: Handle<AudioSource>,
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Startup)]
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(ChargeAssets {
        sound: asset_server.load("unlicensed/worms bazooka charge.mp3"),
    });
}

#[auto_component(plugin = PlayerControllerPlugin, derive(Debug, Default), reflect, register)]
pub struct Charging {
    pub charge_time: Duration,
}

impl Charging {
    /// Gets the maximum power multiplier from this charge.
    pub fn power(&self, max_power: f32, max_time: f32) -> f32 {
        (self.charge_time.as_secs_f32() / max_time).min(max_power)
    }

    /// Gets the maximum power and stamina cost possible from this charge and stamina.
    /// Returns Err if not enough stamina to perform the charge.
    pub fn power_from_stamina(
        &self,
        max_power: f32,
        max_time: f32,
        current_stamina: f32,
        stamina_cost: Range<f32>,
    ) -> Result<f32> {
        if current_stamina < stamina_cost.start {
            return Err(BevyError::from("Not enough stamina to perform maneuver"));
        }

        let spendable_stamina = current_stamina - stamina_cost.start;
        let stamina_per_charge_second = (stamina_cost.end - stamina_cost.start) / max_time;
        let available_charge_time = spendable_stamina / stamina_per_charge_second;
        let charge_time = self.charge_time.as_secs_f32().min(available_charge_time);
        let power = (charge_time / max_time).min(max_power);

        Ok(power)
    }
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = PreUpdate, config(
    in_set = StatSystems::<JumpHeight>::Op(DataTypeOp::Add),
))]
fn charging_jump_height(
    mut players: Query<(
        &mut Stat<JumpHeight>,
        &Charging,
        &Stat<ChargedJumpHeight>,
        &Stat<ChargeMaxPower>,
        &Stat<ChargeMaxTime>,
    )>,
) {
    for (mut jump_height, charging, charged_jump_height, charge_max_power, charge_max_time) in
        players.iter_mut()
    {
        jump_height.add_modifier(
            charging.power(charge_max_power.total(), charge_max_time.total())
                * charged_jump_height.total(),
        );
    }
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = PreUpdate, config(
    in_set = StatSystems::<JumpStaminaCost>::Op(DataTypeOp::Add),
))]
fn charging_jump_stamina_cost(
    mut players: Query<(
        &mut Stat<JumpStaminaCost>,
        &Charging,
        Has<Crouching>,
        &Stat<ChargedJumpStaminaCost>,
        &Stat<ChargedCrouchJumpStaminaCost>,
        &Stat<ChargeMaxPower>,
        &Stat<ChargeMaxTime>,
    )>,
) {
    for (
        mut jump_stamina_cost,
        charging,
        crouching,
        charged_jump_stamina_cost,
        charged_crouch_jump_stamina_cost,
        charge_max_power,
        charge_max_time,
    ) in players.iter_mut()
    {
        jump_stamina_cost.add_modifier(
            charging.power(charge_max_power.total(), charge_max_time.total())
                * if crouching {
                    charged_crouch_jump_stamina_cost.total()
                } else {
                    charged_jump_stamina_cost.total()
                },
        );
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
        &Stat<ChargeMaxPower>,
        &Stat<ChargeMaxTime>,
        &Stat<SpindashStaminaCost>,
        &Stat<SpindashSpeed>,
    )>,
    mut commands: Commands,
    assets: Res<JumpAssets>,
) -> Result {
    let (
        mut velocity,
        charging,
        moving,
        stamina,
        transform,
        charge_max_power,
        charge_max_time,
        spindash_stamina_cost,
        spindash_speed,
    ) = players.get_mut(sprint.input)?;
    let wish_dir = Some(moving).as_motion(transform);
    velocity.linvel = charging
        .power_from_stamina(
            charge_max_power.total(),
            charge_max_time.total(),
            stamina.current,
            0.0..spindash_stamina_cost.total(),
        )
        .unwrap_or_default()
        * spindash_speed.total()
        * wish_dir;
    commands.entity(sprint.input).remove::<Charging>();
    commands.spawn((
        AudioPlayer(assets.charge_jump_sound.clone()),
        PlaybackSettings::DESPAWN,
    ));
    Ok(())
}

#[derive(StatType)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = StatTypeHook)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = OrderStatBeforeHook::<JumpHeight>::default())]
pub struct ChargedJumpHeight;

#[derive(StatType)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = StatTypeHook)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = OrderStatBeforeHook::<JumpStaminaCost>::default())]
pub struct ChargedJumpStaminaCost;

#[derive(StatType)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = StatTypeHook)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = OrderStatBeforeHook::<JumpStaminaCost>::default())]
pub struct ChargedCrouchJumpStaminaCost;

#[derive(StatType)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = StatTypeHook)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = OrderStatBeforeHook::<JumpHeight>::default())]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = OrderStatBeforeHook::<JumpStaminaCost>::default())]
pub struct ChargeMaxPower;

#[derive(StatType)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = StatTypeHook)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = OrderStatBeforeHook::<JumpHeight>::default())]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = OrderStatBeforeHook::<JumpStaminaCost>::default())]
pub struct ChargeMaxTime;

#[derive(StatType)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = StatTypeHook)]
pub struct SpindashStaminaCost;

#[derive(StatType)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = StatTypeHook)]
pub struct SpindashSpeed;
