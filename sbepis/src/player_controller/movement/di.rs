use bevy::prelude::*;
use bevy_butler::*;
use leafwing_input_manager::prelude::ActionState;
use return_ok::ok_or_return;

use crate::camera::PlayerCamera;
use crate::player_controller::movement::MovementControlSet;
use crate::player_controller::{PlayerAction, PlayerBody, PlayerControllerPlugin};

#[derive(Component, Default)]
pub struct DirectionalInput {
    pub input: Vec2,
    pub local_space: Vec3,
    pub world_space: Vec3,
    pub forward: Vec3,
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSet::UpdateDi,
)]
fn update_di(
    input: Query<&ActionState<PlayerAction>>,
    mut players: Query<&mut DirectionalInput, With<PlayerBody>>,
    player_cameras: Query<&GlobalTransform, With<PlayerCamera>>,
) {
    let input = ok_or_return!(input.single());
    let mut di = ok_or_return!(players.single_mut());
    let transform = ok_or_return!(player_cameras.single());
    di.input = input.axis_pair(&PlayerAction::Move).clamp_length_max(1.0) * Vec2::new(1.0, -1.0);
    di.local_space = Vec3::new(di.input.x, 0.0, di.input.y);
    di.world_space = transform.rotation() * di.local_space;
    di.forward = transform.rotation() * -Vec3::Z;
}
