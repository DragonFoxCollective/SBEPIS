use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_pretty_nice_input::prelude::TryFromActionData;

use crate::player::PlayerControllerPlugin;
use crate::prelude::*;

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
    fn as_camera_input(
        &self,
        camera_transform: &GlobalTransform,
        player_transform: &GlobalTransform,
    ) -> Vec2;
}

impl MovingOptExt for Option<&Moving> {
    fn as_input(&self) -> Vec2 {
        match self {
            Some(Moving(input)) => Vec2::new(input.x, -input.y).clamp_length_max(1.0),
            None => Vec2::ZERO,
        }
    }

    fn as_camera_input(
        &self,
        camera_transform: &GlobalTransform,
        player_transform: &GlobalTransform,
    ) -> Vec2 {
        let camera_forward =
            player_transform.inverse_transform_vector3(camera_transform.forward().into());
        let camera_up = player_transform.inverse_transform_vector3(camera_transform.up().into());
        let rotate = if camera_forward.xz().length() >= 0.1 {
            camera_forward.xz().normalize()
        } else if camera_forward.y > 0.0 {
            -camera_up.xz().normalize()
        } else {
            camera_up.xz().normalize()
        }
        .rotate(Vec2::Y); // x-forward to y-forward
        self.as_input().rotate(rotate)
    }
}
