use bevy::prelude::*;
use bevy_butler::*;
use bevy_rapier3d::prelude::*;

use crate::player_controller::movement::MovementControlSystems;
use crate::player_controller::{PlayerBody, PlayerControllerPlugin};

#[derive(Component, Default)]
pub struct Grounded;

#[derive(Component, Deref, DerefMut, Debug)]
pub struct GroundedContact(pub RayIntersection);

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSystems::UpdateGrounded,
)]
fn update_is_grounded(
    mut bodies: Query<(Entity, &GlobalTransform, &PlayerBody)>,
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
