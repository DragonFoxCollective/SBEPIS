use std::time::Duration;

use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_rapier3d::prelude::*;
use num_traits::Pow;
use sbepistats::{
    ConfigureStatTypeAddHook, Stat, StatModifierAdd, StatModifierAddHook, StatType, StatTypeHook,
};

use crate::gravity::AffectedByGravity;
use crate::player::PlayerControllerPlugin;
use crate::player::movement::charge::Charging;
use crate::player::movement::dash::Dashing;
use crate::player::movement::grounded::{Grounded, GroundedContact};
use crate::player::movement::{Moving, MovingOptExt as _};
use crate::player::stamina::Stamina;
use crate::prelude::*;

#[auto_component(plugin = PlayerControllerPlugin, derive(Debug, Default), reflect, register)]
pub struct Jumping;

#[auto_component(plugin = PlayerControllerPlugin, derive(Debug), reflect, register)]
struct JumpTimer {
    direction: Vec3,
    timer: Duration,
    speed: f32,
    stamina_drain: f32,
    jump_type: JumpType,
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

fn exp_decay(x: f32, y0: f32, x1: f32, y1: f32, limit: f32) -> f32 {
    // i hate logarithms
    (y0 - limit) * ((y1 - limit) / (y0 - limit)).pow(x / x1) + limit
}

#[derive(Debug, Reflect, Eq, PartialEq)]
enum JumpType {
    Neutral,
    LongJump,
    TwirlJump,
    Backflip,
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn start_jump(
    jump: On<Add, Jumping>,
    mut players: Query<(
        &GlobalTransform,
        Has<Charging>,
        &GroundedContact,
        Option<&mut JumpCombo>,
        &Velocity,
        Option<&Moving>,
        &Stat<JumpHeight>,
        &Stat<SpeedToJumpHeightMultiplier>,
        &Stat<TwirlJumpHeightMultiplier>,
        &Stat<LongJumpAngle>,
        &Stat<JumpHoldTime>,
        &Stat<JumpStaminaCost>,
    )>,
    assets: Res<JumpAssets>,
    mut commands: Commands,
) -> Result {
    let player = jump.entity;
    let (
        transform,
        charging,
        ground,
        combo,
        velocity,
        moving,
        jump_height,
        jump_height_mult,
        twirl_jump_mult,
        long_jump_angle,
        jump_hold_time,
        jump_stamina_cost,
    ) = players.get_mut(player)?;
    let input = moving.as_input();
    let speed = velocity.linvel.length();

    if let Some(mut combo) = combo {
        combo.0 += 1;
    } else {
        commands.entity(player).insert(JumpCombo::default());
    };

    let jump_type = if input != Vec2::ZERO {
        // angle from the horizontal, where the horizontal is the twirl jump and the vertical is the long/back jump
        let jump_type_angle_threshold = exp_decay(speed, 30.0, 20.0, 80.0, 90.0).to_radians();
        let jump_type_angle_right = input.to_angle().abs();
        let jump_type_angle_left = (input * vec2(-1.0, 1.0)).to_angle().abs();
        let jump_type_angle = jump_type_angle_left.min(jump_type_angle_right);
        if jump_type_angle < jump_type_angle_threshold {
            JumpType::TwirlJump
        } else if input.y > 0.0 {
            JumpType::LongJump
        } else {
            JumpType::Backflip
        }
    } else {
        JumpType::Neutral
    };

    let neutral_jump_height = jump_height.total() * (jump_height_mult.total() * speed + 1.0);
    let jump_height = match jump_type {
        JumpType::Neutral => neutral_jump_height,
        JumpType::LongJump => neutral_jump_height,
        JumpType::Backflip => neutral_jump_height,
        JumpType::TwirlJump => neutral_jump_height * twirl_jump_mult.total(),
    };
    let direction = match jump_type {
        JumpType::Neutral => ground.normal,
        JumpType::LongJump => velocity
            .linvel
            .rotate_towards(ground.normal, long_jump_angle.total()),
        JumpType::Backflip => ground.normal,
        JumpType::TwirlJump => transform.up().into(),
    };

    commands
        .entity(player)
        .remove::<JustLanded>()
        .insert(JumpTimer {
            direction,
            timer: Duration::from_secs_f32(jump_hold_time.total()),
            speed: jump_height / jump_hold_time.total(),
            stamina_drain: jump_stamina_cost.total() / jump_hold_time.total(),
            jump_type,
        });
    if charging {
        commands.entity(player).remove::<Charging>();
        commands.spawn((
            AudioPlayer(assets.charge_jump_sound.clone()),
            PlaybackSettings::DESPAWN,
        ));
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
    commands.entity(remove.entity).remove::<JumpTimer>();
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
pub struct SpeedToJumpHeightMultiplier;

#[derive(StatType)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = StatTypeHook)]
pub struct TwirlJumpHeightMultiplier;

#[derive(StatType)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = StatTypeHook)]
pub struct LongJumpAngle;

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
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = StatModifierAddHook::<JumpHeight>::default())]
pub struct JumpCombo(u32);

impl Default for JumpCombo {
    fn default() -> Self {
        Self(1)
    }
}

impl StatModifierAdd<JumpHeight> for JumpCombo {
    fn add(&self) -> f32 {
        self.0.min(3) as f32 * 0.5
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
