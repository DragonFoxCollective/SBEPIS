use bevy::prelude::*;
use bevy_butler::*;
use bevy_rapier3d::prelude::*;

use crate::entity::Movement;
use crate::input::button_just_pressed;
use crate::player_controller::movement::MovementControlSet;
use crate::player_controller::movement::slide::sliding_to_crouching_or_sneaking;
use crate::player_controller::{PlayerAction, PlayerControllerPlugin};
use crate::prelude::PlayerBody;

use super::slide::Sliding;

#[derive(Resource)]
pub struct RollingAssets {
    pub mesh: Mesh3d,
    pub mesh_transform: Transform,
    pub collider: Collider,
    pub collider_transform: Transform,
    pub camera_transform: Transform,
}

pub fn to_rolling_assets(body: &PlayerBody, commands: &mut Commands, assets: &RollingAssets) {
    commands
        .entity(body.mesh)
        .insert((assets.mesh.clone(), assets.mesh_transform));
    commands
        .entity(body.collider)
        .insert((assets.collider.clone(), assets.collider_transform));
    commands.entity(body.camera).insert(assets.camera_transform);
}

#[derive(Component)]
pub struct Rolling;

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_pressed(PlayerAction::Sprint),
	in_set = MovementControlSet::UpdateState,
	before = sliding_to_crouching_or_sneaking,
)]
fn sliding_to_rolling(
    mut players: Query<(Entity, &PlayerBody), With<Sliding>>,
    mut commands: Commands,
    assets: Res<RollingAssets>,
) {
    for (player, body) in players.iter_mut() {
        commands
            .entity(player)
            .remove::<Sliding>()
            .remove::<Movement>()
            .insert(Rolling);
        to_rolling_assets(body, &mut commands, &assets);
    }
}
