use bevy::mesh::CapsuleUvProfile;
use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::player_controller::PlayerControllerPlugin;
use crate::player_controller::movement::MovementControlSystems;
use crate::player_controller::movement::grounded::{Grounded, GroundedContact};
use crate::player_controller::movement::slide::PlayerSlideSettings;
use crate::player_controller::movement::stand::Standing;
use crate::prelude::Player;

use super::slide::Sliding;

#[auto_resource(plugin = PlayerControllerPlugin, derive, init)]
pub struct CrouchingAssets {
    pub mesh: Mesh3d,
    pub mesh_transform: Transform,
    pub collider: Collider,
    pub collider_transform: Transform,
    pub camera_position: Vec3,
}

impl FromWorld for CrouchingAssets {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();

        CrouchingAssets {
            mesh: Mesh3d(
                meshes.add(
                    Capsule3d::new(0.25, 0.5)
                        .mesh()
                        .rings(1)
                        .latitudes(8)
                        .longitudes(16)
                        .uv_profile(CapsuleUvProfile::Fixed),
                ),
            ),
            mesh_transform: Transform::from_translation(Vec3::Y * 0.5),
            collider: Collider::capsule_y(0.25, 0.25),
            collider_transform: Transform::from_translation(Vec3::Y * 0.5),
            camera_position: Vec3::Y * 0.75,
        }
    }
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn to_crouching_assets(
    add: On<Add, (Crouching, Sliding)>,
    players: Query<&Player>,
    mut cameras: Query<&mut Transform>,
    assets: Res<CrouchingAssets>,
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
pub struct Crouching;

#[auto_system(plugin = PlayerControllerPlugin, schedule = Update, config(
    in_set = MovementControlSystems::UpdateState,
))]
fn start_slide_on_slope(
    players: Query<
        (Entity, &GroundedContact, &GlobalTransform),
        (With<Crouching>, With<Standing>, With<Grounded>),
    >,
    mut commands: Commands,
    slide_settings: Res<PlayerSlideSettings>,
) {
    for (player, grounded, transform) in players.iter() {
        if grounded.normal.angle_between(transform.up().into()) > slide_settings.slope_slip_angle {
            commands
                .entity(player)
                .remove::<Crouching>()
                .remove::<Standing>()
                .insert(Sliding);
        }
    }
}
