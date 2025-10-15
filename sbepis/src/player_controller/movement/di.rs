use bevy::prelude::*;
use bevy_butler::*;
use bevy_pretty_nice_input::Pressed;

use crate::camera::PlayerCamera;
use crate::player_controller::movement::walk::Walk;
use crate::player_controller::{PlayerBody, PlayerControllerPlugin};

#[derive(Component, Default)]
pub struct DirectionalInput {
    pub input: Vec2,
    pub local_space: Vec3,
    pub world_space: Vec3,
    pub forward: Vec3,
}

#[add_observer(plugin = PlayerControllerPlugin)]
fn update_di(
    walk: On<Pressed<Walk>>,
    mut players: Query<(&mut DirectionalInput, &PlayerBody)>,
    player_cameras: Query<&GlobalTransform, With<PlayerCamera>>,
) -> Result {
    let (mut di, body) = players.get_mut(walk.input)?;
    let transform = player_cameras.get(body.camera)?;
    di.input = walk
        .data
        .as_2d()
        .ok_or::<BevyError>("Walk didn't have 2D data".into())?
        .clamp_length_max(1.0)
        * Vec2::new(1.0, -1.0);
    di.local_space = Vec3::new(di.input.x, 0.0, di.input.y);
    di.world_space = transform.rotation() * di.local_space;
    di.forward = transform.rotation() * -Vec3::Z;
    Ok(())
}
