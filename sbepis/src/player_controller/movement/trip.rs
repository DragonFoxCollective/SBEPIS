use std::time::Duration;

use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_pretty_nice_input::{Action, JustPressed};
use bevy_rapier3d::prelude::Velocity;

use crate::entity::Movement;
use crate::gravity::{AffectedByGravity, ComputedGravity};
use crate::player_controller::PlayerControllerPlugin;
use crate::player_controller::movement::MovementControlSystems;
use crate::player_controller::stamina::Stamina;

use super::dash::Dashing;
use super::grounded::Grounded;
use super::slide::Sliding;
use super::sprint::Sprinting;
use super::stand::Standing;
use super::walk::Walking;

#[derive(Action)]
#[action(invalidate = false)]
pub struct Trip;

#[derive(Action)]
pub struct GroundParry;

#[auto_resource(plugin = PlayerControllerPlugin, derive, reflect, register, init)]
pub struct PlayerTripSettings {
    pub upward_speed: f32,
    pub stun_time: Duration,
    pub ground_parry_speed: f32,
    pub trip_speed_threshold: f32,
    pub ground_parry_stamina_gain: f32,
}

impl Default for PlayerTripSettings {
    fn default() -> Self {
        Self {
            upward_speed: 5.0,
            stun_time: Duration::from_secs_f32(1.0),
            ground_parry_speed: 40.0,
            trip_speed_threshold: 25.0,
            ground_parry_stamina_gain: 0.25,
        }
    }
}

/// Marker component to insert the real Tripping component
#[auto_component(plugin = PlayerControllerPlugin, derive(Default), reflect, register)]
pub struct StartTripping;

#[auto_component(plugin = PlayerControllerPlugin, derive, reflect, register)]
pub struct Tripping {
    pub duration: Duration,
    pub up: Vec3,
    pub velocity: Vec3,
}

impl Tripping {
    pub fn new(up: Vec3, velocity: Vec3) -> Self {
        Self {
            duration: Duration::ZERO,
            up,
            velocity,
        }
    }
}

#[auto_component(plugin = PlayerControllerPlugin, derive, reflect, register)]
pub struct TripRecover;

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
                .insert(TripRecover)
                .insert(AffectedByGravity);
        }
    }
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Update, config(
	in_set = MovementControlSystems::DoHorizontalMovement,
))]
fn update_tripping_velocity(
    mut movement: Query<(&mut Movement, &mut Velocity, &mut Tripping)>,
    time: Res<Time>,
    settings: Res<PlayerTripSettings>,
) {
    for (mut movement, mut velocity, mut tripping) in movement.iter_mut() {
        let acceleration = 2.0 * settings.upward_speed / settings.stun_time.as_secs_f32();
        let new_velocity = tripping.velocity - tripping.up * acceleration * time.delta_secs();

        tripping.velocity = new_velocity;
        velocity.linvel = new_velocity;
        movement.0 = new_velocity;
    }
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn ground_parry(
    parry: On<JustPressed<GroundParry>>,
    mut players: Query<(&mut Movement, &Transform, &mut Stamina, &mut Velocity)>,
    mut commands: Commands,
    trip_settings: Res<PlayerTripSettings>,
    parry_sound: Res<ParrySound>,
    hit_stop_settings: Res<HitStopSettings>,
) -> Result {
    debug!("GROUND PARRY!!!!!");

    let (mut movement, transform, mut stamina, mut velocity) = players.get_mut(parry.input)?;

    stamina.current += trip_settings.ground_parry_stamina_gain;

    commands.entity(parry.input).insert(Sliding::default());

    let ground_parry_velocity = transform.rotation * -Vec3::Z * trip_settings.ground_parry_speed;
    movement.0 += ground_parry_velocity;
    velocity.linvel += ground_parry_velocity;

    commands.spawn((
        Name::new("Parry Sound"),
        AudioPlayer::new(parry_sound.0.clone()),
    ));

    commands.entity(parry.input).insert(HitStop {
        duration: hit_stop_settings.ground_parry_duration,
        velocity: velocity.linvel,
        movement: movement.0,
    });

    Ok(())
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Update, config(
	in_set = MovementControlSystems::UpdateState,
))]
fn walking_too_fast_to_tripping(
    mut players: Query<
        (Entity, &ComputedGravity, &Velocity, &Transform),
        (
            Or<(With<Walking>, With<Standing>, With<Sprinting>)>,
            Without<Dashing>,
            With<Grounded>,
        ),
    >,
    mut commands: Commands,
    trip_settings: Res<PlayerTripSettings>,
) {
    for (player, gravity, velocity, transform) in players.iter_mut() {
        if (transform.rotation.inverse() * velocity.linvel)
            .xz()
            .length()
            < trip_settings.trip_speed_threshold
        {
            continue;
        }

        debug!("Too fast! :(");

        commands
            .entity(player)
            .remove::<Walking>()
            .remove::<Standing>()
            .remove::<Sprinting>()
            .remove::<AffectedByGravity>()
            .insert(Tripping::new(
                gravity.up,
                gravity.up * trip_settings.upward_speed,
            ));
    }
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn start_tripping(
    add: On<Add, StartTripping>,
    mut players: Query<(Entity, &ComputedGravity)>,
    mut commands: Commands,
    settings: Res<PlayerTripSettings>,
) -> Result {
    let (player, gravity) = players.get_mut(add.entity)?;
    commands
        .entity(player)
        .remove::<StartTripping>()
        .insert(Tripping::new(
            gravity.up,
            gravity.up * settings.upward_speed,
        ));
    Ok(())
}

#[auto_resource(plugin = PlayerControllerPlugin, derive, reflect, register)]
pub struct ParrySound(pub Handle<AudioSource>);

#[auto_system(plugin = PlayerControllerPlugin, schedule = Startup)]
fn setup_global(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(ParrySound(asset_server.load("parry-ultrakill.mp3")));
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
