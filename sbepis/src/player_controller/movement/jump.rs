use std::f32;
use std::time::Duration;

use crate::gravity::AffectedByGravity;
use crate::player_controller::PlayerControllerPlugin;
use crate::player_controller::movement::charge::{Charging, PlayerChargeSettings};
use crate::player_controller::stamina::Stamina;
use crate::util::MapRangeBetween;
use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_pretty_nice_input::bevy_event_chain::*;
use bevy_pretty_nice_input::bundles::observe;
use bevy_pretty_nice_input::{Action, Condition, ConditionedBindingUpdate};
use bevy_rapier3d::prelude::*;

use super::dash::Dashing;

const JUMP_MULTIPLIER: f32 = 5.0;

fn jump_speed_from_height(jump_height: f32) -> f32 {
    let normal_gravity = crate::NORMAL_GRAVITY;
    (2.0 * normal_gravity * JUMP_MULTIPLIER * jump_height).sqrt()
}

#[auto_component(plugin = PlayerControllerPlugin, derive(Debug, Default), reflect, register)]
pub struct Jumping;

#[auto_component(plugin = PlayerControllerPlugin, derive(Debug, Default), reflect, register)]
pub struct CrouchJumping;

#[auto_component(plugin = PlayerControllerPlugin, derive(Debug, Default), reflect, register)]
pub struct SlideJumping;

#[auto_component(plugin = PlayerControllerPlugin, derive(Debug, Default), reflect, register)]
pub struct ChargeJumping;

#[auto_component(plugin = PlayerControllerPlugin, derive(Debug, Default), reflect, register)]
pub struct ChargeCrouchJumping;

#[derive(Reflect, Debug, Default)]
pub struct ChargeJumpSettings {
    pub min_speed: f32,
    pub max_speed: f32,
    pub min_stamina_cost: f32,
    pub max_stamina_cost: f32,
    pub variable_time: f32,
}

impl ChargeJumpSettings {
    fn max_stamina(&self) -> f32 {
        self.max_stamina_cost
    }
}

#[derive(Reflect, Debug, Default)]
pub struct JumpSettings {
    pub speed: f32,
    pub stamina_cost: f32,
    pub variable_time: f32,
}

impl JumpSettings {
    fn max_stamina(&self) -> f32 {
        self.stamina_cost
    }
}

#[auto_resource(plugin = PlayerControllerPlugin, derive, reflect, register, init)]
pub struct PlayerJumpSettings {
    pub jump: JumpSettings,
    pub high_jump: JumpSettings,
    pub slide_jump: JumpSettings,
    pub charge_jump: ChargeJumpSettings,
    pub unreal_air_jump: ChargeJumpSettings,
}

impl Default for PlayerJumpSettings {
    fn default() -> Self {
        let jump_height = 1.0;
        let high_jump_height = 1.5;
        let charge_jump_height = 2.0;
        let unreal_air_jump_height = 2.5;
        let variable_time = 0.3;

        Self {
            jump: JumpSettings {
                speed: jump_speed_from_height(jump_height),
                stamina_cost: 0.0,
                variable_time,
            },
            high_jump: JumpSettings {
                speed: jump_speed_from_height(high_jump_height),
                stamina_cost: 0.0,
                variable_time,
            },
            slide_jump: JumpSettings {
                speed: jump_speed_from_height(high_jump_height),
                stamina_cost: 0.0,
                variable_time,
            },
            charge_jump: ChargeJumpSettings {
                min_speed: jump_speed_from_height(jump_height),
                max_speed: jump_speed_from_height(charge_jump_height),
                min_stamina_cost: 0.0,
                max_stamina_cost: 0.33,
                variable_time,
            },
            unreal_air_jump: ChargeJumpSettings {
                min_speed: jump_speed_from_height(high_jump_height),
                max_speed: jump_speed_from_height(unreal_air_jump_height),
                min_stamina_cost: 0.0,
                max_stamina_cost: 0.66,
                variable_time,
            },
        }
    }
}

