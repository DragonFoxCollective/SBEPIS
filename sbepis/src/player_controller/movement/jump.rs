use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_pretty_nice_input::bevy_event_chain::*;
use bevy_pretty_nice_input::bundles::observe;
use bevy_pretty_nice_input::{Action, Condition, ConditionedBindingUpdate, JustPressed};
use bevy_rapier3d::prelude::*;

use crate::gravity::AffectedByGravity;
use crate::player_controller::PlayerControllerPlugin;
use crate::player_controller::movement::charge::{Charging, PlayerChargeSettings};
use crate::player_controller::stamina::Stamina;
use crate::util::MapRangeBetween;

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

#[auto_event(plugin = PlayerControllerPlugin, target(entity), derive, reflect, register)]
pub struct DoJump {
    pub entity: Entity,
    pub speed: f32,
    pub stamina_cost: f32,
}

#[auto_resource(plugin = PlayerControllerPlugin, derive, reflect, register, init)]
pub struct PlayerJumpSettings {
    pub jump_speed: f32,
    pub jump_stamina_cost: f32,

    pub high_jump_speed: f32,
    pub high_jump_stamina_cost: f32,

    pub charge_jump_min_speed: f32,
    pub charge_jump_max_speed: f32,
    pub charge_jump_min_stamina_cost: f32,
    pub charge_jump_max_stamina_cost: f32,

    pub unreal_air_jump_min_speed: f32,
    pub unreal_air_jump_max_speed: f32,
    pub unreal_air_jump_min_stamina_cost: f32,
    pub unreal_air_jump_max_stamina_cost: f32,
}

impl Default for PlayerJumpSettings {
    fn default() -> Self {
        Self {
            jump_speed: 5.0,
            jump_stamina_cost: 0.0,

            high_jump_speed: 7.0,
            high_jump_stamina_cost: 0.0,

            charge_jump_min_speed: 5.0,
            charge_jump_max_speed: 10.0,
            charge_jump_min_stamina_cost: 0.0,
            charge_jump_max_stamina_cost: 0.33,

            unreal_air_jump_min_speed: 7.0,
            unreal_air_jump_max_speed: 15.0,
            unreal_air_jump_min_stamina_cost: 0.0,
            unreal_air_jump_max_stamina_cost: 0.66,
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
fn jump(jump: On<JustPressed<Jump>>, mut commands: Commands, settings: Res<PlayerJumpSettings>) {
    commands.trigger(DoJump {
        entity: jump.input,
        speed: settings.jump_speed,
        stamina_cost: settings.jump_stamina_cost,
    });
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn crouch_jump(
    jump: On<JustPressed<CrouchJump>>,
    mut commands: Commands,
    settings: Res<PlayerJumpSettings>,
) {
    commands.trigger(DoJump {
        entity: jump.input,
        speed: settings.high_jump_speed,
        stamina_cost: settings.high_jump_stamina_cost,
    });
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn slide_jump(
    jump: On<JustPressed<SlideJump>>,
    mut commands: Commands,
    settings: Res<PlayerJumpSettings>,
) {
    commands.trigger(DoJump {
        entity: jump.input,
        speed: settings.high_jump_speed,
        stamina_cost: settings.high_jump_stamina_cost,
    });
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn charge_jump(
    jump: On<JustPressed<ChargeJump>>,
    mut commands: Commands,
    players: Query<(&Stamina, &Charging)>,
    charge_settings: Res<PlayerChargeSettings>,
    settings: Res<PlayerJumpSettings>,
    assets: Res<JumpAssets>,
) -> Result {
    let (stamina, charging) = players.get(jump.input)?;
    let (power, stamina_cost) = charging
        .power_and_stamina_cost_from_stamina(
            &charge_settings,
            stamina.current,
            settings.charge_jump_min_stamina_cost,
            settings.charge_jump_max_stamina_cost,
        )
        .ok_or(BevyError::from(
            "Don't have enough stamina to charge jump, despite being in jump transition",
        ))?;
    commands.trigger(DoJump {
        entity: jump.input,
        speed: power.map_from_01(settings.charge_jump_min_speed..settings.charge_jump_max_speed),
        stamina_cost,
    });

    commands.spawn((
        AudioPlayer(assets.charge_jump_sound.clone()),
        PlaybackSettings::DESPAWN,
    ));

    Ok(())
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn charge_crouch_jump(
    jump: On<JustPressed<ChargeCrouchJump>>,
    mut commands: Commands,
    players: Query<(&Stamina, &Charging)>,
    charge_settings: Res<PlayerChargeSettings>,
    settings: Res<PlayerJumpSettings>,
    assets: Res<JumpAssets>,
) -> Result {
    let (stamina, charging) = players.get(jump.input)?;
    let (power, stamina_cost) = charging
        .power_and_stamina_cost_from_stamina(
            &charge_settings,
            stamina.current,
            settings.unreal_air_jump_min_stamina_cost,
            settings.unreal_air_jump_max_stamina_cost,
        )
        .ok_or(BevyError::from(
            "Don't have enough stamina to unreal air, despite being in jump transition",
        ))?;
    commands.trigger(DoJump {
        entity: jump.input,
        speed: power
            .map_from_01(settings.unreal_air_jump_min_speed..settings.unreal_air_jump_max_speed),
        stamina_cost,
    });

    commands.spawn((
        AudioPlayer(assets.charge_jump_sound.clone()),
        PlaybackSettings::DESPAWN,
    ));

    Ok(())
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn do_jump(
    jump: On<DoJump>,
    mut player_bodies: Query<(&mut Velocity, &Transform, &mut Stamina)>,
    mut commands: Commands,
) -> Result {
    let (mut velocity, transform, mut stamina) = player_bodies.get_mut(jump.entity)?;

    stamina.current -= jump.stamina_cost;

    if transform.up().dot(velocity.linvel) < 0.0 {
        velocity.linvel = velocity.linvel.reject_from(transform.up().into());
    }
    velocity.linvel += transform.up() * jump.speed;

    commands
        .entity(jump.entity)
        .remove::<Dashing>()
        .insert(AffectedByGravity);

    Ok(())
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
                if update.data.is_zero() || stamina.current >= settings.jump_stamina_cost {
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
                if update.data.is_zero() || stamina.current >= settings.high_jump_stamina_cost {
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
                if update.data.is_zero() || stamina.current >= settings.high_jump_stamina_cost {
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
                if update.data.is_zero() || stamina.current >= settings.charge_jump_min_stamina_cost
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
                    || stamina.current >= settings.unreal_air_jump_min_stamina_cost
                {
                    update.trigger_next(&mut commands);
                }
                Ok(())
            },
        )
    }
}
