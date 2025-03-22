use std::time::Duration;

use bevy::prelude::*;
use bevy_butler::*;
use crouch::Crouching;
use dash::Dashing;
use grounded::Grounded;
use jump::TryingToJump;
use slide::Sliding;
use sprint::Sprinting;
use stand::Standing;
use walk::Walking;

use crate::player_controller::PlayerControllerPlugin;

use super::PlayerBody;

pub mod crouch;
pub mod dash;
pub mod di;
pub mod grounded;
pub mod jump;
pub mod slide;
pub mod sprint;
pub mod stand;
pub mod walk;

#[derive(Resource)]
#[resource(plugin = PlayerControllerPlugin, init = CoyoteTimeSettings {
	input_buffer_time: Duration::from_secs_f32(0.1),
	coyote_time: Duration::from_secs_f32(0.1),
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

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	after = MovementControlSet::DoHorizontalMovement,
	after = MovementControlSet::DoVerticalMovement,
)]
fn check_states(
	players: Query<
		(
			Has<Standing>,
			Has<Walking>,
			Has<Sprinting>,
			Has<Crouching>,
			Has<Dashing>,
			Has<Sliding>,
			Has<Grounded>,
			Has<TryingToJump>,
		),
		With<PlayerBody>,
	>,
) {
	for tup in players.iter() {
		let arr: [bool; 8] = tup.into();
		let has = arr
			.into_iter()
			.enumerate()
			.filter_map(|(index, has)| {
				if has {
					Some(match index {
						0 => "Standing",
						1 => "Walking",
						2 => "Sprinting",
						3 => "Crouching",
						4 => "Dashing",
						5 => "Sliding",
						6 => "Grounded",
						7 => "TryingToJump",
						_ => unreachable!(),
					})
				} else {
					None
				}
			})
			.collect::<Vec<_>>();
		println!("Player state: {:?}", has);
	}
}
