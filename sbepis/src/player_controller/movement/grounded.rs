use std::time::Duration;

use bevy::prelude::*;
use bevy_butler::*;
use bevy_rapier3d::prelude::*;

use crate::player_controller::movement::MovementControlSet;
use crate::player_controller::{PlayerBody, PlayerControllerPlugin};

use super::CoyoteTimeSettings;

#[derive(Component, Default)]
pub struct Grounded;

#[derive(Component, Default)]
pub struct EffectiveGrounded(pub Duration);

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSet::UpdateGrounded,
)]
fn update_is_grounded(
    mut bodies: Query<(Entity, &GlobalTransform, &PlayerBody)>,
    rapier_context: ReadRapierContext,
    mut commands: Commands,
) -> Result {
    let rapier_context = rapier_context.single()?;
    for (player, transform, body) in bodies.iter_mut() {
        let collider_entity = body.collider;
        let mut grounded = false;
        rapier_context.intersections_with_shape(
            transform.translation(),
            Quat::IDENTITY,
            &Collider::ball(0.25),
            QueryFilter::default(),
            |collided_entity| {
                if collided_entity == collider_entity {
                    true
                } else {
                    grounded = true;
                    false
                }
            },
        );
        if grounded {
            commands.entity(player).insert(Grounded);
        } else {
            commands.entity(player).remove::<Grounded>();
        }
    }
    Ok(())
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	after = update_is_grounded,
	in_set = MovementControlSet::UpdateGrounded,
)]
fn add_effective_grounded(
    players: Query<Entity, (With<Grounded>, Without<EffectiveGrounded>)>,
    mut commands: Commands,
) {
    for player in players.iter() {
        commands.entity(player).insert(EffectiveGrounded::default());
    }
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	after = update_is_grounded,
	before = add_effective_grounded,
	in_set = MovementControlSet::UpdateGrounded,
)]
fn update_effective_grounded_time(
    mut players: Query<(Entity, &mut EffectiveGrounded), Without<Grounded>>,
    time: Res<Time>,
    mut commands: Commands,
    coyote_time_settings: Res<CoyoteTimeSettings>,
) {
    for (player, mut grounded) in players.iter_mut() {
        grounded.0 += time.delta();
        if grounded.0 >= coyote_time_settings.coyote_time {
            commands.entity(player).remove::<EffectiveGrounded>();
        }
    }
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	after = update_is_grounded,
	before = add_effective_grounded,
	in_set = MovementControlSet::UpdateGrounded,
)]
fn reset_effective_grounded_time(mut players: Query<&mut EffectiveGrounded, With<Grounded>>) {
    for mut grounded in players.iter_mut() {
        grounded.0 = Duration::ZERO;
    }
}
