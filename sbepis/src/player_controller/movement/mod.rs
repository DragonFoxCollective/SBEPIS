use std::any::type_name;
use std::time::Duration;

use bevy::prelude::*;
use bevy_butler::*;
use crouch::Crouching;
use dash::Dashing;
use grounded::Grounded;
use itertools::Itertools;
use jump::TryingToJump;
use slide::Sliding;
use sneak::Sneaking;
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
pub mod sneak;
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

#[derive(Component)]
pub struct DebugState;

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
			Has<Sneaking>,
			Has<Dashing>,
			Has<Sliding>,
			Has<Grounded>,
			Has<TryingToJump>,
		),
		With<PlayerBody>,
	>,
	mut debug_states: Query<&mut Text, With<DebugState>>,
) {
	let mut debug_state = debug_states.single_mut();
	for tup in players.iter() {
		let arr: [bool; 9] = tup.into();
		let has = arr
			.into_iter()
			.zip([
				type_name::<Standing>(),
				type_name::<Walking>(),
				type_name::<Sprinting>(),
				type_name::<Crouching>(),
				type_name::<Sneaking>(),
				type_name::<Dashing>(),
				type_name::<Sliding>(),
				type_name::<Grounded>(),
				type_name::<TryingToJump>(),
			])
			.filter_map(|(has, name)| if has { Some(name) } else { None })
			.map(|name| name.split("::").last().unwrap())
			.join("\n");
		debug_state.0 = has;
	}
}
