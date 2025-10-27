use bevy::prelude::*;
use bevy_pretty_nice_input::Action;

#[derive(Action)]
pub struct Sprint;

#[derive(Component, Default)]
pub struct Sprinting;
