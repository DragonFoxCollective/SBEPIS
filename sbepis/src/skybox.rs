use std::array::IntoIter;

use bevy::asset::{LoadState, RenderAssetUsages};
use bevy::core_pipeline::Skybox;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDimension, TextureViewDescriptor, TextureViewDimension,
};
use bevy_auto_plugin::prelude::*;

use crate::prelude::*;

#[derive(AutoPlugin)]
#[auto_add_plugin(plugin = SbepisPlugin)]
#[auto_plugin(impl_plugin_trait)]
pub struct SkyboxPlugin;

#[auto_resource(plugin = SkyboxPlugin, derive(Default), reflect, register, init)]
struct CurrentSkybox {
    skybox: Option<Handle<Image>>,
    left: Option<Handle<Image>>,
    right: Option<Handle<Image>>,
    top: Option<Handle<Image>>,
    bottom: Option<Handle<Image>>,
    back: Option<Handle<Image>>,
    front: Option<Handle<Image>>,
}
impl CurrentSkybox {
    pub fn parts(&self) -> IntoIter<Option<Handle<Image>>, 6> {
        [
            self.left.clone(),
            self.right.clone(),
            self.top.clone(),
            self.bottom.clone(),
            self.back.clone(),
            self.front.clone(),
        ]
        .into_iter()
    }
}

fn is_skybox_loaded(current_skybox: Res<CurrentSkybox>) -> bool {
    current_skybox.skybox.is_some()
}
fn is_skybox_parts_loaded(
    current_skybox: Res<CurrentSkybox>,
    asset_server: Res<AssetServer>,
) -> Result<bool> {
    current_skybox
        .parts()
        .map(|image| match image {
            Some(image) => match asset_server.get_load_state(image.id()) {
                Some(LoadState::NotLoaded) => Ok(false),
                Some(LoadState::Loading) => Ok(false),
                Some(LoadState::Loaded) => Ok(true),
                Some(LoadState::Failed(error)) => Err(BevyError::from(error)),
                None => Err(BevyError::from("No skybox image load state found")),
            },
            None => Ok(false),
        })
        .collect::<Result<Vec<bool>>>()
        .map(|states| states.into_iter().all(|loaded| loaded))
}

#[auto_system(plugin = SkyboxPlugin, schedule = Startup)]
fn start_loading_skybox(asset_server: Res<AssetServer>, mut current_skybox: ResMut<CurrentSkybox>) {
    current_skybox.left = Some(asset_server.load("skybox/left.png"));
    current_skybox.right = Some(asset_server.load("skybox/right.png"));
    current_skybox.top = Some(asset_server.load("skybox/top.png"));
    current_skybox.bottom = Some(asset_server.load("skybox/bottom.png"));
    current_skybox.back = Some(asset_server.load("skybox/back.png"));
    current_skybox.front = Some(asset_server.load("skybox/front.png"));
}

#[auto_system(plugin = SkyboxPlugin, schedule = Update, config(
	run_if = not(is_skybox_loaded).and(is_skybox_parts_loaded),
))]
fn stitch_skybox(
    mut images: ResMut<Assets<Image>>,
    mut current_skybox: ResMut<CurrentSkybox>,
) -> Result {
    let sides = current_skybox
        .parts()
        .map(|side| images.get(side?.id()))
        .collect::<Option<Vec<&Image>>>()
        .ok_or("Side images not found")?;
    let first_side_image = *sides.first().ok_or("Side images not found")?;

    let mut skybox = Image::new(
        Extent3d {
            width: first_side_image.texture_descriptor.size.width,
            height: first_side_image.texture_descriptor.size.width * 6,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        sides
            .into_iter()
            .map(|texture| {
                texture
                    .data
                    .clone()
                    .ok_or(BevyError::from("No image data found"))
            })
            .collect::<Result<Vec<Vec<u8>>>>()?
            .into_iter()
            .flatten()
            .collect(),
        first_side_image.texture_descriptor.format,
        RenderAssetUsages::RENDER_WORLD,
    );
    skybox.reinterpret_stacked_2d_as_array(6)?;
    skybox.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..default()
    });

    current_skybox.skybox = Some(images.add(skybox));

    Ok(())
}

#[auto_system(plugin = SkyboxPlugin, schedule = Update, config(
	run_if = is_skybox_loaded.and(in_state(GameState::InGame)),
))]
fn add_skybox(
    mut commands: Commands,
    camera: Query<Entity, (With<Camera3d>, Without<Skybox>)>,
    current_skybox: Res<CurrentSkybox>,
) -> Result {
    for camera in camera.iter() {
        commands.entity(camera).insert(Skybox {
            image: current_skybox
                .skybox
                .clone()
                .ok_or("Skybox wasn't set up")?,
            brightness: 1000.0,
            ..default()
        });
    }
    Ok(())
}
