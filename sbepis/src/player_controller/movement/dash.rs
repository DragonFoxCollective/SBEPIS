use std::time::Duration;

use bevy::prelude::*;
use bevy_butler::*;
use bevy_rapier3d::prelude::*;

use crate::entity::Movement;
use crate::gravity::AffectedByGravity;
use crate::input::button_just_pressed;
use crate::player_controller::movement::MovementControlSet;
use crate::player_controller::movement::charge::charge_walking_to_trying_to_dash;
use crate::player_controller::stamina::Stamina;
use crate::player_controller::{PlayerAction, PlayerControllerPlugin};
use crate::prelude::PlayerBody;
use crate::util::MapRangeBetween;

use super::CoyoteTimeSettings;
use super::charge::{ChargeWalking, ChargingSound};
use super::di::DirectionalInput;
use super::grounded::EffectiveGrounded;
use super::sprint::Sprinting;
use super::walk::{PlayerWalkSettings, Walking};

#[derive(Resource)]
#[insert_resource(plugin = PlayerControllerPlugin, init = PlayerDashSettings {
	speed_addon: 12.0,
	dash_time: Duration::from_secs_f32(0.3),
	cooldown: Duration::from_secs_f32(0.2),
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
    pub cooldown: Duration,
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

#[derive(Component, Default)]
pub struct TryingToDash(Duration);

#[derive(Component)]
pub struct Dashing {
    pub duration: Duration,
    pub max_duration: Duration,
    pub velocity: Vec3,
    pub speed_addon: f32,
}

#[derive(Component, Default)]
pub struct DashCooldown(Duration);

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_pressed(PlayerAction::Sprint),
	in_set = MovementControlSet::UpdateState,
)]
pub fn add_trying_to_dash(players: Query<Entity, With<PlayerBody>>, mut commands: Commands) {
    for player in players.iter() {
        commands.entity(player).insert(TryingToDash::default());
    }
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSet::UpdateState,
	before = add_trying_to_dash,
)]
fn update_trying_to_dash(
    mut players: Query<(Entity, &mut TryingToDash)>,
    time: Res<Time>,
    coyote_time_settings: Res<CoyoteTimeSettings>,
    mut commands: Commands,
) {
    for (player, mut trying_to_dash) in players.iter_mut() {
        trying_to_dash.0 += time.delta();
        debug!("Trying to dash: {:.2?}", trying_to_dash.0.as_secs_f32());
        if trying_to_dash.0 >= coyote_time_settings.input_buffer_time {
            commands.entity(player).remove::<TryingToDash>();
        }
    }
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	before = update_trying_to_dash,
)]
fn update_dash_cooldown(
    mut players: Query<(Entity, &mut DashCooldown)>,
    time: Res<Time>,
    dash_settings: Res<PlayerDashSettings>,
    mut commands: Commands,
) {
    for (player, mut dash_cooldown) in players.iter_mut() {
        dash_cooldown.0 += time.delta();
        if dash_cooldown.0 >= dash_settings.cooldown {
            commands.entity(player).remove::<DashCooldown>();
        }
    }
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	after = MovementControlSet::UpdateDi,
	after = MovementControlSet::UpdateGrounded,
	after = add_trying_to_dash,
	after = charge_walking_to_trying_to_dash,
	in_set = MovementControlSet::UpdateState,
)]
fn walking_to_dashing(
    mut players: Query<
        (
            Entity,
            &Velocity,
            &DirectionalInput,
            &mut Stamina,
            Option<&ChargeWalking>,
            Option<&ChargingSound>,
            &mut AffectedByGravity,
        ),
        (
            With<EffectiveGrounded>,
            With<TryingToDash>,
            Or<(With<Walking>, With<Sprinting>, With<ChargeWalking>)>,
            Without<Dashing>,
            Without<DashCooldown>,
        ),
    >,
    settings: Res<PlayerDashSettings>,
    mut commands: Commands,
    assets: Res<DashAssets>,
) {
    for (player, velocity, di, mut stamina, charging, charging_sound, mut gravity) in
        players.iter_mut()
    {
        let (min_stamina_cost, result) = if let Some(charging) = charging {
            (
                settings.charge_min_stamina_cost,
                charging
                    .power_and_stamina_cost_from_stamina(
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
                .entity(player)
                .insert(Dashing {
                    duration: Duration::ZERO,
                    max_duration: dash_time,
                    velocity: di.world_space.normalize_or(di.forward)
                        * (velocity.linvel.length() + speed_addon),
                    speed_addon,
                })
                .remove::<TryingToDash>()
                .remove::<ChargeWalking>()
                .remove::<ChargingSound>();

            commands.spawn((AudioPlayer(assets.sound.clone()), PlaybackSettings::DESPAWN));

            gravity.factor = 0.0;

            if let Some(charging_sound) = charging_sound {
                if let Ok(mut sound) = commands.get_entity(charging_sound.0) {
                    sound.despawn();
                }

                commands.entity(player).insert(Walking);
            }
        } else {
            debug!(
                "Not enough stamina to dash! Have {}, need {}",
                stamina.current, min_stamina_cost
            );
        }
    }
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSet::UpdateState,
	before = walking_to_dashing,
)]
fn update_dashing(
    mut players: Query<(
        Entity,
        &mut Dashing,
        &mut Movement,
        &mut Velocity,
        &mut AffectedByGravity,
    )>,
    time: Res<Time>,
    walk_settings: Res<PlayerWalkSettings>,
    mut commands: Commands,
) {
    for (player, mut dashing, mut movement, mut velocity, mut gravity) in players.iter_mut() {
        dashing.duration += time.delta();
        if dashing.duration >= dashing.max_duration {
            velocity.linvel = dashing.velocity.normalize_or_zero()
                * (dashing.velocity.length() - dashing.speed_addon
                    + (walk_settings.sprint_speed - walk_settings.speed));
            movement.0 = velocity.linvel;

            commands
                .entity(player)
                .remove::<Dashing>()
                .insert(DashCooldown::default());

            gravity.factor = 1.0;
        }
    }
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSet::DoHorizontalMovement,
)]
fn update_dash_velocity(mut movement: Query<(&mut Movement, &mut Velocity, &Dashing)>) {
    for (mut movement, mut velocity, dashing) in movement.iter_mut() {
        velocity.linvel = dashing.velocity;
        movement.0 = dashing.velocity;
    }
}
