// Most of this code is from https://github.com/SebLague/Terraforming

use bevy::color::palettes::css;
use bevy::color::palettes::tailwind;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::prelude::*;
use bevy::utils::HashMap;
use marching_cubes::*;

mod march_tables;
mod marching_cubes;

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

    // commands.spawn((
    // 	Name::new("Cube"),
    // 	PbrBundle {
    // 		mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
    // 		material: materials.add(Color::from(palettes::tailwind::SKY_500)),
    // 		..default()
    // 	},
    // ));

    commands.spawn((
        Name::new("Light"),
        PointLightBundle {
            transform: Transform::from_xyz(4.0, 8.0, 4.0),
            ..default()
        },
    ));

    let mut triangles = Vec::<Triangle>::new();
    for x in 0..NUM_VOXELS {
        for y in 0..NUM_VOXELS {
            for z in 0..NUM_VOXELS {
                triangles.extend(process_cube(IVec3::new(x, y, z)));
            }
        }
    }

    let vertex_data: Vec<Vertex> = triangles
        .iter()
        .flat_map(|t| [t.vertex_c.clone(), t.vertex_b.clone(), t.vertex_a.clone()])
        .collect();

    let mut vertex_positions: Vec<Vec3> = vec![];
    let mut vertex_normals: Vec<Vec3> = vec![];
    let mut indices: Vec<u32> = vec![];
    let mut vertex_index_map: HashMap<IVec2, u32> = HashMap::new();

    let mut vertex_index = 0;
    for data in vertex_data.iter() {
        if let Some(shared_vertex_index) = vertex_index_map.get(&data.id) {
            indices.push(*shared_vertex_index);
        } else {
            vertex_index_map.insert(data.id, vertex_index);
            vertex_positions.push(data.position);
            vertex_normals.push(-data.normal);
            indices.push(vertex_index);
            vertex_index += 1;
        }
    }

    let mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_indices(bevy::render::mesh::Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertex_positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vertex_normals);

    let size = 8.0;
    commands.spawn((
        Name::new("MarchingCubesMesh"),
        PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(Color::from(tailwind::EMERALD_500)),
            transform: Transform::from_translation(Vec3::splat(-size / 2.0))
                .with_scale(Vec3::splat(size / NUM_VOXELS as f32)),
            ..default()
        },
    ));
}
