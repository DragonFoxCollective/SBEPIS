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
        let input = Some(moving).as_camera_input(camera_transform, global_transform);
        let angle = input.angle_to(Vec2::NEG_Y) * speed.0;
        if !angle.is_nan() {
            transform.rotate_local_y(angle);
        }
    }
    Ok(())
}
