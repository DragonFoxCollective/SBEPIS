use bevy::mesh::CapsuleUvProfile;
use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_rapier3d::prelude::*;
use return_ok::ok_or_return_ok;

use crate::player_controller::PlayerControllerPlugin;
use crate::player_controller::movement::crouch::Crouching;
use crate::prelude::Player;

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

        let player_height = 1.8;
        let player_width = 0.6;
        let eye_height = 1.7;

        let capsule_radius = player_width * 0.5;
        let capsule_length = player_height - capsule_radius * 2.0;

        StandingAssets {
            mesh: Mesh3d(
                meshes.add(
                    Capsule3d::new(capsule_radius, capsule_length)
                        .mesh()
                        .rings(1)
                        .latitudes(8)
                        .longitudes(16)
                        .uv_profile(CapsuleUvProfile::Fixed),
                ),
            ),
            mesh_transform: Transform::from_translation(Vec3::Y * player_height * 0.5),
            collider: Collider::capsule_y(capsule_length * 0.5, capsule_radius),
            collider_transform: Transform::from_translation(Vec3::Y * player_height * 0.5),
            camera_position: Vec3::Y * eye_height,
        }
    }
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn to_standing_assets(
    add: On<Add, Standing>,
    players: Query<&Player, Without<Crouching>>,
    mut cameras: Query<&mut Transform>,
    assets: Res<StandingAssets>,
    mut commands: Commands,
) -> Result {
    let body = ok_or_return_ok!(players.get(add.entity));
    commands
        .entity(body.mesh)
        .insert((assets.mesh.clone(), assets.mesh_transform));
    commands
        .entity(body.collider)
        .insert((assets.collider.clone(), assets.collider_transform));
    cameras.get_mut(body.camera)?.translation = assets.camera_position;
    Ok(())
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn to_standing_assets_2(
    remove: On<Remove, Crouching>,
    players: Query<&Player>,
    mut cameras: Query<&mut Transform>,
    assets: Res<StandingAssets>,
    mut commands: Commands,
) -> Result {
    let body = players.get(remove.entity)?;
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
