use bevy::prelude::*;
use bevy_butler::*;
use bevy_rapier3d::prelude::*;

use crate::input::{button_is_pressed, button_is_released};
use crate::player_controller::movement::MovementControlSet;
use crate::player_controller::{PlayerAction, PlayerControllerPlugin};
use crate::prelude::PlayerBody;

use super::stand::Standing;

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

pub fn to_standing_assets(body: &PlayerBody, commands: &mut Commands, assets: &StandingAssets) {
	commands
		.entity(body.mesh)
		.insert((assets.mesh.clone(), assets.mesh_transform));
	commands
		.entity(body.collider)
		.insert((assets.collider.clone(), assets.collider_transform));
	commands.entity(body.camera).insert(assets.camera_transform);
}

pub fn to_crouching_assets(body: &PlayerBody, commands: &mut Commands, assets: &CrouchingAssets) {
	commands
		.entity(body.mesh)
		.insert((assets.mesh.clone(), assets.mesh_transform));
	commands
		.entity(body.collider)
		.insert((assets.collider.clone(), assets.collider_transform));
	commands.entity(body.camera).insert(assets.camera_transform);
}

#[derive(Component, Default)]
pub struct Crouching;

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_is_pressed(PlayerAction::Crouch),
	in_set = MovementControlSet::UpdateState,
)]
fn standing_to_crouching(
	players: Query<(Entity, &PlayerBody), With<Standing>>,
	assets: Res<CrouchingAssets>,
	mut commands: Commands,
) {
	for (player, body) in players.iter() {
		commands
			.entity(player)
			.remove::<Standing>()
			.insert(Crouching);
		to_crouching_assets(body, &mut commands, &assets);
	}
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_is_released(PlayerAction::Crouch),
	in_set = MovementControlSet::UpdateState,
)]
fn crouching_to_standing(
	players: Query<(Entity, &PlayerBody), With<Crouching>>,
	assets: Res<StandingAssets>,
	mut commands: Commands,
) {
	for (player, body) in players.iter() {
		commands
			.entity(player)
			.remove::<Crouching>()
			.insert(Standing);
		to_standing_assets(body, &mut commands, &assets);
	}
}
