use crate::prelude::*;
use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use return_ok::ok_or_return;

#[derive(AutoPlugin)]
#[auto_add_plugin(plugin = SbepisPlugin)]
#[auto_plugin(impl_plugin_trait)]
pub struct PlayerCameraPlugin;

#[auto_component(plugin = PlayerCameraPlugin, derive, reflect, register)]
pub struct PlayerCamera;

#[auto_component(plugin = PlayerCameraPlugin, derive, reflect, register)]
pub struct PlayerCameraNode;

#[auto_system(plugin = PlayerCameraPlugin, schedule = Update)]
fn setup_player_camera_added_node(
    mut commands: Commands,
    nodes: Query<Entity, Added<PlayerCameraNode>>,
    camera: Query<Entity, With<PlayerCamera>>,
) {
    let camera = ok_or_return!(camera.single());
    for node in nodes.iter() {
        commands.entity(node).insert(UiTargetCamera(camera));
    }
}

#[auto_system(plugin = PlayerCameraPlugin, schedule = Update)]
fn setup_player_camera_added_camera(
    mut commands: Commands,
    nodes: Query<Entity, With<PlayerCameraNode>>,
    camera: Query<Entity, Added<PlayerCamera>>,
) {
    let camera = ok_or_return!(camera.single());
    for node in nodes.iter() {
        commands.entity(node).insert(UiTargetCamera(camera));
    }
}
