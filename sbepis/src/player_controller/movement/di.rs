use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;

use crate::camera::PlayerCamera;
use crate::player_controller::{PlayerBody, PlayerControllerPlugin};

#[auto_component(plugin = PlayerControllerPlugin, derive(Default, Debug), reflect, register)]
pub struct WalkDI {
    pub input: Vec2,
    pub local_space: Vec3,
    pub world_space: Vec3,
    pub forward: Vec3,
}

#[auto_event(plugin = PlayerControllerPlugin, target(entity), derive, reflect, register)]
pub struct DIUpdate {
    pub entity: Entity,
    pub value: Vec2,
    pub speed: f32,
}

#[auto_event(plugin = PlayerControllerPlugin, target(entity), derive, reflect, register)]
pub struct DIExecute {
    pub entity: Entity,
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn update_di(
    walk: On<DIUpdate>,
    mut players: Query<(&mut WalkDI, &PlayerBody)>,
    player_cameras: Query<&GlobalTransform, With<PlayerCamera>>,
) -> Result {
    let (mut di, body) = players.get_mut(walk.entity)?;
    let transform = player_cameras.get(body.camera)?;
    di.input = walk.value.clamp_length_max(1.0) * Vec2::new(1.0, -1.0) * walk.speed;
    di.local_space = Vec3::new(di.input.x, 0.0, di.input.y);
    di.world_space = transform.rotation() * di.local_space;
    di.forward = transform.rotation() * -Vec3::Z;
    Ok(())
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Update)]
fn update_di_update(
    mut players: Query<(&mut WalkDI, &PlayerBody)>,
    player_cameras: Query<&GlobalTransform, With<PlayerCamera>>,
) -> Result {
    for (mut di, body) in players.iter_mut() {
        let transform = player_cameras.get(body.camera)?;
        di.world_space = transform.rotation() * di.local_space;
        di.forward = transform.rotation() * -Vec3::Z;
    }
    Ok(())
}
