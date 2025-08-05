use bevy::prelude::*;
use bevy_butler::*;
use bevy_marching_cubes::chunk_generator::ChunkMaterial;
use bevy_marching_cubes::{ComputeShader, ShaderRef};

use crate::{gridbox_material, prelude::*};

#[add_plugin(to_plugin = SbepisPlugin, generics = <MyComputeSampler, StandardMaterial>)]
use bevy_marching_cubes::chunk_generator::MarchingCubesPlugin;

#[insert_resource(plugin = SbepisPlugin, init = ChunkGenerator::<MyComputeSampler>::new(0.0, 50, 10.0))]
use bevy_marching_cubes::chunk_generator::ChunkGenerator;

#[derive(TypePath)]
struct MyComputeSampler;
impl ComputeShader for MyComputeSampler {
    fn shader() -> ShaderRef {
        "sample.wgsl".into()
    }
}

#[add_system(plugin = SbepisPlugin, schedule = Startup)]
fn setup_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(ChunkMaterial::<MyComputeSampler, StandardMaterial>::new(
        gridbox_material("white", &mut materials, &asset_server),
    ));
}
