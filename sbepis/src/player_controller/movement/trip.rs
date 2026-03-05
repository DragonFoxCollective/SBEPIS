use std::time::Duration;

use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_pretty_nice_input::prelude::*;
use bevy_rapier3d::prelude::Velocity;

use crate::entity::Movement;
use crate::gravity::ComputedGravity;
use crate::player_controller::PlayerControllerPlugin;
use crate::player_controller::camera_controls::{InterpolateFov, InterpolateFovCurve, PlayerFov};
use crate::player_controller::movement::MovementControlSystems;
use crate::player_controller::movement::jump::PlayerJumpSettings;
use crate::player_controller::stamina::Stamina;

use super::dash::Dashing;
use super::grounded::Grounded;
use super::slide::Sliding;
use super::stand::Standing;
use super::walk::Sprinting;

#[derive(Action)]
#[action(invalidate = false)]
pub struct Trip;

#[derive(Action)]
pub struct GroundParry;

#[auto_resource(plugin = PlayerControllerPlugin, derive, reflect, register, init)]
pub struct PlayerTripSettings {
    pub upward_speed: f32,
    pub hori_to_vert_momentum_redirection_percentage: f32,
    pub stun_time: Duration,
    pub recover_time: Duration,
    pub ground_parry_speed: f32,
    pub trip_speed_threshold: f32,
    pub ground_parry_stamina_gain: f32,

    pub fov_trip_factor: f32,
    pub fov_trip_ease_duration_secs: f32,
    pub fov_recover_factor: f32,
    pub fov_recover_ease_duration_secs: f32,
}

impl Default for PlayerTripSettings {
    fn default() -> Self {
        Self {
            upward_speed: 5.0,
            hori_to_vert_momentum_redirection_percentage: 0.2,
            stun_time: Duration::from_secs_f32(1.0),
            recover_time: Duration::from_secs_f32(0.2),
            ground_parry_speed: 40.0,
            trip_speed_threshold: 25.0,
            ground_parry_stamina_gain: 0.25,

            fov_trip_factor: 0.5,
            fov_trip_ease_duration_secs: 0.2,
            fov_recover_factor: 1.2,
            fov_recover_ease_duration_secs: 1.0,
        }
    }
}

#[auto_component(plugin = PlayerControllerPlugin, derive(Default), reflect, register)]
pub struct Tripping {
    pub duration: Duration,
}

#[auto_component(plugin = PlayerControllerPlugin, derive(Default), reflect, register)]
pub struct TripRecover {
    pub grounded_duration: Duration,
}

#[auto_component(plugin = PlayerControllerPlugin, derive, reflect, register)]
pub struct HitStop {
    pub duration: Duration,
    pub velocity: Vec3,
    pub movement: Vec3,
}

#[auto_resource(plugin = PlayerControllerPlugin, derive, reflect, register, init)]
pub struct HitStopSettings {
    pub ground_parry_duration: Duration,
}

