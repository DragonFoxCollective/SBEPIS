use std::time::Duration;

use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::gravity::AffectedByGravity;
use crate::player::PlayerControllerPlugin;
use crate::player::movement::charge::{Charging, PlayerChargeSettings};
use crate::player::movement::crouch::Crouching;
use crate::player::movement::dash::Dashing;
use crate::player::movement::grounded::GroundedContact;
use crate::player::movement::slide::Sliding;
use crate::player::stamina::Stamina;
use crate::prelude::*;
use crate::stats::{
    JumpHeight, JumpHoldTime, JumpStaminaCost, Stat, StatModifier, StatModifierHook,
};

#[auto_component(plugin = PlayerControllerPlugin, derive(Debug, Default), reflect, register)]
pub struct Jumping;

#[auto_component(plugin = PlayerControllerPlugin, derive, reflect, register)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = StatModifierHook::<JumpHeight>::default())]
struct CrouchJumpStats;
impl StatModifier<JumpHeight> for CrouchJumpStats {
    fn add(&self) -> f32 {
        0.5
    }
}

#[auto_component(plugin = PlayerControllerPlugin, derive, reflect, register)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = StatModifierHook::<JumpHeight>::default())]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = StatModifierHook::<JumpStaminaCost>::default())]
struct ChargeJumpStats {
    power: f32,
}
impl StatModifier<JumpHeight> for ChargeJumpStats {
    fn add(&self) -> f32 {
        self.power * 1.0
    }
}
impl StatModifier<JumpStaminaCost> for ChargeJumpStats {
    fn add(&self) -> f32 {
        self.power * 0.33
    }
}

#[auto_component(plugin = PlayerControllerPlugin, derive, reflect, register)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = StatModifierHook::<JumpHeight>::default())]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = StatModifierHook::<JumpStaminaCost>::default())]
struct ChargeCrouchJumpStats {
    power: f32,
}
impl StatModifier<JumpHeight> for ChargeCrouchJumpStats {
    fn add(&self) -> f32 {
        self.power * 1.5
    }
}
impl StatModifier<JumpStaminaCost> for ChargeCrouchJumpStats {
    fn add(&self) -> f32 {
        self.power * 0.66
    }
}

#[auto_component(plugin = PlayerControllerPlugin, derive(Debug, Default), reflect, register)]
struct JumpTimer {
    direction: Vec3,
    timer: Duration,
}

impl JumpTimer {
    fn checked_add_mut(&mut self, delta: Duration, max_time: Duration) -> bool {
        if self.timer + delta <= max_time {
            self.timer += delta;
            true
        } else {
            false
        }
    }
}

#[auto_resource(plugin = PlayerControllerPlugin, derive, reflect, register)]
pub struct JumpAssets {
    pub charge_jump_sound: Handle<AudioSource>,
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Startup)]
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(JumpAssets {
        charge_jump_sound: asset_server.load("unlicensed/worms bazooka shoot.mp3"),
    });
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn start_jump(
    jump: On<Add, Jumping>,
    players: Query<(
        Has<Crouching>,
        Has<Sliding>,
        Option<&Charging>,
        &GroundedContact,
    )>,
    charge_settings: Res<PlayerChargeSettings>,
    assets: Res<JumpAssets>,
    mut commands: Commands,
) -> Result {
    let player = jump.entity;
    let (crouching, sliding, charging, ground) = players.get(player)?;
    let crouching = crouching || sliding;
    let direction = ground.normal;

    commands.entity(player).insert(JumpTimer {
        direction,
        timer: Duration::ZERO,
    });
    if let Some(charging) = charging {
        commands.entity(player).remove::<Charging>();
        if crouching {
            commands.entity(player).insert(ChargeCrouchJumpStats {
                power: charging.power(&charge_settings),
            });
        } else {
            commands.entity(player).insert(ChargeJumpStats {
                power: charging.power(&charge_settings),
            });
        }
        commands.spawn((
            AudioPlayer(assets.charge_jump_sound.clone()),
            PlaybackSettings::DESPAWN,
        ));
    } else if crouching {
        commands.entity(player).insert(CrouchJumpStats);
    }

    // celeste superdash <3
    commands
        .entity(player)
        .remove::<Dashing>()
        .insert(AffectedByGravity);
    Ok(())
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn jump_release(remove: On<Remove, Jumping>, mut commands: Commands) {
    commands
        .entity(remove.entity)
        .remove::<JumpTimer>()
        .remove::<ChargeJumpStats>()
        .remove::<CrouchJumpStats>()
        .remove::<ChargeCrouchJumpStats>();
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Update)]
fn jump(
    mut players: Query<(
        Entity,
        &mut Velocity,
        &mut Stamina,
        &mut JumpTimer,
        &Stat<JumpHeight>,
        &Stat<JumpHoldTime>,
        &Stat<JumpStaminaCost>,
    )>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (
        entity,
        mut velocity,
        mut stamina,
        mut jump_timer,
        jump_height,
        jump_hold_time,
        jump_stamina_cost,
    ) in players.iter_mut()
    {
        if jump_timer.checked_add_mut(
            time.delta(),
            Duration::from_secs_f32(jump_hold_time.total()),
        ) && stamina.checked_sub_mut(jump_stamina_cost.total() * time.delta_secs())
        {
            let jump_speed = jump_height.total() / jump_hold_time.total();
            let speed_in_dir = velocity.linvel.length_projected_onto(jump_timer.direction);
            if speed_in_dir <= jump_speed {
                velocity.linvel += (jump_speed - speed_in_dir) * jump_timer.direction;
            }
        } else {
            commands.entity(entity).remove::<Jumping>();
        }
    }
}