#[auto_component(plugin = PlayerControllerPlugin, derive(Debug, Default), reflect, register)]
pub struct JumpTimer {
    pub variable_timer: Duration,
    pub timer_max: f32,
    pub stamina_cost: f32,
    pub speed: f32,
}

#[auto_resource(plugin = PlayerControllerPlugin, derive, reflect, register)]
pub struct JumpAssets {
    pub charge_jump_sound: Handle<AudioSource>,
}

macro_rules! define_jump_release {
    ($func_name:ident, $action_type:ty) => {
        #[auto_observer(plugin = PlayerControllerPlugin)]
        fn $func_name(
            remove: On<Remove, $action_type>,
            mut jump_tracker: Query<(&mut JumpTimer, &mut Stamina)>,
            mut commands: Commands,
        ) -> Result {
            match jump_tracker.get_mut(remove.entity) {
                Ok((jump_tracker, mut stamina)) => {
                    stamina.current -= jump_tracker.stamina_cost;
                    commands.entity(remove.entity).remove::<JumpTimer>();
                }
                Err(_e) => {}
            }
            Ok(())
        }
    };
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Startup)]
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(JumpAssets {
        charge_jump_sound: asset_server.load("worms bazooka shoot.mp3"),
    });
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn jump(
    jump: On<Add, Jumping>,
    settings: Res<PlayerJumpSettings>,
    mut commands: Commands,
) -> Result {
    commands.entity(jump.entity).insert(JumpTimer {
        timer_max: settings.jump.variable_time,
        speed: settings.jump.speed,
        stamina_cost: settings.jump.stamina_cost,
        variable_timer: Duration::from_secs(0),
    });
    Ok(())
}

define_jump_release!(jump_release, Jumping);

#[auto_observer(plugin = PlayerControllerPlugin)]
fn crouch_jump(
    jump: On<Add, CrouchJumping>,
    settings: Res<PlayerJumpSettings>,
    mut commands: Commands,
) -> Result {
    commands.entity(jump.entity).insert(JumpTimer {
        timer_max: settings.high_jump.variable_time,
        speed: settings.high_jump.speed,
        stamina_cost: settings.high_jump.stamina_cost,
        variable_timer: Duration::from_secs(0),
    });
    Ok(())
}

define_jump_release!(crouch_jump_release, CrouchJumping);

#[auto_observer(plugin = PlayerControllerPlugin)]
fn slide_jump(
    jump: On<Add, SlideJumping>,
    settings: Res<PlayerJumpSettings>,
    mut commands: Commands,
) -> Result {
    commands.entity(jump.entity).insert(JumpTimer {
        timer_max: settings.high_jump.variable_time,
        speed: settings.high_jump.speed,
        stamina_cost: settings.high_jump.stamina_cost,
        variable_timer: Duration::from_secs(0),
    });
    Ok(())
}

define_jump_release!(slide_jump_release, SlideJumping);

#[auto_observer(plugin = PlayerControllerPlugin)]
fn charge_jump(
    jump: On<Add, ChargeJumping>,
    mut commands: Commands,
    players: Query<(&Stamina, &Charging)>,
    charge_settings: Res<PlayerChargeSettings>,
    assets: Res<JumpAssets>,
    settings: Res<PlayerJumpSettings>,
) -> Result {
    let (stamina, charging) = players.get(jump.entity)?;
    let (power, stamina_cost) = charging
        .power_and_stamina_cost_from_stamina(
            &charge_settings,
            stamina.current,
            settings.charge_jump.min_stamina_cost,
            settings.charge_jump.max_stamina_cost,
        )
        .ok_or(BevyError::from(
            "Don't have enough stamina to charge jump, despite being in jump transition",
        ))?;
    commands.entity(jump.entity).insert(JumpTimer {
        variable_timer: Duration::from_secs(0),
        timer_max: settings.charge_jump.variable_time,
        stamina_cost,
        speed: power.map_from_01(settings.charge_jump.min_speed..settings.charge_jump.max_speed),
    });
    commands.spawn((
        AudioPlayer(assets.charge_jump_sound.clone()),
        PlaybackSettings::DESPAWN,
    ));
    Ok(())
}

