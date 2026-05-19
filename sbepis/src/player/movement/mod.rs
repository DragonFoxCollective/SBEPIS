use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_pretty_nice_input::prelude::TryFromActionData;

use crate::player::PlayerControllerPlugin;
use crate::player::camera::PlayerOfCamera;
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
    /// Raw controls input.
    fn as_input(&self) -> Vec2;

    /// Controls in 3D pointing relative to the given transform (usually the player body).
    fn as_motion(&self, relative_transform: &GlobalTransform) -> Vec3;

    /// Control input added with the camera relative to the player body.
    ///
    /// For example, if the input is right and the camera is looking to the right, the result would be backward (negative Y).
    fn as_camera_input(
        &self,
        camera_transform: &GlobalTransform,
        player_transform: &GlobalTransform,
    ) -> Vec2;

    /// Control input added with the camera relative to the player body, but corrected for neg-z-forward.
    ///
    /// For example, if the input is right and the camera is looking to the right, the result would be backward (positive Y).
    fn as_camera_input_motion(
        &self,
        camera_transform: &GlobalTransform,
        player_transform: &GlobalTransform,
    ) -> Vec2;
}

impl MovingOptExt for Option<&Moving> {
    fn as_input(&self) -> Vec2 {
        match self {
            Some(Moving(input)) => input.clamp_length_max(1.0),
            None => Vec2::ZERO,
        }
    }

    fn as_motion(&self, relative_transform: &GlobalTransform) -> Vec3 {
        let input = self.as_input();
        relative_transform.transform_vector3(input.extend_bevy())
    }

    fn as_camera_input(
        &self,
        camera_transform: &GlobalTransform,
        player_transform: &GlobalTransform,
    ) -> Vec2 {
        let forward = camera_transform.rejected_forward_relative_to(player_transform);
        let rotate = Vec2::NEG_Y.rotate(forward.invert_y()); // neg-y-forward to x-forward because math
        rotate.rotate(self.as_input())
    }

    fn as_camera_input_motion(
        &self,
        camera_transform: &GlobalTransform,
        player_transform: &GlobalTransform,
    ) -> Vec2 {
        self.as_camera_input(camera_transform, player_transform)
            .invert_y()
    }
}

#[auto_resource(plugin = PlayerControllerPlugin, derive, reflect, register, insert(PlayerBodyRotateSpeed(0.6)))]
struct PlayerBodyRotateSpeed(f32);

#[auto_system(plugin = PlayerControllerPlugin, schedule = Update, config(
    in_set = MovementControlSystems::ExecuteMovement,
))]
fn rotate_toward_moving(
    mut players: Query<(&mut Transform, &GlobalTransform, &Moving, &PlayerOfCamera)>,
    cameras: Query<&GlobalTransform>,
    speed: Res<PlayerBodyRotateSpeed>,
) -> Result {
    for (mut transform, global_transform, moving, camera) in players.iter_mut() {
        let camera_transform = cameras.get(**camera)?;
        let input = Some(moving).as_camera_input_motion(camera_transform, global_transform);
        let angle = input.angle_to(Vec2::NEG_Y) * speed.0;
        if !angle.is_nan() {
            transform.rotate_local_y(angle);
        }
    }
    Ok(())
}
