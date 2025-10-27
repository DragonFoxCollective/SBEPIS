use std::time::Duration;

use bevy::prelude::*;
use bevy_butler::*;
use bevy_pretty_nice_input::{Action, JustPressed};
use bevy_rapier3d::prelude::*;

use crate::entity::Movement;
use crate::gravity::AffectedByGravity;
use crate::player_controller::PlayerControllerPlugin;
use crate::player_controller::movement::MovementControlSystems;
use crate::player_controller::movement::charge::{ChargingTime, PlayerChargeSettings};
use crate::player_controller::stamina::Stamina;
use crate::util::MapRangeBetween;

use super::charge::{ChargeWalking, ChargingSound};
use super::di::DirectionalInput;
use super::sprint::Sprinting;
use super::walk::{PlayerWalkSettings, Walking};

#[derive(Action)]
pub struct Dash;

#[derive(Resource)]
#[insert_resource(plugin = PlayerControllerPlugin, init = PlayerDashSettings {
	speed_addon: 12.0,
	dash_time: Duration::from_secs_f32(0.3),
	stamina_cost: 0.33,

	charge_min_speed_addon: 12.0,
	charge_max_speed_addon: 40.0,
	charge_dash_time: Duration::from_secs_f32(0.3),
	charge_min_stamina_cost: 0.33,
	charge_max_stamina_cost: 0.66,
})]
pub struct PlayerDashSettings {
    pub speed_addon: f32,
    pub dash_time: Duration,
    pub stamina_cost: f32,

    pub charge_min_speed_addon: f32,
    pub charge_max_speed_addon: f32,
    pub charge_dash_time: Duration,
    pub charge_min_stamina_cost: f32,
    pub charge_max_stamina_cost: f32,
}

#[derive(Resource)]
pub struct DashAssets {
    pub sound: Handle<AudioSource>,
}

#[add_system(plugin = PlayerControllerPlugin, schedule = Startup)]
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(DashAssets {
        sound: asset_server.load("ultrakill dash sound.mp3"),
    });
}

#[derive(Component)]
pub struct Dashing {
    pub duration: Duration,
    pub max_duration: Duration,
    pub velocity: Vec3,
    pub speed_addon: f32,
}

#[add_observer(plugin = PlayerControllerPlugin)]
fn walking_to_dashing(
    dash: On<JustPressed<Dash>>,
    mut players: Query<
        (
            &Velocity,
            &DirectionalInput,
            &mut Stamina,
            Option<&ChargingTime>,
            Option<&ChargingSound>,
        ),
        Or<(With<Walking>, With<Sprinting>, With<ChargeWalking>)>, // TODO: replace this with event input
    >,
    charge_settings: Res<PlayerChargeSettings>,
    settings: Res<PlayerDashSettings>,
    mut commands: Commands,
    assets: Res<DashAssets>,
) -> Result {
    let (velocity, di, mut stamina, charging, charging_sound) = players.get_mut(dash.input)?;

    let (min_stamina_cost, result) = if let Some(charging) = charging {
        (
            settings.charge_min_stamina_cost,
            charging
                .power_and_stamina_cost_from_stamina(
                    &charge_settings,
                    stamina.current,
                    settings.charge_min_stamina_cost,
                    settings.charge_max_stamina_cost,
                )
                .map(|(power, charge_stamina_cost)| {
                    (
                        power.map_from_01(
                            settings.charge_min_speed_addon..settings.charge_max_speed_addon,
                        ),
                        settings.charge_dash_time,
                        charge_stamina_cost,
                    )
                }),
        )
    } else {
        (
            settings.stamina_cost,
            if stamina.current > settings.stamina_cost {
                Some((
                    settings.speed_addon,
                    settings.dash_time,
                    settings.stamina_cost,
                ))
            } else {
                None
            },
        )
    };

    if let Some((speed_addon, dash_time, stamina_cost)) = result {
        debug!("Dashing!");

        stamina.current -= stamina_cost;

        commands
            .entity(dash.input)
            .insert(Dashing {
                duration: Duration::ZERO,
                max_duration: dash_time,
                velocity: di.world_space.normalize_or(di.forward)
                    * (velocity.linvel.length() + speed_addon),
                speed_addon,
            })
            .remove::<ChargeWalking>()
            .remove::<ChargingSound>()
            .remove::<AffectedByGravity>();

        commands.spawn((AudioPlayer(assets.sound.clone()), PlaybackSettings::DESPAWN));

        if let Some(charging_sound) = charging_sound {
            if let Ok(mut sound) = commands.get_entity(charging_sound.0) {
                sound.despawn();
            }

            commands.entity(dash.input).insert(Walking);
        }
    } else {
        debug!(
            "Not enough stamina to dash! Have {}, need {}",
            stamina.current, min_stamina_cost
        );
    }

    Ok(())
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSystems::UpdateState,
	before = walking_to_dashing,
)]
fn update_dashing(
    mut players: Query<(Entity, &mut Dashing, &mut Movement, &mut Velocity)>,
    time: Res<Time>,
    walk_settings: Res<PlayerWalkSettings>,
    mut commands: Commands,
) {
    for (player, mut dashing, mut movement, mut velocity) in players.iter_mut() {
        dashing.duration += time.delta();
        if dashing.duration >= dashing.max_duration {
            velocity.linvel = dashing.velocity.normalize_or_zero()
                * (dashing.velocity.length() - dashing.speed_addon
                    + (walk_settings.sprint_speed - walk_settings.speed));
            movement.0 = velocity.linvel;

            commands
                .entity(player)
                .remove::<Dashing>()
                .insert(AffectedByGravity);
        }
    }
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSystems::DoHorizontalMovement,
)]
fn update_dash_velocity(mut movement: Query<(&mut Movement, &mut Velocity, &Dashing)>) {
    for (mut movement, mut velocity, dashing) in movement.iter_mut() {
        velocity.linvel = dashing.velocity;
        movement.0 = dashing.velocity;
    }
}
