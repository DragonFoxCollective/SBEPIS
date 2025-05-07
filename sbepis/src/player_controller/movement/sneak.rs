use bevy::prelude::*;
use bevy_butler::*;

use crate::input::{button_just_pressed, button_just_released};
use crate::player_controller::movement::MovementControlSet;
use crate::player_controller::{PlayerAction, PlayerControllerPlugin};
use crate::prelude::PlayerBody;

use super::crouch::{Crouching, StandingAssets, to_standing_assets};
use super::walk::Walking;

#[derive(Component, Default)]
pub struct Sneaking;

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_pressed(PlayerAction::Move),
	in_set = MovementControlSet::UpdateState,
)]
fn crouching_to_sneaking(
	players: Query<Entity, (With<PlayerBody>, With<Crouching>)>,
	mut commands: Commands,
) {
	for player in players.iter() {
		commands
			.entity(player)
			.remove::<Crouching>()
			.insert(Sneaking);
	}
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_released(PlayerAction::Move),
	in_set = MovementControlSet::UpdateState,
)]
fn sneaking_to_crouching(
	players: Query<Entity, (With<PlayerBody>, With<Sneaking>)>,
	mut commands: Commands,
) {
	for player in players.iter() {
		commands
			.entity(player)
			.remove::<Sneaking>()
			.insert(Crouching);
	}
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_released(PlayerAction::Crouch),
	in_set = MovementControlSet::UpdateState,
)]
fn sneaking_to_walking(
	players: Query<(Entity, &PlayerBody), With<Sneaking>>,
	mut commands: Commands,
	assets: Res<StandingAssets>,
) {
	for (player, body) in players.iter() {
		commands.entity(player).remove::<Sneaking>().insert(Walking);
		to_standing_assets(body, &mut commands, &assets);
	}
}
