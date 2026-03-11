use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;

use crate::prelude::*;

pub mod controls;
pub mod fov;
pub mod node;
pub mod third_person;

#[derive(AutoPlugin)]
#[auto_add_plugin(plugin = SbepisPlugin)]
#[auto_plugin(impl_plugin_trait)]
pub struct PlayerCameraPlugin;

#[auto_component(plugin = PlayerCameraPlugin, derive, reflect, register)]
pub struct PlayerCamera;
