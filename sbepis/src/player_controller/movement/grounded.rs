use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::player_controller::movement::MovementControlSystems;
use crate::player_controller::{Player, PlayerControllerPlugin};

#[auto_component(plugin = PlayerControllerPlugin, derive(Default), reflect, register)]
pub struct Grounded;

#[auto_component(plugin = PlayerControllerPlugin, derive(Deref, DerefMut, Debug))]
pub struct GroundedContact(pub RayIntersection);

#[auto_system(plugin = PlayerControllerPlugin, schedule = Update, config(
	in_set = MovementControlSystems::UpdateGrounded,
))]
fn update_is_grounded(
    mut bodies: Query<(Entity, &GlobalTransform, &Player)>,
    rapier_context: ReadRapierContext,
    mut commands: Commands,
) -> Result {
    let rapier_context = rapier_context.single()?;
    for (player, transform, body) in bodies.iter_mut() {
        let collider_entity = body.collider;
        let mut contact = None;
        rapier_context.intersect_ray(
            transform.translation() + transform.up() * 0.05,
            transform.down().into(),
            0.25,
            true,
            QueryFilter::default(),
            |collided_entity, ray_intersection| {
                if collided_entity == collider_entity {
                    true
                } else {
                    contact = Some(GroundedContact(ray_intersection));
                    false
                }
            },
        );
        if let Some(contact) = contact {
            commands.entity(player).insert((Grounded, contact));
        } else {
            commands.entity(player).remove::<Grounded>();
        }
    }
    Ok(())
}
