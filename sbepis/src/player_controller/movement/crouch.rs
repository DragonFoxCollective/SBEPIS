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
    pub collider: Collider,
    pub collider_transform: Transform,
    pub camera_position: Vec3,
}

impl Default for CrouchingAssets {
    fn default() -> Self {
        let player_height = 0.8;
        let player_width = 0.6;
        let eye_height = 0.6;

        let capsule_radius = player_width * 0.5;
        let capsule_length = player_height - capsule_radius * 2.0;

        CrouchingAssets {
            collider: Collider::capsule_y(capsule_length * 0.5, capsule_radius),
            collider_transform: Transform::from_translation(Vec3::Y * player_height * 0.5),
            camera_position: Vec3::Y * eye_height,
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
