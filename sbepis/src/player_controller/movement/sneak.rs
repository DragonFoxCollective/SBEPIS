use bevy::prelude::*;
use bevy_butler::*;
use bevy_pretty_nice_input::{Action, Updated};
use return_ok::ok_or_return_ok;

use crate::player_controller::PlayerControllerPlugin;
use crate::player_controller::movement::di::DIUpdate;
use crate::player_controller::movement::walk::PlayerWalkSettings;

#[derive(Action)]
#[action(invalidate = false)]
pub struct CrouchSneak;

#[derive(Action)]
#[action(invalidate = false)]
pub struct WalkSneak;

#[derive(Component, Default)]
pub struct Sneaking;

#[add_observer(plugin = PlayerControllerPlugin)]
fn update_di_sneak(
    di: On<Updated<CrouchSneak>>,
    mut players: Query<&mut Sneaking>,
    mut commands: Commands,
    walk_settings: Res<PlayerWalkSettings>,
) -> Result {
    let mut _sneaking = ok_or_return_ok!(players.get_mut(di.input));
    commands.trigger(DIUpdate {
        entity: di.input,
        value: di
            .data
            .as_2d()
            .ok_or::<BevyError>("CrouchSneak didn't have 2D data".into())?,
        speed: walk_settings.sneak_speed,
    });
    Ok(())
}
