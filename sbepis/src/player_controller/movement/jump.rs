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
use bevy_pretty_nice_input::{
    Action, Condition, ConditionedBindingUpdate, JustPressed, JustReleased,
};
use bevy_rapier3d::prelude::*;

use super::dash::Dashing;

#[derive(Action)]
#[action(invalidate = false)]
pub struct Jump;

#[derive(Action)]
#[action(invalidate = false)]
pub struct CrouchJump;

#[derive(Action)]
#[action(invalidate = false)]
pub struct SlideJump;

#[derive(Action)]
#[action(invalidate = false)]
pub struct ChargeJump;

#[derive(Action)]
#[action(invalidate = false)]
pub struct ChargeCrouchJump;

#[derive(Reflect, Default, Debug)]
pub enum JumpType {
    #[default]
    Normal,
    Crouch,
    Charge,
    Slide,
    Unreal,
}

#[derive(Reflect, Debug, Default)]
pub struct JumpSettings {
    pub min_speed: f32,
    pub max_speed: f32,
    pub min_stamina_cost: f32,
    pub max_stamina_cost: f32,
    pub variable_time: f32,
}

#[auto_resource(plugin = PlayerControllerPlugin, derive, reflect, register, init)]
pub struct PlayerJumpSettings {
    pub jump: JumpSettings,
    pub high_jump: JumpSettings,
    pub slide_jump: JumpSettings,
    pub charge_jump: JumpSettings,
    pub unreal_air_jump: JumpSettings,
}

impl Default for PlayerJumpSettings {
    fn default() -> Self {
        let jump_height = 1.0;
        let high_jump_height = 1.5;
        let charge_jump_height = 2.0;
        let unreal_air_jump_height = 2.5;
        let variable_time = 5.0;

        Self {
            jump: JumpSettings {
                min_speed: jump_speed_from_height(jump_height),
                max_speed: jump_speed_from_height(jump_height),
                min_stamina_cost: 0.0,
                max_stamina_cost: 0.0,
                variable_time,
            },
            high_jump: JumpSettings {
                min_speed: jump_speed_from_height(high_jump_height),
                max_speed: jump_speed_from_height(high_jump_height),
                min_stamina_cost: 0.0,
                max_stamina_cost: 0.0,
                variable_time,
            },
            slide_jump: JumpSettings {
                min_speed: jump_speed_from_height(high_jump_height),
                max_speed: jump_speed_from_height(high_jump_height),
                min_stamina_cost: 0.0,
                max_stamina_cost: 0.0,
                variable_time,
            },
            charge_jump: JumpSettings {
                min_speed: jump_speed_from_height(jump_height),
                max_speed: jump_speed_from_height(charge_jump_height),
                min_stamina_cost: 0.0,
                max_stamina_cost: 0.33,
                variable_time,
            },
            unreal_air_jump: JumpSettings {
                min_speed: jump_speed_from_height(high_jump_height),
                max_speed: jump_speed_from_height(unreal_air_jump_height),
                min_stamina_cost: 0.0,
                max_stamina_cost: 0.66,
                variable_time,
            },
        }
    }
}

fn jump_speed_from_height(jump_height: f32) -> f32 {
    let normal_gravity = crate::NORMAL_GRAVITY;
    (2.0 * normal_gravity * jump_height).sqrt()
}

impl PlayerJumpSettings {
    fn match_by_type(&self, jump_type: &JumpType) -> &JumpSettings {
        match jump_type {
            JumpType::Normal => &self.jump,
            JumpType::Crouch => &self.high_jump,
            JumpType::Charge => &self.charge_jump,
            JumpType::Slide => &self.slide_jump,
            JumpType::Unreal => &self.unreal_air_jump,
        }
    }
}

#[auto_component(plugin = PlayerControllerPlugin, derive(Debug, Default), reflect, register)]
pub struct JumpTracker {
    pub variable_timer: Duration,
    pub jump_type: JumpType,
    pub stamina_cost: f32,
    pub speed: f32,
}

impl JumpTracker {
    fn new_standard(jump_type: JumpType, settings: &Res<PlayerJumpSettings>) -> Self {
        return Self {
            speed: settings.match_by_type(&jump_type).min_speed,
            stamina_cost: settings.match_by_type(&jump_type).min_stamina_cost,
            jump_type: jump_type,
            variable_timer: Duration::from_secs(0),
        };
    }
}

#[auto_resource(plugin = PlayerControllerPlugin, derive, reflect, register)]
pub struct JumpAssets {
    pub charge_jump_sound: Handle<AudioSource>,
}

