use bevy::prelude::*;
use bevy_butler::*;
use bevy_pretty_nice_input::{Action, Updated};

use crate::player_controller::PlayerControllerPlugin;
use crate::player_controller::movement::di::DIUpdate;
use crate::player_controller::movement::walk::PlayerWalkSettings;

#[derive(Action)]
pub struct CrouchSneak;

#[derive(Action)]
pub struct WalkSneak;

#[derive(Component, Default)]
pub struct Sneaking;

#[add_observer(plugin = PlayerControllerPlugin)]
fn update_di_sneak(
    walk: On<Updated<CrouchSneak>>,
    mut commands: Commands,
    walk_settings: Res<PlayerWalkSettings>,
) -> Result {
    commands.trigger(DIUpdate {
        entity: walk.input,
        value: walk
            .data
            .as_2d()
            .ok_or::<BevyError>("CrouchSneak didn't have 2D data".into())?,
        speed: walk_settings.sneak_speed,
    });
    Ok(())
}
