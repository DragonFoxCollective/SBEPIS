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

#[auto_component(plugin = PlayerCameraPlugin, derive(Deref), reflect, register)]
#[relationship_target(relationship = CameraOfPlayer, linked_spawn)]
pub struct PlayerOfCamera(Entity);

#[auto_component(plugin = PlayerCameraPlugin, derive(Deref), reflect, register)]
#[relationship(relationship_target = PlayerOfCamera)]
pub struct CameraOfPlayer(pub Entity);
