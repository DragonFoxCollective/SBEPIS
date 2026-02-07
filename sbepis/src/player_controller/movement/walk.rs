use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_pretty_nice_input::{Action, Updated};
use bevy_rapier3d::prelude::*;

use crate::entity::Movement;
use crate::player_controller::PlayerControllerPlugin;
use crate::player_controller::movement::MovementControlSystems;
use crate::player_controller::movement::charge::Charging;
use crate::player_controller::movement::di::{DIUpdate, WalkDI};
use crate::player_controller::movement::stand::Standing;

use super::grounded::Grounded;

#[derive(Action)]
#[action(invalidate = false)]
pub struct StandingDI;

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

#[auto_observer(plugin = PlayerControllerPlugin)]
fn update_di_walk(
    di: On<Updated<StandingDI>>,
    mut commands: Commands,
    walk_settings: Res<PlayerWalkSettings>,
) -> Result {
    commands.trigger(DIUpdate {
        entity: di.input,
        value: di
            .data
            .as_2d()
            .ok_or::<BevyError>("StandingDI didn't have 2D data".into())?,
        speed: walk_settings.speed,
    });
    Ok(())
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Update, config(
	in_set = MovementControlSystems::DoHorizontalMovement,
))]
fn update_walk_velocity(
    mut players: Query<
        (&mut Movement, &Velocity, &Transform, &WalkDI, Has<Grounded>),
        Or<(With<Standing>, With<Charging>)>, // ewwwww two states?
    >,
    walk_settings: Res<PlayerWalkSettings>,
    time: Res<Time>,
) -> Result {
    for (mut movement, velocity, transform, di, grounded) in players.iter_mut() {
        // Set up vectors
        let velocity = (transform.rotation.inverse() * velocity.linvel).xz();
        let wish_velocity = di.input;
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

    Ok(())
}
