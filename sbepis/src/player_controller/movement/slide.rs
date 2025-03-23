use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_butler::*;

use crate::entity::Movement;
use crate::entity::movement::ExecuteMovementSet;
use crate::input::{button_just_pressed, button_just_released};
use crate::player_controller::movement::MovementControlSet;
use crate::player_controller::{PlayerAction, PlayerControllerPlugin};
use crate::prelude::PlayerBody;
use crate::util::MapRange;

use super::crouch::{CrouchingAssets, StandingAssets};
use super::di::DirectionalInput;
use super::walk::Walking;

#[derive(Resource)]
#[resource(plugin = PlayerControllerPlugin, init = PlayerSlideSettings {
	speed_cap: 10.0,
	friction: 1.0,
	forward_friction: 0.1,
	brake_friction: 10.0,
	turn_factor: 0.2,
	turn_friction: 1.0,
})]
pub struct PlayerSlideSettings {
	pub speed_cap: f32,
	pub friction: f32,
	pub forward_friction: f32,
	pub brake_friction: f32,
	/// In (radians per second) / (meters per second)
	pub turn_factor: f32,
	pub turn_friction: f32,
}

#[derive(Component, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct Sliding {
	pub current_friction: f32,
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_pressed(PlayerAction::Crouch),
	in_set = MovementControlSet::UpdateState,
)]
fn walking_to_sliding(
	players: Query<(Entity, &PlayerBody), With<Walking>>,
	assets: Res<CrouchingAssets>,
	mut commands: Commands,
) {
	for (player, body) in players.iter() {
		commands
			.entity(player)
			.remove::<Walking>()
			.insert(Sliding::default());
		commands
			.entity(body.mesh)
			.insert((assets.mesh.clone(), assets.mesh_transform));
		commands
			.entity(body.collider)
			.insert((assets.collider.clone(), assets.collider_transform));
		commands.entity(body.camera).insert(assets.camera_transform);
	}
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_released(PlayerAction::Crouch),
	in_set = MovementControlSet::UpdateState,
)]
fn sliding_to_walking(
	players: Query<(Entity, &PlayerBody), With<Sliding>>,
	assets: Res<StandingAssets>,
	mut commands: Commands,
) {
	for (player, body) in players.iter() {
		commands.entity(player).remove::<Sliding>().insert(Walking);
		commands
			.entity(body.mesh)
			.insert((assets.mesh.clone(), assets.mesh_transform));
		commands
			.entity(body.collider)
			.insert((assets.collider.clone(), assets.collider_transform));
		commands.entity(body.camera).insert(assets.camera_transform);
	}
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSet::DoHorizontalMovement,
	before = ExecuteMovementSet,
)]
fn update_slide_velocity(
	mut movement: Query<(&mut Movement, &Transform, &DirectionalInput), With<Sliding>>,
	slide_settings: Res<PlayerSlideSettings>,
	time: Res<Time>,
) {
	// This is stupid, why can't I store this anywhere?
	let easing = EasingCurve::new(
		slide_settings.brake_friction,
		slide_settings.turn_friction,
		EaseFunction::CircularInOut,
	)
	.reparametrize_linear(Interval::new(0.0, PI / 2.0).unwrap())
	.unwrap()
	.chain(
		EasingCurve::new(
			slide_settings.turn_friction,
			slide_settings.forward_friction,
			EaseFunction::CircularInOut,
		)
		.reparametrize_linear(Interval::new(PI / 2.0, PI).unwrap())
		.unwrap(),
	)
	.unwrap();

	for (mut movement, transform, di) in movement.iter_mut() {
		let velocity = (transform.rotation.inverse() * movement.0).xz();

		let friction = if velocity == Vec2::ZERO || di.input == Vec2::ZERO {
			slide_settings.friction
		} else {
			let angle = di.input.angle_to(Vec2::Y).abs();
			let max_friction = easing
				.sample(angle)
				.unwrap_or_else(|| panic!("Angle out of bounds: {:?}", angle));
			di.input
				.length()
				.map_from_01(slide_settings.friction..max_friction)
		};

		let friction = -time.delta_secs()
			* friction
			* (velocity.length() - slide_settings.speed_cap).max(0.0)
			* velocity.normalize_or_zero();
		let velocity = velocity + friction;

		let turn_angle =
			slide_settings.turn_factor * velocity.length() * di.input.x * time.delta_secs();
		let velocity = Vec2::from_angle(turn_angle).rotate(velocity);

		movement.0 = transform.rotation * Vec3::new(velocity.x, 0.0, velocity.y);
	}
}
