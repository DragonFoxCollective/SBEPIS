use bevy::prelude::*;

pub mod charge;
pub mod crouch;
pub mod dash;
pub mod di;
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
