use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_pretty_nice_input::prelude::TryFromActionData;

use crate::player_controller::PlayerControllerPlugin;

pub mod charge;
pub mod crouch;
pub mod dash;
pub mod grounded;
pub mod jump;
pub mod roll;
pub mod slide;
pub mod stand;
pub mod trip;
pub mod walk;

// TODO: remove
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum MovementControlSystems {
    UpdateDi,
    UpdateGrounded,
    UpdateState,
    DoHorizontalMovement,
    DoVerticalMovement,
    ExecuteMovement,
}

#[auto_component(plugin = PlayerControllerPlugin, derive(TryFromActionData, Debug), reflect, register)]
#[action_data(Axis2D)]
pub struct Moving(pub Vec2);

pub trait MovingOptExt {
    fn as_input(&self) -> Vec2;
}

impl MovingOptExt for Option<&Moving> {
    fn as_input(&self) -> Vec2 {
        match self {
            Some(Moving(input)) => Vec2::new(input.x, -input.y).clamp_length_max(1.0),
            None => Vec2::ZERO,
        }
    }
}
