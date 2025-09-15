use bevy::prelude::*;
use bevy_butler::*;

use crate::input::{button_just_pressed, button_just_released};
use crate::player_controller::movement::MovementControlSet;
use crate::player_controller::{PlayerAction, PlayerControllerPlugin};
use crate::prelude::PlayerBody;

use super::walk::Walking;

#[derive(Component, Default)]
pub struct Sprinting;

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSet::UpdateState,
	run_if = button_just_pressed(PlayerAction::Sprint),
)]
fn walking_to_sprinting(
    players: Query<Entity, (With<PlayerBody>, With<Walking>)>,
    mut commands: Commands,
) {
    for player in players.iter() {
        commands
            .entity(player)
            .remove::<Walking>()
            .insert(Sprinting);
    }
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSet::UpdateState,
	run_if = button_just_released(PlayerAction::Sprint),
)]
fn sprinting_to_walking(
    players: Query<Entity, (With<PlayerBody>, With<Sprinting>)>,
    mut commands: Commands,
) {
    for player in players.iter() {
        commands
            .entity(player)
            .remove::<Sprinting>()
            .insert(Walking);
    }
}
