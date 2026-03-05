use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::entity::Movement;
use crate::player_controller::PlayerControllerPlugin;
use crate::player_controller::movement::charge::Charging;
use crate::player_controller::movement::grounded::Grounded;
use crate::player_controller::movement::stand::Standing;
use crate::player_controller::movement::{MovementControlSystems, Moving, MovingOptExt as _};

#[auto_component(plugin = PlayerControllerPlugin, derive(Default), reflect, register)]
pub struct Sprinting;

#[auto_resource(plugin = PlayerControllerPlugin, derive, reflect, register, init)]
pub struct PlayerWalkSettings {
    pub speed: f32,
    pub sneak_speed: f32,
    pub sprint_speed: f32,

    pub friction: f32,
    pub air_friction: f32,
    pub acceleration: f32,
    pub air_acceleration: f32,
}

impl Default for PlayerWalkSettings {
    fn default() -> Self {
        Self {
            speed: 6.0,
            sneak_speed: 3.0,
            sprint_speed: 9.0,

            friction: 6.0,
            air_friction: 0.0,
            acceleration: 8.0,
            air_acceleration: 2.0,
        }
    }
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Update, config(
	in_set = MovementControlSystems::DoHorizontalMovement,
))]
fn update_walk_velocity(
    mut players: Query<
        (
            &mut Movement,
            &Velocity,
            &GlobalTransform,
            Option<&Moving>,
            Has<Grounded>,
            Has<Sprinting>,
            Has<Charging>,
        ),
        Or<(With<Standing>, With<Charging>)>, // ewwwww two states?
    >,
    walk_settings: Res<PlayerWalkSettings>,
    time: Res<Time>,
) -> Result {
    for (mut movement, velocity, transform, moving, grounded, sprinting, charging) in
        players.iter_mut()
    {
        // Set up vectors
        let velocity = (transform.rotation().inverse() * velocity.linvel).xz();
        let input = if charging {
            Vec2::ZERO
        } else {
            moving.as_input()
        };
        let wish_speed = if sprinting {
            walk_settings.sprint_speed
        } else {
            walk_settings.speed
        };
        let wish_velocity = input * wish_speed;
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

        movement.0 = transform.rotation() * Vec3::new(new_velocity.x, 0.0, new_velocity.y);
    }

    Ok(())
}
