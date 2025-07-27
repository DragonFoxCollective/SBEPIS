use bevy::color::palettes::css;
use bevy::color::palettes::tailwind;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::prelude::*;
use fastnoise_lite::FastNoiseLite;
use marching_cubes::chunk_generator::ChunkGenerator;
use marching_cubes::terrain_sampler::NoiseTerrainSampler;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            WireframePlugin,
            bevy_inspector_egui::quick::WorldInspectorPlugin::new(),
            bevy_panorbit_camera::PanOrbitCameraPlugin,
        ))
        .insert_resource(WireframeConfig {
            global: true,
            default_color: css::WHITE.into(),
        })
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Name::new("Camera"),
        Camera3dBundle {
            transform: Transform::from_xyz(4.0, 6.5, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        bevy_panorbit_camera::PanOrbitCamera {
            button_orbit: MouseButton::Left,
            button_pan: MouseButton::Left,
            modifier_pan: Some(KeyCode::ShiftLeft),
            reversed_zoom: true,
            ..default()
        },
    ));

    commands.spawn((
        Name::new("Light"),
        PointLightBundle {
            transform: Transform::from_xyz(4.0, 8.0, 4.0),
            ..default()
        },
    ));

    let mut noise = FastNoiseLite::with_seed(1);
    noise.set_frequency(Some(0.05));
    let chunk_generator = ChunkGenerator::<NoiseTerrainSampler> {
        surface_threshold: 0.5,
        num_voxels: 32,
        terrain_sampler: NoiseTerrainSampler(noise),
    };
    let mesh = chunk_generator.generate_chunk();

    let size = 8.0;
    commands.spawn((
        Name::new("MarchingCubesMesh"),
        PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(Color::from(tailwind::EMERALD_500)),
            transform: Transform::from_translation(Vec3::splat(-size / 2.0))
                .with_scale(Vec3::splat(size / chunk_generator.num_voxels as f32)),
            ..default()
        },
    ));
}
