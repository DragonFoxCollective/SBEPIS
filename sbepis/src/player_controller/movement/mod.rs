use std::any::type_name;
use std::time::Duration;

use bevy::prelude::*;
use bevy_butler::*;
use charge::{ChargeCrouching, ChargeStanding, ChargeWalking};
use crouch::Crouching;
use dash::{Dashing, TryingToDash};
use grounded::{EffectiveGrounded, Grounded};
use itertools::Itertools;
use jump::TryingToJump;
use slide::Sliding;
use sneak::Sneaking;
use sprint::Sprinting;
use stand::Standing;
use trip::{TripRecoverInAir, TripRecoverOnGround, Tripping, TryingToGroundParry};
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

#[derive(Component)]
pub struct DebugState;

#[add_system(
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
            (
                Has<ChargeStanding>,
                Has<ChargeCrouching>,
                Has<ChargeWalking>,
            ),
            (
                Has<Tripping>,
                Has<TripRecoverInAir>,
                Has<TripRecoverOnGround>,
            ),
            Has<Sliding>,
            Has<Grounded>,
            Has<EffectiveGrounded>,
            (
                Has<TryingToDash>,
                Has<TryingToJump>,
                Has<TryingToGroundParry>,
            ),
        ),
        With<PlayerBody>,
    >,
    mut debug_states: Query<&mut Text, With<DebugState>>,
) -> Result {
    let mut debug_state = debug_states.single_mut()?;
    for tup in players.iter() {
        let arr = [
            tup.0, tup.1, tup.2, tup.3, tup.4, tup.5, tup.6.0, tup.6.1, tup.6.2, tup.7.0, tup.7.1,
            tup.7.2, tup.8, tup.9, tup.10, tup.11.0, tup.11.1, tup.11.2,
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
                type_name::<ChargeStanding>(),
                type_name::<ChargeCrouching>(),
                type_name::<ChargeWalking>(),
                type_name::<Tripping>(),
                type_name::<TripRecoverInAir>(),
                type_name::<TripRecoverOnGround>(),
                type_name::<Sliding>(),
                type_name::<Grounded>(),
                type_name::<EffectiveGrounded>(),
                type_name::<TryingToDash>(),
                type_name::<TryingToJump>(),
                type_name::<TryingToGroundParry>(),
            ])
            .filter_map(|(has, name)| if has { Some(name) } else { None })
            .map(|name| name.split("::").last().unwrap())
            .join("\n");
        debug_state.0 = has;
    }
    Ok(())
}
