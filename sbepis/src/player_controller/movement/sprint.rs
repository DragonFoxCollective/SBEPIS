use bevy::prelude::*;
use bevy_butler::*;
use bevy_pretty_nice_input::{Action, Updated};
use return_ok::ok_or_return_ok;

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
    di: On<Updated<SprintWalk>>,
    mut players: Query<&mut Sprinting>,
    mut commands: Commands,
    walk_settings: Res<PlayerWalkSettings>,
) -> Result {
    let mut _sprinting = ok_or_return_ok!(players.get_mut(di.input));
    commands.trigger(DIUpdate {
        entity: di.input,
        value: di
            .data
            .as_2d()
            .ok_or::<BevyError>("SprintWalk didn't have 2D data".into())?,
        speed: walk_settings.sprint_speed,
    });
    Ok(())
}
