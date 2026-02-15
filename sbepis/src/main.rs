#![cfg_attr(not(feature = "terminal"), windows_subsystem = "windows")]

use std::io::Cursor;

use bevy::gltf::GltfPlugin;
use bevy::gltf::convert_coordinates::GltfConvertCoordinates;
use bevy::input::common_conditions::input_just_pressed;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_marching_cubes::{ChunkGeneratorRunning, ChunkGeneratorSettings};
use bevy_rapier3d::prelude::*;
use winit::window::Icon;

use crate::gravity::{GlobalGravity, GravityPoint, GravityPriority};
use crate::prelude::GameState;
use crate::worldgen::terrain::WorldGen;

mod blenvy;
mod camera;
mod dialogue;
mod entity;
#[cfg(feature = "framerate_indicator")]
mod framerate;
mod fray;
mod gravity;
mod inventory;
mod main_bundles;
mod main_menu;
mod npcs;
#[cfg(feature = "overview_camera")]
mod overview_camera;
mod player_commands;
mod player_controller;
mod post_processing;
mod questing;
mod skybox;
pub mod util;
mod worldgen;

mod prelude {
    #![allow(unused_imports)]
    pub use crate::SbepisPlugin;
    pub use crate::camera::PlayerCameraNode;
    pub use crate::main_menu::GameState;
    pub use crate::player_controller::PlayerBody;
    pub use crate::player_controller::camera_controls::{InteractWith, interact_with};
    pub use crate::post_processing::outlines::PostProcessOutlinesSettings;
    pub use crate::post_processing::quantize::PostProcessQuantizeSettings;
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
			filter: "info,sbepis=debug,avian3d=debug,wgpu=error,naga=warn,calloop=error,symphonia_core=warn,symphonia_bundle_mp3=warn,blenvy=error,bevy_pretty_nice_input=debug,bevy_pretty_nice_menus=debug".into(),
			..default()
		})
		.set(GltfPlugin {
		    convert_coordinates: GltfConvertCoordinates {
		        rotate_scene_entity: true,
				..default()
			},
			..default()
		})).add_plugins(SbepisPlugin).run();
}

#[derive(AutoPlugin)]
pub struct SbepisPlugin;

#[auto_plugin(plugin = SbepisPlugin)]
fn build(app: &mut App) {
    app.add_plugins(bevy_rapier3d::prelude::RapierPhysicsPlugin::<NoUserData>::default());
    #[cfg(feature = "rapier_debug")]
    app.add_plugins(bevy_rapier3d::prelude::RapierDebugRenderPlugin::default());

    #[cfg(feature = "inspector")]
    {
        app.insert_resource(bevy_inspector_egui::bevy_egui::EguiGlobalSettings {
            auto_create_primary_context: false,
            ..default()
        });
        app.add_plugins((
            bevy_inspector_egui::bevy_egui::EguiPlugin::default(),
            bevy_inspector_egui::quick::WorldInspectorPlugin::default(),
        ));
    }

    app.add_plugins(bevy_hanabi::HanabiPlugin);

    app.add_plugins((
        bevy_pretty_nice_input::PrettyNiceInputPlugin,
        bevy_pretty_nice_menus::PrettyNiceMenusPlugin,
    ));
}

#[auto_system(plugin = SbepisPlugin, schedule = Startup)]
fn set_window_icon() -> Result {
    let icon_buf = Cursor::new(include_bytes!("../assets/house.png"));
    let image = image::load(icon_buf, image::ImageFormat::Png)?;
    let image = image.into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    let icon = Icon::from_rgba(rgba, width, height)?;

    bevy::winit::WINIT_WINDOWS.with_borrow_mut(|windows| {
        for window in windows.windows.values() {
            window.set_window_icon(Some(icon.clone()));
        }
    });

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

#[auto_system(plugin = SbepisPlugin, schedule = Startup)]
fn setup_global(mut rapier_config: Query<&mut RapierConfiguration>) -> Result {
    rapier_config.single_mut()?.gravity = Vec3::ZERO;
    Ok(())
}

const NORMAL_GRAVITY: f32 = 15.0;

fn setup_default_planet(
    _click: On<Pointer<Click>>,
    mut commands: Commands,
    mut settings: ResMut<ChunkGeneratorSettings<WorldGen>>,
) {
    settings.running = ChunkGeneratorRunning::Run;

    let planet_radius = 1000.0;
    commands.spawn((
        Name::new("Gravity"),
        Transform::from_translation(Vec3::NEG_Y * planet_radius),
        GravityPoint {
            standard_radius: planet_radius,
            acceleration_at_radius: NORMAL_GRAVITY,
            has_volume: true,
        },
        GravityPriority(0),
        DespawnOnExit(GameState::InGame),
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
        DespawnOnExit(GameState::InGame),
    ));
}

fn setup_jump_gym(
    _click: On<Pointer<Click>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let gym_scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset("jump gym.glb"));
    commands.spawn((
        Name::new("Gym"),
        SceneRoot(gym_scene),
        DespawnOnExit(GameState::InGame),
    ));

    commands.spawn((
        Name::new("Gravity"),
        GlobalGravity {
            acceleration: NORMAL_GRAVITY * Vec3::NEG_Y,
        },
        GravityPriority(0),
        DespawnOnExit(GameState::InGame),
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
        DespawnOnExit(GameState::InGame),
    ));
}

#[auto_system(plugin = SbepisPlugin, schedule = Update, config(
	run_if = input_just_pressed(KeyCode::Escape)
))]
fn exit(mut exit: MessageWriter<AppExit>) {
    exit.write(AppExit::Success);
}
