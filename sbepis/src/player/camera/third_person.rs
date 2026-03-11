use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;

use crate::player::camera::PlayerCameraPlugin;

#[auto_component(plugin = PlayerCameraPlugin, derive, reflect, register)]
pub struct PlayerAimCamera;

#[auto_component(plugin = PlayerCameraPlugin, derive, reflect, register)]
pub struct PlayerAimHead;

#[auto_component(plugin = PlayerCameraPlugin, derive, reflect, register)]
#[require(Transform)]
pub struct HiddenInFirstPerson;

#[auto_observer(plugin = PlayerCameraPlugin)]
fn hide_first_person(
    add: On<Add, HiddenInFirstPerson>,
    mut transforms: Query<&mut Transform>,
) -> Result {
    let mut transform = transforms.get_mut(add.entity)?;
    transform.scale = Vec3::ZERO;
    Ok(())
}
