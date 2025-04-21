use std::time::Duration;

use bevy::prelude::*;
use bevy_butler::*;
use bevy_rapier3d::prelude::Velocity;

use crate::entity::Movement;
use crate::gravity::AffectedByGravity;
use crate::input::button_just_pressed;
use crate::player_controller::movement::MovementControlSet;
use crate::player_controller::{PlayerAction, PlayerControllerPlugin};
use crate::prelude::PlayerBody;

use super::CoyoteTimeSettings;
use super::crouch::{StandingAssets, to_standing_assets};
use super::grounded::Grounded;
use super::slide::{SlideAssets, Sliding};
use super::stand::Standing;

#[derive(Resource)]
#[resource(plugin = PlayerControllerPlugin, init = PlayerTripSettings {
	upward_speed: 5.0,
	stun_time: Duration::from_secs_f32(1.0),
	ground_parry_speed: 40.0,
})]
pub struct PlayerTripSettings {
	pub upward_speed: f32,
	pub stun_time: Duration,
	pub ground_parry_speed: f32,
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

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSet::UpdateState,
)]
fn tripping_to_trip_recover_air(
	mut players: Query<(Entity, &mut Tripping, &mut AffectedByGravity)>,
	time: Res<Time>,
	settings: Res<PlayerTripSettings>,
	mut commands: Commands,
) {
	for (player, mut tripping, mut gravity) in players.iter_mut() {
		tripping.duration += time.delta();
		if tripping.duration >= settings.stun_time {
			commands
				.entity(player)
				.remove::<Tripping>()
				.insert(TripRecoverInAir);

			gravity.factor = 1.0;
		}
	}
}

#[system(
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

#[system(
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

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSet::UpdateState,
	before = trip_recover_air_to_trip_recover_ground,
)]
fn update_trip_recover_ground(
	mut players: Query<(Entity, &PlayerBody, &mut TripRecoverOnGround)>,
	time: Res<Time>,
	coyote_time_settings: Res<CoyoteTimeSettings>,
	mut commands: Commands,
	assets: Res<StandingAssets>,
) {
	for (player, body, mut trip_recover_ground) in players.iter_mut() {
		trip_recover_ground.duration += time.delta();
		if trip_recover_ground.duration >= coyote_time_settings.coyote_time {
			commands
				.entity(player)
				.remove::<TripRecoverOnGround>()
				.insert(Standing);

			to_standing_assets(body, &mut commands, &assets);
		}
	}
}

#[system(
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

#[system(
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
			"Trying to ground parry: {:.2?}",
			trying_to_ground_parry.duration.as_secs_f32()
		);
		if trying_to_ground_parry.duration >= coyote_time_settings.input_buffer_time {
			commands.entity(player).remove::<TryingToGroundParry>();
		}
	}
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSet::UpdateState,
	after = add_trying_to_ground_parry,
	before = update_trip_recover_ground,
)]
fn ground_parry(
	players: Query<Entity, (With<TryingToGroundParry>, With<TripRecoverOnGround>)>,
	mut commands: Commands,
	slide_assets: Res<SlideAssets>,
) {
	for player in players.iter() {
		debug!("GROUND PARRY!!!!!");

		let sound = commands
			.spawn((
				AudioPlayer::new(slide_assets.sound.clone()),
				PlaybackSettings::LOOP,
			))
			.id();
		commands
			.entity(player)
			.remove::<TryingToGroundParry>()
			.remove::<TripRecoverOnGround>()
			.insert(Sliding {
				current_friction: 0.0,
				sound,
			});
	}
}
