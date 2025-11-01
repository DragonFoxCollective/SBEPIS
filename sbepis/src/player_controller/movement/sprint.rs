use bevy::prelude::*;
use bevy_pretty_nice_input::Action;

#[derive(Action)]
pub struct Sprint;

#[derive(Action)]
pub struct SprintWalk;

#[derive(Action)]
pub struct UnSprintWalk;

#[derive(Component, Default)]
pub struct SprintStanding;

#[derive(Component, Default)]
pub struct Sprinting;
