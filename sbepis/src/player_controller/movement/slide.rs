use bevy::prelude::*;
use bevy_butler::*;

use crate::player_controller::PlayerControllerPlugin;

#[derive(Resource)]
#[resource(plugin = PlayerControllerPlugin, init = PlayerSlideSettings {
	slide_speed_cap: 10.0,
	slide_friction: 1.0,
	slide_forward_friction: 0.1,
	slide_break_friction: 10.0,
	slide_turn_radius: 0.5,
	slide_turn_friction: 1.0,
})]
pub struct PlayerSlideSettings {
	pub slide_speed_cap: f32,
	pub slide_friction: f32,
	pub slide_forward_friction: f32,
	pub slide_break_friction: f32,
	pub slide_turn_radius: f32,
	pub slide_turn_friction: f32,
}
