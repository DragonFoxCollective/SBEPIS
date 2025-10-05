use std::time::Duration;

use bevy::prelude::*;
use bevy_butler::*;
use bevy_rapier3d::prelude::Velocity;
use leafwing_input_manager::prelude::ActionState;
use return_ok::ok_or_return;

use crate::entity::Movement;
use crate::gravity::{AffectedByGravity, ComputedGravity};
use crate::input::{button_just_pressed, button_pressed};
use crate::player_controller::movement::MovementControlSet;
use crate::player_controller::stamina::Stamina;
use crate::player_controller::{PlayerAction, PlayerControllerPlugin};

use super::CoyoteTimeSettings;
use super::dash::Dashing;
use super::grounded::Grounded;
use super::slide::Sliding;
use super::sprint::Sprinting;
use super::stand::Standing;
use super::walk::Walking;

#[derive(Resource)]
#[insert_resource(plugin = PlayerControllerPlugin, init = PlayerTripSettings {
	upward_speed: 5.0,
	stun_time: Duration::from_secs_f32(1.0),
	ground_parry_speed: 40.0,
	trip_speed_threshold: 25.0,
	ground_parry_stamina_gain: 0.25,
})]
pub struct PlayerTripSettings {
    pub upward_speed: f32,
    pub stun_time: Duration,
    pub ground_parry_speed: f32,
    pub trip_speed_threshold: f32,
    pub ground_parry_stamina_gain: f32,
}

#[derive(Component)]
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

#[derive(Component, Default)]
pub struct TripRecoverInAir;

/// Coyote time
#[derive(Component, Default)]
pub struct TripRecoverOnGround {
    pub duration: Duration,
}

/// Input buffer
#[derive(Component, Default)]
pub struct TryingToGroundParry {
    pub duration: Duration,
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSet::UpdateState,
)]
fn tripping_to_trip_recover_air(
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
                .insert(TripRecoverInAir)
                .insert(AffectedByGravity);
        }
    }
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSet::DoHorizontalMovement,
)]
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

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSet::UpdateState,
)]
fn trip_recover_air_to_trip_recover_ground(
    mut players: Query<Entity, (With<Grounded>, With<TripRecoverInAir>)>,
    mut commands: Commands,
) {
    for player in players.iter_mut() {
        commands
            .entity(player)
            .remove::<TripRecoverInAir>()
            .insert(TripRecoverOnGround::default());
    }
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSet::UpdateState,
	before = trip_recover_air_to_trip_recover_ground,
)]
fn update_trip_recover_ground(
    mut players: Query<(Entity, &mut TripRecoverOnGround)>,
    time: Res<Time>,
    coyote_time_settings: Res<CoyoteTimeSettings>,
    mut commands: Commands,
    input: Query<&ActionState<PlayerAction>>,
) {
    let input = ok_or_return!(input.single());
    for (player, mut trip_recover_ground) in players.iter_mut() {
        trip_recover_ground.duration += time.delta();
        if trip_recover_ground.duration >= coyote_time_settings.coyote_time {
            commands.entity(player).remove::<TripRecoverOnGround>();

            if button_pressed(input, &PlayerAction::Move) {
                commands.entity(player).insert(Walking);
            } else {
                commands.entity(player).insert(Standing);
            }
        }
    }
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_pressed(PlayerAction::Crouch),
	in_set = MovementControlSet::UpdateState,
)]
pub fn add_trying_to_ground_parry(
    players: Query<
        Entity,
        Or<(
            With<Tripping>,
            With<TripRecoverOnGround>,
            With<TripRecoverInAir>,
        )>,
    >,
    mut commands: Commands,
) {
    for player in players.iter() {
        commands
            .entity(player)
            .insert(TryingToGroundParry::default());
    }
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSet::UpdateState,
	before = add_trying_to_ground_parry,
)]
fn update_trying_to_ground_parry(
    mut players: Query<(Entity, &mut TryingToGroundParry)>,
    time: Res<Time>,
    coyote_time_settings: Res<CoyoteTimeSettings>,
    mut commands: Commands,
) {
    for (player, mut trying_to_ground_parry) in players.iter_mut() {
        trying_to_ground_parry.duration += time.delta();
        debug!(
            "Trying to ground parry: {:.2?} / {:.2?}",
            trying_to_ground_parry.duration.as_secs_f32(),
            coyote_time_settings.input_buffer_time.as_secs_f32()
        );
        if trying_to_ground_parry.duration >= coyote_time_settings.input_buffer_time {
            commands.entity(player).remove::<TryingToGroundParry>();
        }
    }
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSet::UpdateState,
	after = add_trying_to_ground_parry,
	before = update_trip_recover_ground,
)]
fn ground_parry(
    mut players: Query<
        (
            Entity,
            &mut Movement,
            &Transform,
            &mut Stamina,
            &mut Velocity,
        ),
        (With<TryingToGroundParry>, With<TripRecoverOnGround>),
    >,
    mut commands: Commands,
    trip_settings: Res<PlayerTripSettings>,
    parry_sound: Res<ParrySound>,
) {
    for (player, mut movement, transform, mut stamina, mut velocity) in players.iter_mut() {
        debug!("GROUND PARRY!!!!!");

        stamina.current += trip_settings.ground_parry_stamina_gain;

        commands
            .entity(player)
            .remove::<TryingToGroundParry>()
            .remove::<TripRecoverOnGround>()
            .insert(Sliding::default());

        let ground_parry_velocity =
            transform.rotation * -Vec3::Z * trip_settings.ground_parry_speed;
        movement.0 += ground_parry_velocity;
        velocity.linvel += ground_parry_velocity;

        commands.spawn((
            Name::new("Parry Sound"),
            AudioPlayer::new(parry_sound.0.clone()),
        ));
    }
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSet::UpdateState,
)]
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

#[derive(Resource)]
pub struct ParrySound(pub Handle<AudioSource>);

#[add_system(plugin = PlayerControllerPlugin, schedule = Startup)]
fn setup_global(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(ParrySound(asset_server.load("parry-ultrakill.mp3")));
}
