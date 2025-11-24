use bevy::prelude::*;
use bevy_butler::*;
use bevy_pretty_nice_input::{Action, Updated};
use bevy_rapier3d::prelude::*;
use return_ok::ok_or_return_ok;

use crate::entity::Movement;
use crate::entity::movement::ExecuteMovementSet;
use crate::player_controller::PlayerControllerPlugin;
use crate::player_controller::movement::MovementControlSystems;
use crate::player_controller::movement::di::DIUpdate;

use super::di::WalkDI;
use super::grounded::Grounded;

#[derive(Action)]
#[action(invalidate = false)]
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
fn update_di_walk(
    di: On<Updated<Walk>>,
    mut players: Query<&mut Walking>,
    mut commands: Commands,
    walk_settings: Res<PlayerWalkSettings>,
) -> Result {
    let mut _walking = ok_or_return_ok!(players.get_mut(di.input));
    commands.trigger(DIUpdate {
        entity: di.input,
        value: di
            .data
            .as_2d()
            .ok_or::<BevyError>("Walk didn't have 2D data".into())?,
        speed: walk_settings.speed,
    });
    Ok(())
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSystems::DoHorizontalMovement,
	before = ExecuteMovementSet,
)]
fn update_walk_velocity(
    mut movement: Query<(&mut Movement, &Velocity, &Transform, &WalkDI, Has<Grounded>)>,
    walk_settings: Res<PlayerWalkSettings>,
    time: Res<Time>,
) {
    for (mut movement, velocity, transform, di, grounded) in movement.iter_mut() {
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
}
