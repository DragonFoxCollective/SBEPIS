use bevy::prelude::*;
use bevy_pretty_nice_input::Action;

#[derive(Action)]
pub struct CrouchSneak;

#[derive(Action)]
pub struct WalkSneak;

#[derive(Component, Default)]
pub struct Sneaking;
