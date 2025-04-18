use std::any::type_name;
use std::time::Duration;

use bevy::prelude::*;
use bevy_butler::*;
use charge::{ChargeCrouching, Charging};
use crouch::Crouching;
use dash::{Dashing, TryingToDash};
use grounded::{EffectiveGrounded, Grounded};
use itertools::Itertools;
use jump::TryingToJump;
use slide::Sliding;
use sneak::Sneaking;
use sprint::Sprinting;
use stand::Standing;
use walk::Walking;

use crate::player_controller::PlayerControllerPlugin;

use super::PlayerBody;

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
			Has<TryingToDash>,
			Has<Charging>,
			Has<ChargeCrouching>,
			Has<Sliding>,
			Has<Grounded>,
			Has<EffectiveGrounded>,
			Has<TryingToJump>,
		),
		With<PlayerBody>,
	>,
	mut debug_states: Query<&mut Text, With<DebugState>>,
) {
	let mut debug_state = debug_states.single_mut();
	for tup in players.iter() {
		let arr = [
			tup.0, tup.1, tup.2, tup.3, tup.4, tup.5, tup.6, tup.7, tup.8, tup.9, tup.10, tup.11,
			tup.12,
		];
		let has = arr
			.into_iter()
			.zip([
				type_name::<Standing>(),
				type_name::<Walking>(),
				type_name::<Sprinting>(),
				type_name::<Crouching>(),
				type_name::<Sneaking>(),
				type_name::<Dashing>(),
				type_name::<TryingToDash>(),
				type_name::<Charging>(),
				type_name::<ChargeCrouching>(),
				type_name::<Sliding>(),
				type_name::<Grounded>(),
				type_name::<EffectiveGrounded>(),
				type_name::<TryingToJump>(),
			])
			.filter_map(|(has, name)| if has { Some(name) } else { None })
			.map(|name| name.split("::").last().unwrap())
			.join("\n");
		debug_state.0 = has;
	}
}
