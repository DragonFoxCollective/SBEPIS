use std::time::Duration;

use bevy::prelude::*;
use bevy_butler::*;

use crate::player_controller::PlayerControllerPlugin;

pub mod charge;
pub mod crouch;
pub mod dash;
pub mod di;
pub mod grounded;
pub mod jump;
pub mod slide;
pub mod sneak;
pub mod sprint;
pub mod stand;
pub mod trip;
pub mod walk;

#[derive(Resource)]
#[insert_resource(plugin = PlayerControllerPlugin, init = CoyoteTimeSettings {
	input_buffer_time: Duration::from_secs_f32(0.5),
	coyote_time: Duration::from_secs_f32(0.2),
})]
pub struct CoyoteTimeSettings {
    pub input_buffer_time: Duration,
    pub coyote_time: Duration,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum MovementControlSet {
    UpdateDi,
    UpdateGrounded,
    UpdateState,
    DoHorizontalMovement,
    DoVerticalMovement,
}
