use bevy::prelude::*;
use bevy_butler::*;
use return_ok::ok_or_return;

#[butler_plugin]
pub struct PlayerCameraPlugin;

#[derive(Component)]
pub struct PlayerCamera;

#[derive(Component)]
pub struct PlayerCameraNode;

#[system(
	plugin = PlayerCameraPlugin, schedule = Update,
)]
fn setup_player_camera_added_node(
	mut commands: Commands,
	nodes: Query<Entity, Added<PlayerCameraNode>>,
	camera: Query<Entity, With<PlayerCamera>>,
) {
	let camera = ok_or_return!(camera.get_single());
	for node in nodes.iter() {
		commands.entity(node).insert(TargetCamera(camera));
	}
}

#[system(
	plugin = PlayerCameraPlugin, schedule = Update,
)]
fn setup_player_camera_added_camera(
	mut commands: Commands,
	nodes: Query<Entity, With<PlayerCameraNode>>,
	camera: Query<Entity, Added<PlayerCamera>>,
) {
	let camera = ok_or_return!(camera.get_single());
	for node in nodes.iter() {
		commands.entity(node).insert(TargetCamera(camera));
	}
}
