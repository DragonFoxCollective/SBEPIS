use bevy::prelude::*;
use bevy_butler::*;
use bevy_rapier3d::prelude::*;

use crate::input::{button_just_pressed, button_just_released};
use crate::player_controller::movement::MovementControlSet;
use crate::player_controller::{PlayerAction, PlayerControllerPlugin};
use crate::prelude::PlayerBody;

#[derive(Resource)]
pub struct StandingAssets {
	pub mesh: Mesh3d,
	pub mesh_transform: Transform,
	pub collider: Collider,
	pub collider_transform: Transform,
	pub camera_transform: Transform,
}

#[derive(Resource)]
pub struct CrouchingAssets {
	pub mesh: Mesh3d,
	pub mesh_transform: Transform,
	pub collider: Collider,
	pub collider_transform: Transform,
	pub camera_transform: Transform,
}

#[derive(Component, Default)]
pub struct Crouching;

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_pressed(PlayerAction::Crouch),
	in_set = MovementControlSet::UpdateCrouching,
)]
fn add_crouching(
	players: Query<(Entity, &PlayerBody)>,
	assets: Res<CrouchingAssets>,
	mut commands: Commands,
) {
	for (player, body) in players.iter() {
		commands.entity(player).insert(Crouching);
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
	in_set = MovementControlSet::UpdateCrouching,
)]
fn remove_crouching(
	players: Query<(Entity, &PlayerBody)>,
	assets: Res<StandingAssets>,
	mut commands: Commands,
) {
	for (player, body) in players.iter() {
		commands.entity(player).remove::<Crouching>();
		commands
			.entity(body.mesh)
			.insert((assets.mesh.clone(), assets.mesh_transform));
		commands
			.entity(body.collider)
			.insert((assets.collider.clone(), assets.collider_transform));
		commands.entity(body.camera).insert(assets.camera_transform);
	}
}
