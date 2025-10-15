use bevy::prelude::*;
use bevy_butler::*;
use bevy_pretty_nice_input::{Action, JustPressed};
use bevy_rapier3d::prelude::*;

use crate::entity::Movement;
use crate::entity::movement::ExecuteMovementSet;
use crate::player_controller::PlayerControllerPlugin;
use crate::player_controller::movement::MovementControlSystems;

use super::charge::{ChargeCrouching, ChargeStanding, ChargeWalking};
use super::crouch::Crouching;
use super::di::DirectionalInput;
use super::grounded::Grounded;
use super::sneak::Sneaking;
use super::sprint::Sprinting;
use super::stand::Standing;

#[derive(Action)]
pub struct Walk;

#[derive(Resource)]
#[insert_resource(plugin = PlayerControllerPlugin, init = PlayerWalkSettings {
	speed: 6.0,
	sneak_speed: 3.0,
	sprint_speed: 9.0,

	friction: 6.0,
	air_friction: 0.0,
	acceleration: 8.0,
	air_acceleration: 2.0,
})]
pub struct PlayerWalkSettings {
    pub speed: f32,
    pub sneak_speed: f32,
    pub sprint_speed: f32,

    pub friction: f32,
    pub air_friction: f32,
    pub acceleration: f32,
    pub air_acceleration: f32,
}

#[derive(Component, Default)]
pub struct Walking;

#[add_observer(plugin = PlayerControllerPlugin)]
fn standing_to_walking(walk: On<JustPressed<Walk>>, mut commands: Commands) {
    commands
        .entity(walk.input)
        .remove::<Standing>()
        .insert(Walking);
}

#[add_observer(plugin = PlayerControllerPlugin)]
fn walking_to_standing(walk: On<JustPressed<Walk>>, mut commands: Commands) {
    commands
        .entity(walk.input)
        .remove::<Walking>()
        .insert(Standing);
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSystems::DoHorizontalMovement,
	before = ExecuteMovementSet,
)]
fn update_walk_velocity(
    mut movement: Query<
        (
            &mut Movement,
            &Velocity,
            &Transform,
            &DirectionalInput,
            Has<Walking>,
            Has<Sprinting>,
            Has<Sneaking>,
            Has<Grounded>,
        ),
        Or<(
            With<Standing>,
            With<Walking>,
            With<Sprinting>,
            With<Crouching>,
            With<Sneaking>,
            With<ChargeStanding>,
            With<ChargeCrouching>,
            With<ChargeWalking>,
        )>,
    >,
    walk_settings: Res<PlayerWalkSettings>,
    time: Res<Time>,
) {
    for (mut movement, velocity, transform, di, walking, sprinting, sneaking, grounded) in
        movement.iter_mut()
    {
        // Set up vectors
        let velocity = (transform.rotation.inverse() * velocity.linvel).xz();
        let wish_velocity = di.input
            * if walking {
                walk_settings.speed
            } else if sprinting {
                walk_settings.sprint_speed
            } else if sneaking {
                walk_settings.sneak_speed
            } else {
                0.0
            };
        let wish_speed = wish_velocity.length();
        let wish_direction = wish_velocity.normalize_or_zero();
        let friction = if grounded {
            walk_settings.friction
        } else {
            walk_settings.air_friction
        };
        let acceleration = if grounded {
            walk_settings.acceleration
        } else {
            walk_settings.air_acceleration
        };

        // Apply friction
        let friction = -time.delta_secs() * friction * velocity;
        let velocity = velocity + friction;

        // Do funny quake movement
        let funny_quake_speed = velocity.dot(wish_direction);
        let add_speed = (wish_speed - funny_quake_speed)
            .clamp(0.0, acceleration * wish_speed * time.delta_secs()); // TODO: In absolute units, ignores relativity
        let new_velocity = velocity + wish_direction * add_speed;

        movement.0 = transform.rotation * Vec3::new(new_velocity.x, 0.0, new_velocity.y);
    }
}
