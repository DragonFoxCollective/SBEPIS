use bevy::prelude::*;
use bevy_butler::*;
use bevy_pretty_nice_input::{Action, JustPressed, JustReleased};

use crate::player_controller::PlayerControllerPlugin;

use super::walk::Walking;

#[derive(Action)]
pub struct Sprint;

#[derive(Action)]
pub struct UnSprint;

#[derive(Component, Default)]
pub struct Sprinting;

#[add_observer(plugin = PlayerControllerPlugin)]
fn walking_to_sprinting(sprint: On<JustPressed<Sprint>>, mut commands: Commands) {
    commands
        .entity(sprint.input)
        .remove::<Walking>()
        .insert(Sprinting);
}

#[add_observer(plugin = PlayerControllerPlugin)]
fn sprinting_to_walking(sprint: On<JustReleased<UnSprint>>, mut commands: Commands) {
    commands
        .entity(sprint.input)
        .remove::<Sprinting>()
        .insert(Walking);
}
