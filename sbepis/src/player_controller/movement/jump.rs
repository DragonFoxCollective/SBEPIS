use std::ops::Range;
use std::time::Duration;

use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::gravity::AffectedByGravity;
use crate::player_controller::PlayerControllerPlugin;
use crate::player_controller::movement::charge::{Charging, PlayerChargeSettings};
use crate::player_controller::movement::crouch::Crouching;
use crate::player_controller::movement::dash::Dashing;
use crate::player_controller::movement::slide::Sliding;
use crate::player_controller::stamina::Stamina;
use crate::util::{MapRange as _, Vec3Ext as _};

#[auto_component(plugin = PlayerControllerPlugin, derive(Debug, Default), reflect, register)]
pub struct Jumping;

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
            timer: Duration::from_secs_f32(self.max_hold_time),
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
            speed: power.map_range(self.speed.clone()),
            stamina_cost: power.map_range(self.stamina_cost.clone()),
            timer: Duration::from_secs_f32(
                power.map_range(self.max_hold_time.start..self.max_hold_time.end),
            ),
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
struct JumpTimer {
    speed: f32,
    stamina_cost: f32,
    timer: Duration,
}

impl JumpTimer {
    fn checked_sub_mut(&mut self, delta: Duration) -> bool {
        if let Some(timer) = self.timer.checked_sub(delta) {
            self.timer = timer;
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
        charge_jump_sound: asset_server.load("worms bazooka shoot.mp3"),
    });
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn start_jump(
    jump: On<Add, Jumping>,
    players: Query<(Has<Crouching>, Has<Sliding>, Option<&Charging>, &Stamina)>,
    settings: Res<PlayerJumpSettings>,
    charge_settings: Res<PlayerChargeSettings>,
    assets: Res<JumpAssets>,
    mut commands: Commands,
) -> Result {
    let player = jump.entity;
    let (crouching, sliding, charging, stamina) = players.get(player)?;
    let crouching = crouching || sliding;

    if let Some(charging) = charging {
        commands.entity(player).remove::<Charging>().insert(
            if crouching {
                &settings.charge_crouch_jump
            } else {
                &settings.charge_jump
            }
            .timer_from_charge(&charge_settings, charging, stamina)?,
        );
        commands.spawn((
            AudioPlayer(assets.charge_jump_sound.clone()),
            PlaybackSettings::DESPAWN,
        ));
    } else {
        commands.entity(player).insert(
            if crouching {
                &settings.crouch_jump
            } else {
                &settings.jump
            }
            .timer(),
        );
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
fn jump(
    mut players: Query<(
        Entity,
        &mut Velocity,
        &Transform,
        &mut Stamina,
        &mut JumpTimer,
    )>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut velocity, transform, mut stamina, mut jump_timer) in players.iter_mut() {
        if jump_timer.checked_sub_mut(time.delta())
            && stamina.checked_sub_mut(jump_timer.stamina_cost * time.delta_secs())
        {
            let speed = velocity.linvel.length_projected_onto(transform.up());
            velocity.linvel += transform.up() * (jump_timer.speed - speed).max(0.0);
        } else {
            commands.entity(entity).remove::<Jumping>();
        }
    }
}
