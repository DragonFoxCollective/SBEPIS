use bevy::prelude::*;
use bevy_butler::*;

use crate::input::{button_just_pressed, button_just_released};
use crate::player_controller::movement::MovementControlSet;
use crate::player_controller::{PlayerAction, PlayerControllerPlugin};
use crate::prelude::PlayerBody;

use super::crouch::{CrouchingAssets, StandingAssets};
use super::walk::Walking;

#[derive(Resource)]
#[resource(plugin = PlayerControllerPlugin, init = PlayerSlideSettings {
	slide_speed_cap: 10.0,
	slide_friction: 1.0,
	slide_forward_friction: 0.1,
	slide_break_friction: 10.0,
	slide_turn_radius: 0.5,
	slide_turn_friction: 1.0,
})]
pub struct PlayerSlideSettings {
	pub slide_speed_cap: f32,
	pub slide_friction: f32,
	pub slide_forward_friction: f32,
	pub slide_break_friction: f32,
	pub slide_turn_radius: f32,
	pub slide_turn_friction: f32,
}

#[derive(Component, Default)]
pub struct Sliding;

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
		commands.entity(player).remove::<Walking>().insert(Sliding);
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
