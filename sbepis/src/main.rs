#![cfg_attr(not(feature = "terminal"), windows_subsystem = "windows")]

use std::io::Cursor;

use bevy::gltf::convert_coordinates::GltfConvertCoordinates;
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_rapier3d::prelude::*;
use winit::window::Icon;

mod blenvy;
mod commands;
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
mod player;
mod post_processing;
mod questing;
mod skybox;
#[cfg(test)]
pub mod test;
pub mod util;
mod worldgen;
mod worlds;

mod prelude {
    #![allow(unused_imports)]
    pub use crate::main_menu::GameState;
    pub use crate::player::Player;
    pub use crate::player::camera::node::PlayerCameraNode;
    pub use crate::player::interaction::{InteractWith, interact_with};
    pub use crate::post_processing::outlines::PostProcessOutlinesSettings;
    pub use crate::post_processing::quantize::PostProcessQuantizeSettings;
    #[cfg(test)]
    pub use crate::test::{TestAppExt, assert_near_f32, assert_near_vec3, new_test_app};
    pub use crate::util::*;
    pub use crate::worlds::NORMAL_GRAVITY;
    pub use crate::{SbepisAppPlugin, SbepisPlugin};
}

fn main() {
    App::new()
        .add_plugins((SbepisAppPlugin { headless: false }, SbepisPlugin))
        .run();
}

#[derive(AutoPlugin)]
#[auto_plugin(impl_plugin_trait)]
pub struct SbepisPlugin;

/// Everything required for running or testing
#[derive(AutoPlugin)]
pub struct SbepisAppPlugin {
    headless: bool,
}

impl Plugin for SbepisAppPlugin {
    #[auto_plugin]
    fn build(&self, app: &mut App) {
        app.add_plugins(
            if self.headless {
                MinimalPlugins
                    .build()
                    .add_before::<bevy::app::TaskPoolPlugin>(bevy::log::LogPlugin::default())
                    .add(bevy::transform::TransformPlugin)
                    .add(bevy::input::InputPlugin)
                    .add(bevy::asset::AssetPlugin::default())
                    .add(bevy::scene::ScenePlugin)
                    .add(bevy::image::ImagePlugin::default())
                    .add(bevy::gltf::GltfPlugin::default())
                    .add(bevy::animation::AnimationPlugin)
                    .add(bevy::state::app::StatesPlugin)
            } else {
                DefaultPlugins.set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "SBEPIS".to_string(),
                        ..default()
                    }),
                    ..default()
                })
            }
            .set(ImagePlugin {
                default_sampler: bevy::image::ImageSamplerDescriptor {
                    address_mode_u: bevy::image::ImageAddressMode::Repeat,
                    address_mode_v: bevy::image::ImageAddressMode::Repeat,
                    address_mode_w: bevy::image::ImageAddressMode::Repeat,
                    ..default()
                },
            })
            .set(bevy::log::LogPlugin {
                filter: [
                    "info",
                    "sbepis=debug",
                    "avian3d=debug",
                    "wgpu=error",
                    "naga=warn",
                    "calloop=error",
                    "symphonia_core=warn",
                    "symphonia_bundle_mp3=warn",
                    "blenvy=error",
                    "bevy_pretty_nice_input=debug",
                    "bevy_pretty_nice_menus=debug",
                ]
                .join(","),
                ..default()
            })
            .set(bevy::gltf::GltfPlugin {
                convert_coordinates: GltfConvertCoordinates {
                    rotate_scene_entity: true,
                    ..default()
                },
                ..default()
            }),
        );

        if self.headless {
            // for now we need these as assets, even if we don't need the plugins
            app.init_asset::<Mesh>()
                .init_asset::<StandardMaterial>()
                .init_asset::<AudioSource>();
        }

        app.add_plugins(bevy_rapier3d::prelude::RapierPhysicsPlugin::<NoUserData>::default());
        #[cfg(feature = "rapier_debug")]
        if !self.headless {
            app.add_plugins(bevy_rapier3d::prelude::RapierDebugRenderPlugin::default());
        }

        #[cfg(feature = "inspector")]
        if !self.headless {
            app.insert_resource(bevy_inspector_egui::bevy_egui::EguiGlobalSettings {
                auto_create_primary_context: false,
                ..default()
            });
            app.add_plugins((
                bevy_inspector_egui::bevy_egui::EguiPlugin::default(),
                bevy_inspector_egui::quick::WorldInspectorPlugin::default(),
            ));
        }

        if !self.headless {
            app.add_plugins(bevy_hanabi::HanabiPlugin);
        }

        app.add_plugins((
            bevy_pretty_nice_input::PrettyNiceInputPlugin,
            bevy_pretty_nice_menus::PrettyNiceMenusPlugin,
        ));
    }
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

#[auto_system(plugin = SbepisPlugin, schedule = Update, config(
	run_if = input_just_pressed(KeyCode::Escape)
))]
fn exit(mut exit: MessageWriter<AppExit>) {
    exit.write(AppExit::Success);
}
