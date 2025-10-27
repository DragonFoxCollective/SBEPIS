use bevy::prelude::*;
use bevy_butler::*;
use bevy_pretty_nice_input::{Action, JustPressed};
use bevy_rapier3d::prelude::*;

use crate::gravity::AffectedByGravity;
use crate::player_controller::PlayerControllerPlugin;
use crate::player_controller::movement::charge::{ChargingTime, PlayerChargeSettings};
use crate::player_controller::movement::stand::Standing;
use crate::player_controller::movement::walk::Walking;
use crate::player_controller::stamina::Stamina;
use crate::util::MapRangeBetween;

use super::charge::{ChargeCrouching, ChargeStanding, ChargeWalking, ChargingSound};
use super::crouch::Crouching;
use super::dash::Dashing;
use super::roll::Rolling;
use super::slide::Sliding;

#[derive(Action)]
pub struct Jump;

#[derive(Resource)]
#[insert_resource(plugin = PlayerControllerPlugin, init = PlayerJumpSettings {
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
})]
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

#[derive(Resource)]
pub struct JumpAssets {
    pub charge_jump_sound: Handle<AudioSource>,
}

#[add_system(plugin = PlayerControllerPlugin, schedule = Startup)]
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(JumpAssets {
        charge_jump_sound: asset_server.load("worms bazooka shoot.mp3"),
    });
}

#[add_observer(plugin = PlayerControllerPlugin)]
fn jump(
    jump: On<JustPressed<Jump>>,
    mut player_bodies: Query<
        (
            &mut Velocity,
            &Transform,
            &mut Stamina,
            Has<Crouching>,
            Option<&ChargingTime>,
            Has<ChargeCrouching>,
            Has<ChargeWalking>,
            Option<&ChargingSound>,
        ),
        Or<(
            // TODO: replace these with event input
            With<Standing>,
            With<Walking>,
            With<Sliding>,
            With<ChargeStanding>,
            With<ChargeWalking>,
            With<ChargeCrouching>,
            With<Rolling>,
        )>,
    >,
    charge_settings: Res<PlayerChargeSettings>,
    settings: Res<PlayerJumpSettings>,
    assets: Res<JumpAssets>,
    mut commands: Commands,
) -> Result {
    let (
        mut velocity,
        transform,
        mut stamina,
        crouching,
        charging,
        charge_crouching,
        charge_walking,
        charging_sound,
    ) = player_bodies.get_mut(jump.input)?;

    let (min_stamina_cost, result) = if crouching {
        (
            settings.high_jump_stamina_cost,
            if stamina.current > settings.high_jump_stamina_cost {
                Some((settings.high_jump_speed, settings.high_jump_stamina_cost))
            } else {
                None
            },
        )
    } else if let Some(charging) = charging {
        let (min_stamina_cost, max_stamina_cost, min_speed, max_speed) = if charge_crouching {
            (
                settings.unreal_air_jump_min_stamina_cost,
                settings.unreal_air_jump_max_stamina_cost,
                settings.unreal_air_jump_min_speed,
                settings.unreal_air_jump_max_speed,
            )
        } else {
            (
                settings.charge_jump_min_stamina_cost,
                settings.charge_jump_max_stamina_cost,
                settings.charge_jump_min_speed,
                settings.charge_jump_max_speed,
            )
        };
        (
            min_stamina_cost,
            charging
                .power_and_stamina_cost_from_stamina(
                    &charge_settings,
                    stamina.current,
                    min_stamina_cost,
                    max_stamina_cost,
                )
                .map(|(power, stamina_cost)| {
                    (power.map_from_01(min_speed..max_speed), stamina_cost)
                }),
        )
    } else {
        (
            settings.jump_stamina_cost,
            if stamina.current > settings.jump_stamina_cost {
                Some((settings.jump_speed, settings.jump_stamina_cost))
            } else {
                None
            },
        )
    };

    if let Some((speed, stamina_cost)) = result {
        debug!("Jumping!");

        stamina.current -= stamina_cost;

        if transform.up().dot(velocity.linvel) < 0.0 {
            velocity.linvel = velocity.linvel.reject_from(transform.up().into());
        }
        velocity.linvel += transform.up() * speed;
    } else {
        debug!(
            "Not enough stamina to jump! Have {}, need {}",
            stamina.current, min_stamina_cost
        );
    }

    commands
        .entity(jump.input)
        .remove::<Dashing>()
        .remove::<ChargeStanding>()
        .remove::<ChargeCrouching>()
        .remove::<ChargeWalking>()
        .remove::<ChargingSound>()
        .insert(AffectedByGravity);

    if let Some(charging_sound) = charging_sound {
        commands.spawn((
            AudioPlayer(assets.charge_jump_sound.clone()),
            PlaybackSettings::DESPAWN,
        ));

        if let Ok(mut sound) = commands.get_entity(charging_sound.0) {
            sound.despawn();
        }

        if charge_crouching {
            commands.entity(jump.input).insert(Crouching);
        } else if charge_walking {
            commands.entity(jump.input).insert(Walking);
        } else {
            commands.entity(jump.input).insert(Standing);
        }
    }

    Ok(())
}
