use std::time::Duration;

use bevy::prelude::*;
use bevy_butler::*;
use dash::DashAssets;

use crate::player_controller::PlayerControllerPlugin;

pub mod crouch;
pub mod dash;
pub mod di;
pub mod grounded;
pub mod jump;
pub mod walk;

#[derive(Resource)]
#[resource(plugin = PlayerControllerPlugin, init = PlayerSpeed {
	speed: 6.0,
	sneak_speed: 3.0,
	sprint_speed: 9.0,

	friction: 6.0,
	air_friction: 0.0,
	acceleration: 8.0,
	air_acceleration: 2.0,

	jump_speed: 5.0,
	high_jump_speed: 7.0,

	dash_speed_addon: 12.0,
	dash_time: Duration::from_secs_f32(0.3),
	dash_cooldown: Duration::from_secs_f32(0.2),
	dash_stamina_cost: 0.33,

	input_buffer_time: Duration::from_secs_f32(0.1),
	coyote_time: Duration::from_secs_f32(0.1),
})]
pub struct PlayerSpeed {
	pub speed: f32,
	pub sneak_speed: f32,
	pub sprint_speed: f32,

	pub friction: f32,
	pub air_friction: f32,
	pub acceleration: f32,
	pub air_acceleration: f32,

	pub jump_speed: f32,
	pub high_jump_speed: f32,

	pub dash_speed_addon: f32,
	pub dash_time: Duration,
	pub dash_cooldown: Duration,
	pub dash_stamina_cost: f32,

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
	UpdateCrouching,
	DoHorizontalMovement,
	DoVerticalMovement,
}

#[system(plugin = PlayerControllerPlugin, schedule = Startup)]
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
	commands.insert_resource(DashAssets {
		sound: asset_server.load("ultrakill dash sound.mp3"),
	});
}
