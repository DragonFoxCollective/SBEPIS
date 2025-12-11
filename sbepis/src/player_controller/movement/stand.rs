use bevy::mesh::CapsuleUvProfile;
use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::player_controller::PlayerControllerPlugin;
use crate::prelude::PlayerBody;

use super::charge::ChargeStanding;
use super::sprint::Sprinting;
use super::walk::Walking;

#[auto_resource(plugin = PlayerControllerPlugin, derive, init)]
pub struct StandingAssets {
    pub mesh: Mesh3d,
    pub mesh_transform: Transform,
    pub collider: Collider,
    pub collider_transform: Transform,
    pub camera_position: Vec3,
}

impl FromWorld for StandingAssets {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();

        StandingAssets {
            mesh: Mesh3d(
                meshes.add(
                    Capsule3d::new(0.25, 1.0)
                        .mesh()
                        .rings(1)
                        .latitudes(8)
                        .longitudes(16)
                        .uv_profile(CapsuleUvProfile::Fixed),
                ),
            ),
            mesh_transform: Transform::from_translation(Vec3::Y * 0.75),
            collider: Collider::capsule_y(0.5, 0.25),
            collider_transform: Transform::from_translation(Vec3::Y * 0.75),
            camera_position: Vec3::Y * 1.25,
        }
    }
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn to_standing_assets(
    add: On<Add, (Standing, ChargeStanding, Walking, Sprinting)>,
    players: Query<&PlayerBody>,
    mut cameras: Query<&mut Transform>,
    assets: Res<StandingAssets>,
    mut commands: Commands,
) -> Result {
    let body = players.get(add.entity)?;
    commands
        .entity(body.mesh)
        .insert((assets.mesh.clone(), assets.mesh_transform));
    commands
        .entity(body.collider)
        .insert((assets.collider.clone(), assets.collider_transform));
    cameras.get_mut(body.camera)?.translation = assets.camera_position;
    Ok(())
}

#[auto_component(plugin = PlayerControllerPlugin, derive(Default), reflect, register)]
pub struct Standing;
