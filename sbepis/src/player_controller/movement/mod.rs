use std::time::Duration;

use bevy::prelude::*;
use bevy_butler::*;

use crate::player_controller::PlayerControllerPlugin;

pub mod dash;
pub mod di;
pub mod grounded;
pub mod jump;
pub mod walk;

#[derive(Resource)]
#[resource(plugin = PlayerControllerPlugin, init = PlayerSpeed {
	speed: 6.0,
	sprint_modifier: 1.5,
	jump_speed: 5.0,
	friction: 6.0,
	air_friction: 0.0,
	acceleration: 8.0,
	air_acceleration: 2.0,

	dash_speed_addon: 12.0,
	dash_time: Duration::from_secs_f32(0.3),

	input_buffer_time: Duration::from_secs_f32(0.1),
	coyote_time: Duration::from_secs_f32(0.1),
})]
pub struct PlayerSpeed {
	pub speed: f32,
	pub sprint_modifier: f32,
	pub jump_speed: f32,
	pub friction: f32,
	pub air_friction: f32,
	pub acceleration: f32,
	pub air_acceleration: f32,

	pub dash_speed_addon: f32,
	pub dash_time: Duration,

	pub input_buffer_time: Duration,
	pub coyote_time: Duration,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum MovementControlSet {
	UpdateDi,
	UpdateGrounded,
	UpdateJumping,
	UpdateDashing,
	UpdateSprinting,
	DoHorizontalMovement,
	DoVerticalMovement,
}
