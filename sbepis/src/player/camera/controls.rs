use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_pretty_nice_input::prelude::*;
use sbepistats::{Stat, StatType, StatTypeHook};

use crate::player::camera::PlayerCameraPlugin;
use crate::player::camera::third_person::PlayerOfCameraRoots;

#[derive(Action)]
pub struct Look;

#[auto_component(plugin = PlayerCameraPlugin, derive, reflect, register)]
pub struct Yaw;

#[auto_component(plugin = PlayerCameraPlugin, derive, reflect, register)]
pub struct SlerpYaw(f32);

impl SlerpYaw {
    pub fn new(angle: f32) -> SlerpYaw {
        SlerpYaw(angle)
    }
}

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
    yaws: Query<(), With<Yaw>>,
    mut pitches: Query<&mut Pitch>,
    mut transforms: Query<&mut Transform>,
    players: Query<&PlayerOfCameraRoots>,
    children: Query<&Children>,
) -> Result {
    let delta = look.data.as_2d_ok()? * sensitivity.0;
    let roots = players.get(look.input)?;

    for child in roots
        .iter()
        .flat_map(|root| std::iter::once(root).chain(children.iter_descendants(root)))
    {
        if yaws.get(child).is_ok() {
            let mut transform = transforms.get_mut(child)?;
            transform.rotate_y(-delta.x);
        }

        if let Ok(mut pitch) = pitches.get_mut(child) {
            let new_pitch = (pitch.0 + delta.y).clamp(-PI / 2., PI / 2.);
            let delta_y = new_pitch - pitch.0;
            pitch.0 = new_pitch;
            let mut transform = transforms.get_mut(child)?;
            transform.rotate_local_x(-delta_y);
        }
    }

    Ok(())
}

#[auto_system(plugin = PlayerCameraPlugin, schedule = Update)]
fn slerp_yaw(
    mut players: Query<(
        Entity,
        &mut SlerpYaw,
        &PlayerOfCameraRoots,
        &Stat<CameraSlerpRate>,
    )>,
    yaws: Query<(), With<Yaw>>,
    children: Query<&Children>,
    mut transforms: Query<&mut Transform>,
    time: Res<Time>,
    mut commands: Commands,
) -> Result {
    for (player, mut slerp, roots, slerp_rate) in players.iter_mut() {
        let new_slerp = 0f32.lerp(slerp.0, (-slerp_rate.total() * time.delta_secs()).exp());
        let delta = slerp.0 - new_slerp;
        if new_slerp.abs() < 0.01 {
            commands.entity(player).remove::<SlerpYaw>();
        } else {
            slerp.0 = new_slerp;
        }

        for child in roots
            .iter()
            .flat_map(|root| std::iter::once(root).chain(children.iter_descendants(root)))
        {
            if yaws.get(child).is_ok() {
                let mut transform = transforms.get_mut(child)?;
                transform.rotate_y(-delta);
            }
        }
    }

    Ok(())
}

#[derive(StatType)]
#[auto_plugin_build_hook(plugin = PlayerCameraPlugin, hook = StatTypeHook)]
pub struct CameraSlerpRate;
