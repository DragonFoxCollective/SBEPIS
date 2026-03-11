use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_pretty_nice_input::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::player::camera::{PlayerCamera, PlayerCameraPlugin};
use crate::prelude::*;

#[derive(Action)]
pub struct Look;

#[auto_component(plugin = PlayerCameraPlugin, derive, reflect, register)]
pub struct Pitch(pub f32);

/// Probably in radians per mouse sensor pixel?
#[auto_resource(plugin = PlayerCameraPlugin, derive, reflect, register, init)]
pub struct MouseSensitivity(pub f32);

impl Default for MouseSensitivity {
    fn default() -> Self {
        Self(0.0015)
    }
}

#[auto_observer(plugin = PlayerCameraPlugin)]
fn rotate_camera_and_body(
    look: On<Pressed<Look>>,
    sensitivity: Res<MouseSensitivity>,
    mut player_camera: Query<
        (&mut Transform, &mut Pitch, &Camera),
        (With<PlayerCamera>, Without<Player>),
    >,
    mut player_body: Query<(&mut Transform, &mut Velocity), (Without<PlayerCamera>, With<Player>)>,
) -> Result {
    let delta = look
        .data
        .as_2d()
        .ok_or::<BevyError>("Look action expects 2d data".into())?;

    {
        let (mut camera_transform, mut camera_pitch, camera) = player_camera.single_mut()?;
        if !camera.is_active {
            return Ok(());
        }

        camera_pitch.0 += delta.y * sensitivity.0;
        camera_pitch.0 = camera_pitch.0.clamp(-PI / 2., PI / 2.);
        camera_transform.rotation = Quat::from_rotation_x(-camera_pitch.0);
    }

    {
        let (mut body_transform, mut body_velocity) = player_body.single_mut()?;

        body_transform.rotation *= Quat::from_rotation_y(-delta.x * sensitivity.0);

        body_velocity.angvel = body_velocity
            .angvel
            .reject_from(body_transform.rotation * Vec3::Z);
    }

    Ok(())
}
