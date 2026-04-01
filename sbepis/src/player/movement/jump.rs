use std::time::Duration;

use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_rapier3d::prelude::*;
use sbepistats::{
    ConfigureStatTypeAddHook, Stat, StatModifierAdd, StatModifierAddHook, StatType, StatTypeHook,
};

use crate::gravity::AffectedByGravity;
use crate::player::PlayerControllerPlugin;
use crate::player::movement::charge::Charging;
use crate::player::movement::crouch::Crouching;
use crate::player::movement::dash::Dashing;
use crate::player::movement::grounded::{Grounded, GroundedContact};
use crate::player::movement::slide::Sliding;
use crate::player::stamina::Stamina;
use crate::prelude::*;

#[auto_component(plugin = PlayerControllerPlugin, derive(Debug, Default), reflect, register)]
pub struct Jumping;

#[auto_component(plugin = PlayerControllerPlugin, derive, reflect, register)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = StatModifierAddHook::<JumpHeight>::default())]
struct CrouchJumpStats;
impl StatModifierAdd<JumpHeight> for CrouchJumpStats {
    fn add(&self) -> f32 {
        0.5
    }
}

#[auto_component(plugin = PlayerControllerPlugin, derive(Debug, Default), reflect, register)]
struct JumpTimer {
    direction: Vec3,
    timer: Duration,
    speed: f32,
    stamina_drain: f32,
}

impl JumpTimer {
    fn checked_sub_mut(&mut self, delta: Duration) -> bool {
        if let Some(new) = self.timer.checked_sub(delta) {
            self.timer = new;
            true
        } else {
            self.timer = Duration::ZERO;
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
    mut players: Query<(
        Has<Crouching>,
        Has<Sliding>,
        Has<Charging>,
        &GroundedContact,
        Option<&mut JumpCombo>,
        &Stat<JumpHeight>,
        &Stat<JumpHoldTime>,
        &Stat<JumpStaminaCost>,
    )>,
    assets: Res<JumpAssets>,
    mut commands: Commands,
) -> Result {
    let player = jump.entity;
    let (
        crouching,
        sliding,
        charging,
        ground,
        combo,
        jump_height,
        jump_hold_time,
        jump_stamina_cost,
    ) = players.get_mut(player)?;
    let crouching = crouching || sliding;
    let direction = ground.normal;

    if let Some(mut combo) = combo {
        combo.0 += 1;
    } else {
        commands.entity(player).insert(JumpCombo::default());
    }

    commands
        .entity(player)
        .remove::<JustLanded>()
        .insert(JumpTimer {
            direction,
            timer: Duration::from_secs_f32(jump_hold_time.total()),
            speed: jump_height.total() / jump_hold_time.total(),
            stamina_drain: jump_stamina_cost.total() / jump_hold_time.total(),
        });
    if charging {
        commands.entity(player).remove::<Charging>();
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
        .remove::<CrouchJumpStats>();
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Update)]
fn update_jump(
    mut players: Query<(Entity, &mut Velocity, &mut Stamina, &mut JumpTimer)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut velocity, mut stamina, mut jump_timer) in players.iter_mut() {
        if jump_timer.checked_sub_mut(time.delta())
            && stamina.checked_sub_mut(jump_timer.stamina_drain * time.delta_secs())
        {
            let speed_in_dir = velocity.linvel.length_projected_onto(jump_timer.direction);
            if speed_in_dir <= jump_timer.speed {
                velocity.linvel += (jump_timer.speed - speed_in_dir) * jump_timer.direction;
            }
        } else {
            commands.entity(entity).remove::<Jumping>();
        }
    }
}

#[derive(StatType)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = StatTypeHook)]
pub struct JumpHeight;

#[derive(StatType)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = StatTypeHook)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = ConfigureStatTypeAddHook)]
pub struct JumpStaminaCost;

#[derive(StatType)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = StatTypeHook)]
pub struct JumpHoldTime;

/// The max time a grounded player has to jump to activate the next combo.
#[derive(StatType)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = StatTypeHook)]
pub struct JumpComboMaxTime;

/// The max jump combo level.
#[derive(StatType)]
#[stat_type(u32)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = StatTypeHook)]
pub struct JumpComboMaxLevel;

#[auto_component(plugin = PlayerControllerPlugin, derive, reflect, register)]
pub struct JumpCombo(u32);

impl Default for JumpCombo {
    fn default() -> Self {
        Self(1)
    }
}

#[auto_component(plugin = PlayerControllerPlugin, derive(Default), reflect, register)]
pub struct JustLanded(Duration);

#[auto_observer(plugin = PlayerControllerPlugin)]
fn add_landing_timer(
    add: On<Add, Grounded>,
    filter: Query<(), With<Stat<JumpComboMaxTime>>>,
    mut commands: Commands,
) {
    if filter.get(add.entity).is_err() {
        return;
    }
    commands.entity(add.entity).insert(JustLanded::default());
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Update)]
fn update_landing_timer(
    mut players: Query<(Entity, &mut JustLanded, &Stat<JumpComboMaxTime>)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (player, mut just_landed, max_time) in players.iter_mut() {
        just_landed.0 += time.delta();
        if just_landed.0.as_secs_f32() > max_time.total() {
            commands
                .entity(player)
                .remove::<JustLanded>()
                .remove::<JumpCombo>();
        }
    }
}

#[auto_component(plugin = PlayerControllerPlugin, derive, reflect, register)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = StatModifierAddHook::<JumpHeight>::default())]
struct JumpCombo1Stats;
impl StatModifierAdd<JumpHeight> for JumpCombo1Stats {
    fn add(&self) -> f32 {
        0.5
    }
}

#[auto_component(plugin = PlayerControllerPlugin, derive, reflect, register)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = StatModifierAddHook::<JumpHeight>::default())]
struct JumpCombo2Stats;
impl StatModifierAdd<JumpHeight> for JumpCombo2Stats {
    fn add(&self) -> f32 {
        1.0
    }
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Update, config(
    after = update_landing_timer
))]
fn update_combo_stats(
    players: Query<&JumpCombo>,
    changes: Query<Entity, Changed<JumpCombo>>,
    mut removals: RemovedComponents<JumpCombo>,
    mut commands: Commands,
) {
    for player in changes.iter().chain(removals.read()) {
        commands
            .entity(player)
            .remove::<JumpCombo1Stats>()
            .remove::<JumpCombo2Stats>();

        if let Ok(combo) = players.get(player) {
            match combo.0 {
                1 => {
                    commands.entity(player).insert(JumpCombo1Stats);
                }
                _ => {
                    commands.entity(player).insert(JumpCombo2Stats);
                }
            }
        }
    }
}
