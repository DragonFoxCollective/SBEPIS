use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::prelude::PlayerBody;

#[derive(Resource)]
pub struct StandingAssets {
    pub mesh: Mesh3d,
    pub mesh_transform: Transform,
    pub collider: Collider,
    pub collider_transform: Transform,
    pub camera_transform: Transform,
}

pub fn to_standing_assets(body: &PlayerBody, commands: &mut Commands, assets: &StandingAssets) {
    commands
        .entity(body.mesh)
        .insert((assets.mesh.clone(), assets.mesh_transform));
    commands
        .entity(body.collider)
        .insert((assets.collider.clone(), assets.collider_transform));
    commands.entity(body.camera).insert(assets.camera_transform);
}

#[derive(Component, Default)]
pub struct Standing;
