use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_pretty_nice_input::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::player::camera::PlayerCameraPlugin;
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
    mut pitches: Query<(&mut Transform, &mut Pitch)>,
    mut players: Query<(&mut Transform, &mut Velocity), (Without<Pitch>, With<Player>)>,
) -> Result {
    let delta = look
        .data
        .as_2d()
        .ok_or::<BevyError>("Look action expects 2d data".into())?;

    for (mut camera_transform, mut pitch) in pitches.iter_mut() {
        pitch.0 += delta.y * sensitivity.0;
        pitch.0 = pitch.0.clamp(-PI / 2., PI / 2.);
        camera_transform.rotation = Quat::from_rotation_x(-pitch.0);
    }

    {
        let (mut transform, mut velocity) = players.single_mut()?;

        transform.rotation *= Quat::from_rotation_y(-delta.x * sensitivity.0);

        velocity.angvel = velocity.angvel.reject_from(transform.rotation * Vec3::Z);
    }

    Ok(())
}
