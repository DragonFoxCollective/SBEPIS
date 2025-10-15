use bevy::prelude::*;
use bevy_butler::*;
use bevy_pretty_nice_input::{Action, JustPressed, JustReleased};

use crate::player_controller::PlayerControllerPlugin;
use crate::player_controller::movement::crouch::Crouch;

use super::crouch::Crouching;
use super::walk::Walking;

#[derive(Action)]
pub struct Sneak;

#[derive(Component, Default)]
pub struct Sneaking;

// TODO: Remove these somehow... state machine IN THE INPUT SYSTEM???

#[add_observer(plugin = PlayerControllerPlugin)]
fn crouching_to_sneaking(sneak: On<JustPressed<Sneak>>, mut commands: Commands) {
    commands
        .entity(sneak.input)
        .remove::<Crouching>()
        .insert(Sneaking);
}

#[add_observer(plugin = PlayerControllerPlugin)]
fn sneaking_to_crouching(sneak: On<JustReleased<Sneak>>, mut commands: Commands) {
    commands
        .entity(sneak.input)
        .remove::<Sneaking>()
        .insert(Crouching);
}

#[add_observer(plugin = PlayerControllerPlugin)]
fn sneaking_to_walking(crouch: On<JustReleased<Crouch>>, mut commands: Commands) {
    commands
        .entity(crouch.input)
        .remove::<Sneaking>()
        .insert(Walking);
}
