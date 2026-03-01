use std::ops::Range;
use std::time::Duration;

use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_pretty_nice_input::bevy_event_chain::*;
use bevy_pretty_nice_input::bundles::observe;
use bevy_pretty_nice_input::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::gravity::AffectedByGravity;
use crate::player_controller::PlayerControllerPlugin;
use crate::player_controller::movement::charge::{Charging, PlayerChargeSettings};
use crate::player_controller::stamina::Stamina;
use crate::util::MapRange as _;

use super::dash::Dashing;

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

struct JumpSettingsBuilder {
    height: f32,
    stamina_cost: f32,
    max_hold_time: f32,
}

impl JumpSettingsBuilder {
    fn speed(&self) -> f32 {
        self.height / self.max_hold_time
    }

    fn build(&self) -> JumpSettings {
        JumpSettings {
            speed: self.speed(),
            stamina_cost: self.stamina_cost,
            max_hold_time: self.max_hold_time,
        }
    }

    fn build_charge(&self, uncharged: &JumpSettingsBuilder) -> ChargeJumpSettings {
        ChargeJumpSettings {
            speed: uncharged.speed()..self.speed(),
            stamina_cost: uncharged.stamina_cost..self.stamina_cost,
            max_hold_time: uncharged.stamina_cost..self.max_hold_time,
        }
    }
}

#[derive(Reflect, Debug, Default)]
pub struct JumpSettings {
    pub speed: f32,
    pub stamina_cost: f32,
    pub max_hold_time: f32,
}

impl JumpSettings {
    fn timer(&self) -> JumpTimer {
        JumpTimer {
            timer: Duration::ZERO,
            timer_max: Duration::from_secs_f32(self.max_hold_time),
            stamina_cost: self.stamina_cost,
            speed: self.speed,
        }
    }
}

#[derive(Reflect, Debug, Default)]
pub struct ChargeJumpSettings {
    pub speed: Range<f32>,
    pub stamina_cost: Range<f32>,
    pub max_hold_time: Range<f32>,
}

impl ChargeJumpSettings {
    fn timer_from_power(&self, power: f32) -> JumpTimer {
        JumpTimer {
            timer: Duration::ZERO,
            timer_max: Duration::from_secs_f32(
                power.map_range(self.max_hold_time.start..self.max_hold_time.end),
            ),
            stamina_cost: power.map_range(self.stamina_cost.clone()),
            speed: power.map_range(self.speed.clone()),
        }
    }

    fn timer_from_charge(
        &self,
        charge_settings: &PlayerChargeSettings,
        charging: &Charging,
        stamina: &Stamina,
    ) -> Result<JumpTimer> {
        let power = charging
            .power_from_stamina(charge_settings, stamina.current, self.stamina_cost.clone())
            .ok_or(BevyError::from("Don't have enough stamina to charge jump"))?;
        Ok(self.timer_from_power(power))
    }
}

#[auto_resource(plugin = PlayerControllerPlugin, derive, reflect, register, init)]
pub struct PlayerJumpSettings {
    pub jump: JumpSettings,
    pub crouch_jump: JumpSettings,
    pub charge_jump: ChargeJumpSettings,
    pub charge_crouch_jump: ChargeJumpSettings,
}

impl Default for PlayerJumpSettings {
    fn default() -> Self {
        let jump = JumpSettingsBuilder {
            height: 1.0,
            stamina_cost: 0.0,
            max_hold_time: 0.3,
        };
        let crouch_jump = JumpSettingsBuilder {
            height: 1.5,
            stamina_cost: 0.0,
            max_hold_time: 0.3,
        };
        let charge_jump = JumpSettingsBuilder {
            height: 2.0,
            stamina_cost: 0.33,
            max_hold_time: 0.3,
        };
        let charge_crouch_jump = JumpSettingsBuilder {
            height: 2.5,
            stamina_cost: 0.66,
            max_hold_time: 0.3,
        };

        Self {
            jump: jump.build(),
            crouch_jump: crouch_jump.build(),
            charge_jump: charge_jump.build_charge(&jump),
            charge_crouch_jump: charge_crouch_jump.build_charge(&crouch_jump),
        }
    }
}

