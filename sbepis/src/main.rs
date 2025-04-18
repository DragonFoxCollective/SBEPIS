#![cfg_attr(not(feature = "terminal"), windows_subsystem = "windows")]

use std::io::Cursor;

use ::blenvy::blueprints::spawn_from_blueprints::{BlueprintInfo, HideUntilReady, SpawnBlueprint};
use bevy::input::common_conditions::input_just_pressed;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::winit::WinitWindows;
use bevy_rapier3d::prelude::*;
use winit::window::Icon;

use self::main_bundles::*;

mod blenvy;
mod camera;
mod dialogue;
mod entity;
mod fray;
mod gravity;
mod input;
mod inventory;
mod main_bundles;
pub mod menus;
mod npcs;
#[cfg(feature = "overview_camera")]
mod overview_camera;
mod player_commands;
mod player_controller;
mod questing;
mod skybox;
pub mod util;

mod prelude {
	pub use crate::camera::PlayerCameraNode;
	pub use crate::player_controller::PlayerBody;
	pub use crate::player_controller::camera_controls::{
		InteractedWith, InteractedWithSet, interact_with,
	};
}

fn main() {
	let mut app = App::new();
	app
		.add_plugins((
			DefaultPlugins
				.set(WindowPlugin {
					primary_window: Some(Window {
						title: "SBEPIS".to_string(),
						..default()
					}),
					..default()
				})
				.set(ImagePlugin {
					default_sampler: bevy::image::ImageSamplerDescriptor {
						address_mode_u: bevy::image::ImageAddressMode::Repeat,
						address_mode_v: bevy::image::ImageAddressMode::Repeat,
						address_mode_w: bevy::image::ImageAddressMode::Repeat,
						..default()
					},
				})
				.set(LogPlugin {
					filter: "info,sbepis=debug,avian3d=debug,wgpu=error,naga=warn,calloop=error,symphonia_core=warn,symphonia_bundle_mp3=warn,blenvy=error".into(),
					..default()
				}),
			RapierPhysicsPlugin::<NoUserData>::default(),
			#[cfg(feature = "rapier_debug")]
			RapierDebugRenderPlugin::default(),
			#[cfg(feature = "inspector")]
			bevy_inspector_egui::quick::WorldInspectorPlugin::new(),
			#[cfg(feature = "overview_camera")]
			overview_camera::OverviewCameraPlugin,
			bevy_hanabi::HanabiPlugin,
			#[cfg(feature = "movement_indicators")]
			player_controller::speed_indicator::SpeedIndicatorPlugin,
		));

	app.add_plugins((
		player_commands::PlayerCommandsPlugin,
		camera::PlayerCameraPlugin,
		skybox::SkyboxPlugin,
		entity::EntityPlugin,
		player_controller::PlayerControllerPlugin,
		npcs::NpcPlugin,
		gravity::GravityPlugin,
		fray::FrayPlugin,
		questing::QuestingPlugin,
		menus::MenusPlugin,
		inventory::InventoryPlugin,
		blenvy::BlenvyPlugin,
	))
	.add_systems(Startup, (set_window_icon, setup))
	.add_systems(
		Update,
		(
			quit.run_if(input_just_pressed(KeyCode::Escape)),
			util::despawn_after_timer,
			util::billboard,
		),
	)
	.run();
}

fn set_window_icon(windows: NonSend<WinitWindows>) {
	let icon_buf = Cursor::new(include_bytes!("../assets/house.png"));
	let image = image::load(icon_buf, image::ImageFormat::Png).unwrap();
	let image = image.into_rgba8();
	let (width, height) = image.dimensions();
	let rgba = image.into_raw();
	let icon = Icon::from_rgba(rgba, width, height).unwrap();

	for window in windows.windows.values() {
		window.set_window_icon(Some(icon.clone()));
	}
}

fn gridbox_texture(color: &str) -> String {
	format!("Gridbox Prototype Materials/prototype_512x512_{color}.png")
}

fn gridbox_material(
	color: &str,
	materials: &mut Assets<StandardMaterial>,
	asset_server: &AssetServer,
) -> Handle<StandardMaterial> {
	gridbox_material_extra(color, materials, asset_server, StandardMaterial::default())
}

fn gridbox_material_extra(
	color: &str,
	materials: &mut Assets<StandardMaterial>,
	asset_server: &AssetServer,
	material: StandardMaterial,
) -> Handle<StandardMaterial> {
	materials.add(StandardMaterial {
		base_color_texture: Some(asset_server.load(gridbox_texture(color))),
		..material
	})
}

fn setup(mut commands: Commands, mut rapier_config: Query<&mut RapierConfiguration>) {
	commands.spawn((
		BlueprintInfo::from_path("levels/World.glb"),
		SpawnBlueprint,
		HideUntilReady,
	));

	commands.spawn((
		Name::new("Sun"),
		DirectionalLight {
			illuminance: 4000.0,
			shadows_enabled: true,
			..default()
		},
		Transform {
			rotation: Quat::from_euler(EulerRot::XYZ, -1.9, 0.8, 0.0),
			..default()
		},
	));

	rapier_config.single_mut().gravity = Vec3::ZERO;
}

fn quit(mut ev_quit: EventWriter<AppExit>) {
	ev_quit.send(AppExit::Success);
}