macro_rules! define_jump_release {
    ($func_name:ident, $action_type:ty) => {
        #[auto_observer(plugin = PlayerControllerPlugin)]
        fn $func_name(
            jump: On<JustReleased<$action_type>>,
            mut jump_tracker: Query<(&mut JumpTracker, &mut Stamina)>,
            mut commands: Commands,
        ) -> Result {
            let (jump_tracker, mut stamina) = jump_tracker.get_mut(jump.input)?;
            debug!("hi from release");
            stamina.current -= jump_tracker.stamina_cost;
            commands.entity(jump.input).remove::<JumpTracker>();
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
    jump: On<JustPressed<Jump>>,
    settings: Res<PlayerJumpSettings>,
    mut commands: Commands,
) -> Result {
    debug!("hi from start");
    commands
        .entity(jump.input)
        .insert(JumpTracker::new_standard(JumpType::Normal, &settings));
    Ok(())
}
//
define_jump_release!(jump_release, Jump);

#[auto_observer(plugin = PlayerControllerPlugin)]
fn crouch_jump(
    jump: On<JustPressed<CrouchJump>>,
    settings: Res<PlayerJumpSettings>,
    mut commands: Commands,
) -> Result {
    commands
        .entity(jump.input)
        .insert(JumpTracker::new_standard(JumpType::Crouch, &settings));
    Ok(())
}

define_jump_release!(crouch_jump_release, CrouchJump);

#[auto_observer(plugin = PlayerControllerPlugin)]
fn slide_jump(
    jump: On<JustPressed<SlideJump>>,
    settings: Res<PlayerJumpSettings>,
    mut commands: Commands,
) -> Result {
    commands
        .entity(jump.input)
        .insert(JumpTracker::new_standard(JumpType::Slide, &settings));
    Ok(())
}

define_jump_release!(slide_jump_release, SlideJump);

#[auto_observer(plugin = PlayerControllerPlugin)]
fn charge_jump(
    jump: On<JustPressed<ChargeJump>>,
    mut commands: Commands,
    players: Query<(&Stamina, &Charging)>,
    charge_settings: Res<PlayerChargeSettings>,
    assets: Res<JumpAssets>,
    settings: Res<PlayerJumpSettings>,
) -> Result {
    let (stamina, charging) = players.get(jump.input)?;
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
    commands.entity(jump.input).insert(JumpTracker {
        variable_timer: Duration::from_secs(0),
        jump_type: JumpType::Charge,
        stamina_cost,
        speed: power.map_from_01(settings.charge_jump.min_speed..settings.charge_jump.max_speed),
    });
    commands.spawn((
        AudioPlayer(assets.charge_jump_sound.clone()),
        PlaybackSettings::DESPAWN,
    ));
    Ok(())
}

define_jump_release!(charge_jump_release, ChargeJump);

#[auto_observer(plugin = PlayerControllerPlugin)]
fn charge_crouch_jump(
    jump: On<JustPressed<ChargeCrouchJump>>,
    mut commands: Commands,
    players: Query<(&Stamina, &Charging)>,
    charge_settings: Res<PlayerChargeSettings>,
    assets: Res<JumpAssets>,
    settings: Res<PlayerJumpSettings>,
) -> Result {
    let (stamina, charging) = players.get(jump.input)?;
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
    commands.entity(jump.input).insert(JumpTracker {
        variable_timer: Duration::from_secs(0),
        jump_type: JumpType::Unreal,
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

define_jump_release!(charge_crouch_jump_release, ChargeCrouchJump);

#[auto_system(plugin = PlayerControllerPlugin, schedule = Update)]
fn do_jump(
    mut player_bodies: Query<(
        Entity,
        &mut Velocity,
        &Transform,
        &mut Stamina,
        &mut JumpTracker,
    )>,
    time: Res<Time>,
    settings: Res<PlayerJumpSettings>,
    mut commands: Commands,
) {
    for (entity, mut velocity, transform, mut stamina, mut jump_tracker) in player_bodies.iter_mut()
    {
        if jump_tracker.variable_timer.as_secs_f32()
            <= settings
                .match_by_type(&jump_tracker.jump_type)
                .variable_time
        {
            if transform.up().dot(velocity.linvel) < 0.0 {
                velocity.linvel = velocity.linvel.reject_from(transform.up().into());
            }
            velocity.linvel += transform.up() * jump_tracker.speed;
        } else {
            stamina.current -= jump_tracker.stamina_cost;
            commands
                .entity(entity)
                .remove::<Dashing>()
                .remove::<JumpTracker>()
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
                            || stamina.current >= settings.$settings_field.max_stamina_cost
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