#[auto_component(plugin = PlayerControllerPlugin, derive(Debug, Default), reflect, register)]
pub struct JumpTimer {
    pub timer: Duration,
    pub timer_max: Duration,
    pub stamina_cost: f32,
    pub speed: f32,
}

#[auto_resource(plugin = PlayerControllerPlugin, derive, reflect, register)]
pub struct JumpAssets {
    pub charge_jump_sound: Handle<AudioSource>,
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Startup)]
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(JumpAssets {
        charge_jump_sound: asset_server.load("worms bazooka shoot.mp3"),
    });
}

#[auto_event(plugin = PlayerControllerPlugin, target(entity), derive, reflect, register)]
pub struct JumpRelease {
    pub entity: Entity,
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn jump(
    jump: On<Add, Jumping>,
    settings: Res<PlayerJumpSettings>,
    mut commands: Commands,
) -> Result {
    commands.entity(jump.entity).insert(settings.jump.timer());
    Ok(())
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn jump_release(remove: On<Remove, Jumping>, mut commands: Commands) {
    commands.trigger(JumpRelease {
        entity: remove.entity,
    });
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn crouch_jump(
    jump: On<Add, CrouchJumping>,
    settings: Res<PlayerJumpSettings>,
    mut commands: Commands,
) -> Result {
    commands
        .entity(jump.entity)
        .insert(settings.crouch_jump.timer());
    Ok(())
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn crouch_jump_release(remove: On<Remove, CrouchJumping>, mut commands: Commands) {
    commands.trigger(JumpRelease {
        entity: remove.entity,
    });
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn slide_jump(
    jump: On<Add, SlideJumping>,
    settings: Res<PlayerJumpSettings>,
    mut commands: Commands,
) -> Result {
    commands
        .entity(jump.entity)
        .insert(settings.crouch_jump.timer());
    Ok(())
}
#[auto_observer(plugin = PlayerControllerPlugin)]
fn slide_jump_release(remove: On<Remove, SlideJumping>, mut commands: Commands) {
    commands.trigger(JumpRelease {
        entity: remove.entity,
    });
}

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
    commands
        .entity(jump.entity)
        .insert(
            settings
                .charge_jump
                .timer_from_charge(&charge_settings, charging, stamina)?,
        );
    commands.entity(jump.entity).remove::<Charging>();
    commands.spawn((
        AudioPlayer(assets.charge_jump_sound.clone()),
        PlaybackSettings::DESPAWN,
    ));
    Ok(())
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn charge_jump_release(remove: On<Remove, ChargeJumping>, mut commands: Commands) {
    commands.trigger(JumpRelease {
        entity: remove.entity,
    });
}

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
    commands
        .entity(jump.entity)
        .insert(settings.charge_crouch_jump.timer_from_charge(
            &charge_settings,
            charging,
            stamina,
        )?);
    commands.entity(jump.entity).remove::<Charging>();
    commands.spawn((
        AudioPlayer(assets.charge_jump_sound.clone()),
        PlaybackSettings::DESPAWN,
    ));
    Ok(())
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn charge_crouch_jump_release(remove: On<Remove, ChargeCrouchJumping>, mut commands: Commands) {
    commands.trigger(JumpRelease {
        entity: remove.entity,
    });
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn jump_release_observer(
    jump: On<JumpRelease>,
    mut query: Query<(&JumpTimer, &mut Stamina)>,
    mut commands: Commands,
) {
    match query.get_mut(jump.entity) {
        Ok((jump_timer, mut stamina)) => {
            stamina.current -= jump_timer.stamina_cost;
            commands.entity(jump.entity).remove::<JumpTimer>();
        }
        Err(_e) => {}
    }
}

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
    for (entity, mut velocity, transform, mut stamina, mut jump_timer) in player_bodies.iter_mut() {
        if jump_timer.timer <= jump_timer.timer_max {
            if transform.up().dot(velocity.linvel) < 0.0 {
                velocity.linvel = velocity.linvel.reject_from(transform.up().into());
            }
            let len = velocity.linvel.length();
            velocity.linvel += transform.up() * (jump_timer.speed - len).max(0.0);
        } else {
            stamina.current -= jump_timer.stamina_cost;
            commands
                .entity(entity)
                .remove::<Dashing>()
                .remove::<JumpTimer>()
                .insert(AffectedByGravity);
        }
        jump_timer.timer += time.delta();
    }
}

#[auto_component(plugin = PlayerControllerPlugin, derive, reflect, register)]
pub struct HasEnoughStaminaToJump;

impl Condition for HasEnoughStaminaToJump {
    fn bundle<A: Action>(&self) -> impl Bundle {
        observe(
            |update: On<ConditionedBindingUpdate>,
             mut commands: Commands,
             players: Query<&Stamina>,
             settings: Res<PlayerJumpSettings>|
             -> Result {
                let stamina = players.get(update.input)?;
                if update.data.is_zero() || stamina.current >= settings.jump.stamina_cost {
                    update.trigger_next(&mut commands);
                }
                Ok(())
            },
        )
    }
}

#[auto_component(plugin = PlayerControllerPlugin, derive, reflect, register)]
pub struct HasEnoughStaminaToCrouchJump;

impl Condition for HasEnoughStaminaToCrouchJump {
    fn bundle<A: Action>(&self) -> impl Bundle {
        observe(
            |update: On<ConditionedBindingUpdate>,
             mut commands: Commands,
             players: Query<&Stamina>,
             settings: Res<PlayerJumpSettings>|
             -> Result {
                let stamina = players.get(update.input)?;
                if update.data.is_zero() || stamina.current >= settings.crouch_jump.stamina_cost {
                    update.trigger_next(&mut commands);
                }
                Ok(())
            },
        )
    }
}

#[auto_component(plugin = PlayerControllerPlugin, derive, reflect, register)]
pub struct HasEnoughStaminaToSlideJump;

impl Condition for HasEnoughStaminaToSlideJump {
    fn bundle<A: Action>(&self) -> impl Bundle {
        observe(
            |update: On<ConditionedBindingUpdate>,
             mut commands: Commands,
             players: Query<&Stamina>,
             settings: Res<PlayerJumpSettings>|
             -> Result {
                let stamina = players.get(update.input)?;
                if update.data.is_zero() || stamina.current >= settings.crouch_jump.stamina_cost {
                    update.trigger_next(&mut commands);
                }
                Ok(())
            },
        )
    }
}

#[auto_component(plugin = PlayerControllerPlugin, derive, reflect, register)]
pub struct HasEnoughStaminaToChargeJump;

impl Condition for HasEnoughStaminaToChargeJump {
    fn bundle<A: Action>(&self) -> impl Bundle {
        observe(
            |update: On<ConditionedBindingUpdate>,
             mut commands: Commands,
             players: Query<&Stamina>,
             settings: Res<PlayerJumpSettings>|
             -> Result {
                let stamina = players.get(update.input)?;
                if update.data.is_zero()
                    || stamina.current >= settings.charge_jump.stamina_cost.start
                {
                    update.trigger_next(&mut commands);
                }
                Ok(())
            },
        )
    }
}

#[auto_component(plugin = PlayerControllerPlugin, derive, reflect, register)]
pub struct HasEnoughStaminaToChargeCrouchJump;

impl Condition for HasEnoughStaminaToChargeCrouchJump {
    fn bundle<A: Action>(&self) -> impl Bundle {
        observe(
            |update: On<ConditionedBindingUpdate>,
             mut commands: Commands,
             players: Query<&Stamina>,
             settings: Res<PlayerJumpSettings>|
             -> Result {
                let stamina = players.get(update.input)?;
                if update.data.is_zero()
                    || stamina.current >= settings.charge_crouch_jump.stamina_cost.start
                {
                    update.trigger_next(&mut commands);
                }
                Ok(())
            },
        )
    }
}
