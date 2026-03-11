use bevy::prelude::*;
use bevy_cube_marcher::{ChunkGeneratorRunning, ChunkGeneratorSettings};

use crate::gravity::{GlobalGravity, GravityPoint, GravityPriority};
use crate::player::PlayerSpawnPoint;
use crate::prelude::GameState;
use crate::worldgen::terrain::WorldGen;

pub const NORMAL_GRAVITY: f32 = 15.0;

pub fn setup_default_planet(
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

    commands.spawn((PlayerSpawnPoint, Transform::from_xyz(0.0, 0.0, 0.0)));
}

pub fn setup_jump_gym(
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

    commands.spawn((PlayerSpawnPoint, Transform::from_xyz(0.0, 0.0, 0.0)));
}
