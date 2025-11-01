use bevy::prelude::*;
use bevy_butler::*;
use bevy_pretty_nice_input::{Action, Updated};

use crate::player_controller::PlayerControllerPlugin;
use crate::player_controller::movement::di::DIUpdate;
use crate::player_controller::movement::walk::PlayerWalkSettings;

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

#[add_observer(plugin = PlayerControllerPlugin)]
fn update_di_sprintwalk(
    walk: On<Updated<SprintWalk>>,
    mut commands: Commands,
    walk_settings: Res<PlayerWalkSettings>,
) -> Result {
    commands.trigger(DIUpdate {
        entity: walk.input,
        value: walk
            .data
            .as_2d()
            .ok_or::<BevyError>("SprintWalk didn't have 2D data".into())?,
        speed: walk_settings.sprint_speed,
    });
    Ok(())
}
