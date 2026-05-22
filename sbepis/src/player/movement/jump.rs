use std::f32::consts::{FRAC_PI_2, PI};
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
use crate::player::camera::PlayerOfCamera;
use crate::player::camera::controls::SlerpYaw;
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
    jump_type: JumpType,
    timer: Duration,
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

fn exp_decay(x: f32, y0: f32, x1: f32, y1: f32, limit: f32) -> f32 {
    // i hate logarithms
    (y0 - limit) * ((y1 - limit) / (y0 - limit)).pow(x / x1) + limit
}

#[derive(Debug, Reflect)]
enum JumpType {
    Neutral {
        upward_velocity: Vec3,
    },
    LongJump {
        upward_velocity: Vec3,
        upward_time: Duration,
        forward_velocity: Vec3,
    },
    TwirlJump {
        upward_velocity: Vec3,
        upward_time: Duration,
        forward_velocity: Vec3,
        sideward_velocity: Vec3,
    },
    Backflip {
        upward_velocity: Vec3,
        backward_velocity: Vec3,
    },
    Helicopter {
        previous_speed: f32,
    },
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn start_jump(
    jump: On<Add, Jumping>,
    mut players: Query<(
        &GlobalTransform,
        &PlayerOfCamera,
        Has<Charging>,
        &GroundedContact,
        Option<&mut JumpCombo>,
        &Velocity,
        Option<&Moving>,
        (
            &Stat<JumpHeight>,
            &Stat<SpeedToJumpHeightMultiplier>,
            &Stat<TwirlJumpHeight>,
            &Stat<SpeedToTwirlJumpHeightSpeedMultiplier>,
            &Stat<TwirlJumpSpeedMultiplier>,
            &Stat<LongJumpHeight>,
            &Stat<SpeedToLongJumpHeightSpeedMultiplier>,
            &Stat<LongJumpSpeedMultiplier>,
            &Stat<BackflipSpeedMultiplier>,
            &Stat<JumpHoldTime>,
            &Stat<JumpStaminaCost>,
        ),
    )>,
    cameras: Query<&GlobalTransform>,
    assets: Res<JumpAssets>,
    mut commands: Commands,
) -> Result {
    let player = jump.entity;
    let (
        transform,
        camera,
        charging,
        ground,
        combo,
        velocity,
        moving,
        (
            jump_height,
            jump_height_mult,
            twirl_jump_height,
            twirl_jump_height_mult,
            twirl_jump_mult,
            long_jump_height,
            long_jump_height_mult,
            long_jump_mult,
            backflip_mult,
            jump_hold_time,
            jump_stamina_cost,
        ),
    ) = players.get_mut(player)?;
    let camera_transform = cameras.get(**camera)?;
    let input_motion = moving.as_camera_motion(camera_transform, transform);
    let raw_input_angle = moving.as_input().angle_to(Vec2::Y);
    let input_angle = raw_input_angle.abs();
    let linvel = velocity.linvel.reject_from(transform.up().into());
    let speed = linvel.length();
    let hold_time = jump_hold_time.total();

    if let Some(mut combo) = combo {
        combo.0 += 1;
    } else {
        commands.entity(player).insert(JumpCombo::default());
    };

    let jump_type = if !input_angle.is_nan() {
        // angle from the horizontal, where the horizontal is the twirl jump and the vertical is the long/back jump
        let jump_type_angle_threshold = exp_decay(speed, 30.0, 20.0, 80.0, 90.0).to_radians();
        let jump_type_angle = FRAC_PI_2 - input_angle.min(PI - input_angle);
        if jump_type_angle < jump_type_angle_threshold {
            let vertical_speed = speed * twirl_jump_height_mult.total();
            JumpType::TwirlJump {
                upward_time: Duration::from_secs_f32(twirl_jump_height.total() / vertical_speed),
                upward_velocity: transform.up() * vertical_speed,
                forward_velocity: linvel * twirl_jump_mult.total(),
                sideward_velocity: input_motion * speed * twirl_jump_mult.total(),
            }
        } else if input_angle < FRAC_PI_2 {
            let vertical_speed = speed * long_jump_height_mult.total();
            JumpType::LongJump {
                upward_time: Duration::from_secs_f32(long_jump_height.total() / vertical_speed),
                upward_velocity: transform.up() * vertical_speed,
                forward_velocity: input_motion * speed * long_jump_mult.total(),
            }
        } else {
            commands
                .entity(player)
                .insert(SlerpYaw::new(raw_input_angle));
            JumpType::Backflip {
                upward_velocity: ground.normal
                    * jump_height.total()
                    * (jump_height_mult.total() * speed + 1.0)
                    / hold_time,
                backward_velocity: input_motion * speed * backflip_mult.total(),
            }
        }
    } else {
        JumpType::Neutral {
            upward_velocity: ground.normal
                * jump_height.total()
                * (jump_height_mult.total() * speed + 1.0)
                / hold_time,
        }
    };
    debug!("Jumping {jump_type:?}");

    commands
        .entity(player)
        .remove::<JustLanded>()
        .insert(JumpTimer {
            jump_type,
            timer: Duration::from_secs_f32(hold_time),
            stamina_drain: jump_stamina_cost.total() / hold_time,
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

fn go_up(velocity: Vec3, linvel: &mut Vec3, velocity_mult: f32) {
    let (direction, speed) = velocity.normalize_and_length();
    let impulse = speed * velocity_mult - linvel.length_projected_onto(velocity);
    if impulse > 0.0 {
        *linvel += impulse * direction;
    }
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Update)]
fn update_jump(
    mut players: Query<
        (
            Entity,
            &mut Velocity,
            &mut Stamina,
            &mut JumpTimer,
            &GlobalTransform,
        ),
        With<Jumping>,
    >,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut velocity, mut stamina, mut jump_timer, transform) in players.iter_mut() {
        if jump_timer.checked_sub_mut(time.delta())
            && stamina.checked_sub_mut(jump_timer.stamina_drain * time.delta_secs())
        {
            match jump_timer.jump_type {
                JumpType::Neutral { upward_velocity } => {
                    go_up(upward_velocity, &mut velocity.linvel, 1.0);
                }
                JumpType::LongJump {
                    upward_velocity,
                    ref mut upward_time,
                    forward_velocity,
                } => {
                    if let Some(upw) = upward_time.checked_sub(time.delta()) {
                        *upward_time = upw;
                        go_up(upward_velocity, &mut velocity.linvel, 1.0);
                    } else {
                        go_up(upward_velocity, &mut velocity.linvel, 0.0);
                    }
                    go_up(forward_velocity, &mut velocity.linvel, 1.0);
                }
                JumpType::TwirlJump {
                    upward_velocity,
                    ref mut upward_time,
                    forward_velocity,
                    sideward_velocity,
                } => {
                    if let Some(upw) = upward_time.checked_sub(time.delta()) {
                        *upward_time = upw;
                        go_up(upward_velocity, &mut velocity.linvel, 1.0);
                    } else {
                        go_up(upward_velocity, &mut velocity.linvel, 0.0);
                    }
                    let lerp = jump_timer.timer.as_secs_f32().clamp(0.0, 1.0);
                    go_up(forward_velocity, &mut velocity.linvel, lerp);
                    go_up(sideward_velocity, &mut velocity.linvel, 1.0 - lerp);
                }
                JumpType::Backflip {
                    upward_velocity,
                    backward_velocity,
                } => {
                    let lerp = jump_timer.timer.as_secs_f32().clamp(0.0, 1.0);
                    go_up(upward_velocity, &mut velocity.linvel, 1.0);
                    go_up(backward_velocity, &mut velocity.linvel, 1.0 - lerp);
                }
                JumpType::Helicopter { .. } => {
                    go_up(transform.up().into(), &mut velocity.linvel, 0.0);
                }
            }
        } else {
            if let JumpType::Helicopter { previous_speed } = jump_timer.jump_type {
                velocity.linvel += transform.up() * previous_speed;
            }
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
pub struct TwirlJumpHeight;

#[derive(StatType)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = StatTypeHook)]
pub struct SpeedToTwirlJumpHeightSpeedMultiplier;

#[derive(StatType)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = StatTypeHook)]
pub struct TwirlJumpSpeedMultiplier;

#[derive(StatType)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = StatTypeHook)]
pub struct LongJumpHeight;

#[derive(StatType)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = StatTypeHook)]
pub struct SpeedToLongJumpHeightSpeedMultiplier;

#[derive(StatType)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = StatTypeHook)]
pub struct LongJumpSpeedMultiplier;

#[derive(StatType)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = StatTypeHook)]
pub struct BackflipSpeedMultiplier;

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

#[derive(StatType)]
#[stat_type(u32)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = StatTypeHook)]
pub struct JumpComboMaxLevel;

#[auto_component(plugin = PlayerControllerPlugin, derive, reflect, register)]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = StatModifierAddHook::<JumpHeight>::default())]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = StatModifierAddHook::<TwirlJumpSpeedMultiplier>::default())]
#[auto_plugin_build_hook(plugin = PlayerControllerPlugin, hook = StatModifierAddHook::<LongJumpSpeedMultiplier>::default())]
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

impl StatModifierAdd<TwirlJumpSpeedMultiplier> for JumpCombo {
    fn add(&self) -> f32 {
        self.0.min(3) as f32 * 0.1
    }
}

impl StatModifierAdd<LongJumpSpeedMultiplier> for JumpCombo {
    fn add(&self) -> f32 {
        self.0.min(3) as f32 * 0.1
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