define_jump_release!(charge_jump_release, ChargeJumping);

#[auto_observer(plugin = PlayerControllerPlugin)]
fn charge_crouch_jump(
    jump: On<Add, ChargeCrouchJumping>,
    mut commands: Commands,
    players: Query<(&Stamina, &Charging)>,
    charge_settings: Res<PlayerChargeSettings>,
    assets: Res<JumpAssets>,
    settings: Res<PlayerJumpSettings>,
) -> Result {
    let (stamina, charging) = players.get(jump.entity)?;
    let (power, stamina_cost) = charging
        .power_and_stamina_cost_from_stamina(
            &charge_settings,
            stamina.current,
            settings.unreal_air_jump.min_stamina_cost,
            settings.unreal_air_jump.max_stamina_cost,
        )
        .ok_or(BevyError::from(
            "Don't have enough stamina to unreal air, despite being in jump transition",
        ))?;
    commands.entity(jump.entity).insert(JumpTimer {
        variable_timer: Duration::from_secs(0),
        timer_max: settings.unreal_air_jump.variable_time,
        stamina_cost,
        speed: power
            .map_from_01(settings.unreal_air_jump.min_speed..settings.unreal_air_jump.max_speed),
    });
    commands.spawn((
        AudioPlayer(assets.charge_jump_sound.clone()),
        PlaybackSettings::DESPAWN,
    ));
    Ok(())
}

define_jump_release!(charge_crouch_jump_release, ChargeCrouchJumping);

#[auto_system(plugin = PlayerControllerPlugin, schedule = Update)]
fn do_jump(
    mut player_bodies: Query<(
        Entity,
        &mut Velocity,
        &Transform,
        &mut Stamina,
        &mut JumpTimer,
    )>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut velocity, transform, mut stamina, mut jump_tracker) in player_bodies.iter_mut()
    {
        if jump_tracker.variable_timer.as_secs_f32() <= jump_tracker.timer_max {
            if transform.up().dot(velocity.linvel) < 0.0 {
                velocity.linvel = velocity.linvel.reject_from(transform.up().into());
            }
            let len = velocity.linvel.length();
            velocity.linvel += transform.up() * (jump_tracker.speed - len).max(0.0);
        } else {
            stamina.current -= jump_tracker.stamina_cost;
            commands
                .entity(entity)
                .remove::<Dashing>()
                .remove::<JumpTimer>()
                .insert(AffectedByGravity);
        }
        jump_tracker.variable_timer += time.delta();
    }
}

macro_rules! define_stamina_condition {
    ($struct_name:ident, $settings_field:ident) => {
        #[auto_component(plugin = PlayerControllerPlugin, derive, reflect, register)]
        pub struct $struct_name;

        impl Condition for $struct_name {
            fn bundle<A: Action>(&self) -> impl Bundle {
                observe(
                    |update: On<ConditionedBindingUpdate>,
                     mut commands: Commands,
                     players: Query<&Stamina>,
                     settings: Res<PlayerJumpSettings>|
                     -> Result {
                        let stamina = players.get(update.input)?;
                        if update.data.is_zero()
                            || stamina.current >= settings.$settings_field.max_stamina()
                        {
                            update.trigger_next(&mut commands);
                        }
                        Ok(())
                    },
                )
            }
        }
    };
}

define_stamina_condition!(HasEnoughStaminaToJump, jump);

define_stamina_condition!(HasEnoughStaminaToCrouchJump, high_jump);

define_stamina_condition!(HasEnoughStaminaToSlideJump, slide_jump);

define_stamina_condition!(HasEnoughStaminaToChargeJump, charge_jump);

define_stamina_condition!(HasEnoughStaminaToChargeCrouchJump, unreal_air_jump);