impl Default for HitStopSettings {
    fn default() -> Self {
        Self {
            ground_parry_duration: Duration::from_secs_f32(0.1),
        }
    }
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Update, config(
	in_set = MovementControlSystems::UpdateState,
))]
fn tripping_to_trip_recover(
    mut players: Query<(Entity, &mut Tripping)>,
    time: Res<Time>,
    settings: Res<PlayerTripSettings>,
    mut commands: Commands,
) {
    for (player, mut tripping) in players.iter_mut() {
        tripping.duration += time.delta();
        if tripping.duration >= settings.stun_time {
            commands
                .entity(player)
                .remove::<Tripping>()
                .insert(TripRecover::default());
        }
    }
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn trip_add(
    add: On<Add, Tripping>,
    mut commands: Commands,
    settings: Res<PlayerTripSettings>,
    players: Query<&PlayerFov>,
    assets: Res<TripAssets>,
) -> Result {
    let fov = players.get(add.entity)?;
    commands.entity(add.entity).insert(InterpolateFov::new(
        fov.0 * settings.fov_trip_factor,
        settings.fov_trip_ease_duration_secs,
    ));
    commands.spawn((
        Name::new("Trip Sound"),
        AudioPlayer(assets.trip_sound.clone()),
    ));
    Ok(())
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Update, config(
	in_set = MovementControlSystems::UpdateState,
))]
fn trip_recover_update(
    mut players: Query<(Entity, &mut TripRecover, Has<Grounded>, &PlayerFov)>,
    time: Res<Time>,
    settings: Res<PlayerTripSettings>,
    mut commands: Commands,
) {
    for (player, mut trip_recover, grounded, fov) in players.iter_mut() {
        if grounded {
            trip_recover.grounded_duration += time.delta();

            if trip_recover.grounded_duration > settings.recover_time {
                commands
                    .entity(player)
                    .remove::<TripRecover>()
                    .insert(Standing)
                    .insert(InterpolateFov::new(
                        fov.0,
                        settings.fov_trip_ease_duration_secs,
                    ));
            }
        } else {
            trip_recover.grounded_duration = Duration::ZERO;
        }
    }
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn ground_parry(
    parry: On<JustPressed<GroundParry>>,
    mut players: Query<(
        &mut Movement,
        &Transform,
        &mut Stamina,
        &mut Velocity,
        &PlayerFov,
    )>,
    mut commands: Commands,
    settings: Res<PlayerTripSettings>,
    assets: Res<TripAssets>,
    hit_stop_settings: Res<HitStopSettings>,
) -> Result {
    debug!("GROUND PARRY!!!!!");

    let (mut movement, transform, mut stamina, mut velocity, fov) = players.get_mut(parry.input)?;

    stamina.current += settings.ground_parry_stamina_gain;

    commands
        .entity(parry.input)
        .remove::<TripRecover>()
        .insert(Sliding);

    let ground_parry_velocity = transform.rotation * -Vec3::Z * settings.ground_parry_speed;
    movement.0 += ground_parry_velocity;
    velocity.linvel += ground_parry_velocity;

    commands.spawn((
        Name::new("Parry Sound"),
        AudioPlayer::new(assets.parry_sound.clone()),
    ));

    commands
        .entity(parry.input)
        .insert(HitStop {
            duration: hit_stop_settings.ground_parry_duration,
            velocity: velocity.linvel,
            movement: movement.0,
        })
        .insert(InterpolateFov {
            curves: vec![
                InterpolateFovCurve {
                    fov: fov.0 * settings.fov_recover_factor,
                    duration_secs: settings.fov_trip_ease_duration_secs,
                    ease: EaseFunction::CircularOut,
                },
                InterpolateFovCurve {
                    fov: fov.0,
                    duration_secs: settings.fov_recover_ease_duration_secs,
                    ease: EaseFunction::Linear,
                },
            ],
        });

    Ok(())
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Update, config(
	in_set = MovementControlSystems::UpdateState,
))]
fn walking_too_fast_to_tripping(
    mut players: Query<
        (Entity, &mut Velocity, &ComputedGravity),
        (With<Standing>, Without<Dashing>, With<Grounded>),
    >,
    mut commands: Commands,
    trip_settings: Res<PlayerTripSettings>,
    jump_settings: Res<PlayerJumpSettings>,
) {
    for (player, mut velocity, gravity) in players.iter_mut() {
        let hori_velocity = velocity.linvel.reject_from(gravity.up);
        if hori_velocity.length() < trip_settings.trip_speed_threshold {
            continue;
        }

        debug!("Too fast! :(");

        velocity.linvel = gravity.up
            * (jump_settings.jump.speed
                + hori_velocity.length()
                    * trip_settings.hori_to_vert_momentum_redirection_percentage)
            + hori_velocity * (1.0 - trip_settings.hori_to_vert_momentum_redirection_percentage);
        commands
            .entity(player)
            .remove::<Standing>()
            .remove::<Sprinting>()
            .insert(Tripping::default());
    }
}

#[auto_resource(plugin = PlayerControllerPlugin, derive, reflect, register)]
pub struct TripAssets {
    pub trip_sound: Handle<AudioSource>,
    pub parry_sound: Handle<AudioSource>,
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Startup)]
fn setup_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(TripAssets {
        trip_sound: asset_server.load("trip.mp3"),
        parry_sound: asset_server.load("parry-ultrakill.mp3"),
    });
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Update, config(
	before = MovementControlSystems::DoHorizontalMovement,
	before = MovementControlSystems::DoVerticalMovement,
	after = MovementControlSystems::UpdateState,
))]
fn hit_stop_reset(mut players: Query<(&mut Movement, &mut Velocity), With<HitStop>>) {
    for (mut movement, mut velocity) in players.iter_mut() {
        movement.0 = Vec3::ZERO;
        velocity.linvel = Vec3::ZERO;
    }
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Update, config(
	after = MovementControlSystems::DoHorizontalMovement,
	after = MovementControlSystems::DoVerticalMovement,
))]
fn hit_stop_update(
    mut players: Query<(Entity, &mut HitStop, &mut Movement, &mut Velocity)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut hit_stop, mut movement, mut velocity) in players.iter_mut() {
        if let Some(new_duration) = hit_stop.duration.checked_sub(time.delta()) {
            movement.0 = Vec3::ZERO;
            velocity.linvel = Vec3::ZERO;
            hit_stop.duration = new_duration;
        } else {
            movement.0 = hit_stop.movement;
            velocity.linvel = hit_stop.velocity;
            commands.entity(entity).remove::<HitStop>();
        }
    }
}
