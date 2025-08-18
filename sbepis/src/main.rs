#![cfg_attr(not(feature = "terminal"), windows_subsystem = "windows")]

use std::io::Cursor;

use bevy::input::common_conditions::input_just_pressed;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::winit::WinitWindows;
use bevy_butler::*;
use bevy_rapier3d::prelude::*;
use winit::window::Icon;

mod blenvy;
mod camera;
mod dialogue;
mod entity;
#[cfg(feature = "framerate_indicator")]
mod framerate;
mod fray;
mod gravity;
mod input;
mod inventory;
mod main_bundles;
mod main_menu;
pub mod menus;
mod npcs;
#[cfg(feature = "overview_camera")]
mod overview_camera;
mod player_commands;
mod player_controller;
mod questing;
mod skybox;
pub mod util;
mod worldgen;

mod prelude {
    #![allow(unused_imports)]
    pub use crate::SbepisPlugin;
    pub use crate::camera::PlayerCameraNode;
    pub use crate::player_controller::PlayerBody;
    pub use crate::player_controller::camera_controls::{
        InteractedWith, InteractedWithSet, interact_with,
    };
}

fn main() {
    App::new().add_plugins(DefaultPlugins
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
		})).add_plugins(SbepisPlugin).run();
}

#[butler_plugin]
pub struct SbepisPlugin;

#[add_plugin(to_plugin = crate::SbepisPlugin, generics = <NoUserData>)]
use bevy_rapier3d::prelude::RapierPhysicsPlugin;

#[cfg(feature = "rapier_debug")]
#[add_plugin(to_plugin = crate::SbepisPlugin)]
use bevy_rapier3d::prelude::RapierDebugRenderPlugin;

#[cfg(feature = "inspector")]
#[add_plugin(to_plugin = crate::SbepisPlugin)]
use bevy_inspector_egui::bevy_egui::EguiPlugin;

#[cfg(feature = "inspector")]
#[add_plugin(to_plugin = crate::SbepisPlugin)]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

#[add_plugin(to_plugin = crate::SbepisPlugin, init = HanabiPlugin)]
use bevy_hanabi::HanabiPlugin;

#[add_system(
	plugin = SbepisPlugin, schedule = Startup,
)]
fn set_window_icon(windows: NonSend<WinitWindows>) -> Result {
    let icon_buf = Cursor::new(include_bytes!("../assets/house.png"));
    let image = image::load(icon_buf, image::ImageFormat::Png)?;
    let image = image.into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    let icon = Icon::from_rgba(rgba, width, height)?;

    for window in windows.windows.values() {
        window.set_window_icon(Some(icon.clone()));
    }

    Ok(())
}

fn gridbox_texture(color: &str) -> String {
    format!("Gridbox Prototype Materials/prototype_512x512_{color}.png")
}

fn gridbox_material(
    color: &str,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
) -> Handle<StandardMaterial> {
    materials.add(gridbox_material_direct(color, asset_server))
}

fn gridbox_material_extra(
    color: &str,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
    material: StandardMaterial,
) -> Handle<StandardMaterial> {
    materials.add(gridbox_material_direct_extra(color, asset_server, material))
}

fn gridbox_material_direct(color: &str, asset_server: &AssetServer) -> StandardMaterial {
    gridbox_material_direct_extra(color, asset_server, StandardMaterial::default())
}

fn gridbox_material_direct_extra(
    color: &str,
    asset_server: &AssetServer,
    material: StandardMaterial,
) -> StandardMaterial {
    StandardMaterial {
        base_color_texture: Some(asset_server.load(gridbox_texture(color))),
        ..material
    }
}

#[add_system(
	plugin = SbepisPlugin, schedule = Startup,
)]
fn setup(mut commands: Commands, mut rapier_config: Query<&mut RapierConfiguration>) -> Result {
    commands.spawn((
        Name::new("Gravity"),
        Transform::from_translation(Vec3::NEG_Y * 1000.0),
        GravityPoint {
            standard_radius: 1000.0,
            acceleration_at_radius: 15.0,
        },
        GravityPriority(0),
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

    rapier_config.single_mut()?.gravity = Vec3::ZERO;

    Ok(())
}

#[add_system(
	plugin = SbepisPlugin, schedule = Update,
	run_if = input_just_pressed(KeyCode::Escape),
)]
fn quit(mut ev_quit: EventWriter<AppExit>) {
    ev_quit.write(AppExit::Success);
}

use crate::gravity::{GravityPoint, GravityPriority};
#[add_system(
	plugin = SbepisPlugin, schedule = Update,
)]
use crate::util::despawn_after_timer;

#[add_system(
	plugin = SbepisPlugin, schedule = Update,
)]
use crate::util::billboard;
