use bevy::pbr::{ExtendedMaterial, MaterialExtension, NotShadowCaster};
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy_butler::*;
use bevy_marching_cubes::chunk_generator::{ChunkComputeShader, ChunkGenSystems, ChunkMaterial};

#[butler_plugin]
// #[add_plugin(to_plugin = crate::worldgen::WorldGenPlugin)]
pub struct LowLODWorldGenPlugin;

#[add_plugin(to_plugin = LowLODWorldGenPlugin, generics = <LowLODWorldGen, LODMaterial>)]
use bevy_marching_cubes::chunk_generator::MarchingCubesPlugin;

#[insert_resource(plugin = LowLODWorldGenPlugin, generics = <LowLODWorldGen>, init = ChunkGenerator::<LowLODWorldGen>::new(0.0, 50, 100.0))]
use bevy_marching_cubes::chunk_generator::ChunkGenerator;

#[derive(TypePath)]
pub struct LowLODWorldGen;
impl ComputeShader for LowLODWorldGen {
    fn shader() -> ShaderRef {
        "sample.wgsl".into()
    }
}
impl ChunkComputeShader for LowLODWorldGen {}

#[add_system(plugin = LowLODWorldGenPlugin, schedule = Startup)]
fn setup_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<LODMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(ChunkMaterial::<LowLODWorldGen, LODMaterial>::new(
        materials.add(ExtendedMaterial {
            base: gridbox_material_direct("grey2", &asset_server),
            extension: LODMaterialExtension {
                cull_distance: 100.0,
                depth_bias_distance: 100.0,
            },
        }),
    ));
}

#[add_system(plugin = LowLODWorldGenPlugin, schedule = Update, after = ChunkGenSystems)]
fn add_not_shadow_caster(
    mut commands: Commands,
    chunks: Query<Entity, (With<Chunk<LowLODWorldGen>>, Without<NotShadowCaster>)>,
) {
    for chunk in chunks.iter() {
        commands.entity(chunk).insert(NotShadowCaster);
    }
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
struct LODMaterialExtension {
    // We need to ensure that the bindings of the base material and the extension do not conflict,
    // so we start from binding slot 100, leaving slots 0-99 for the base material.
    #[uniform(100)]
    cull_distance: f32,
    #[uniform(100)]
    depth_bias_distance: f32,
}

impl MaterialExtension for LODMaterialExtension {
    fn vertex_shader() -> ShaderRef {
        "lod_chunks.vert.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "lod_chunks.frag.wgsl".into()
    }
}

type LODMaterial = ExtendedMaterial<StandardMaterial, LODMaterialExtension>;

#[add_plugin(to_plugin = LowLODWorldGenPlugin, generics = <LODMaterial>)]
use bevy::prelude::MaterialPlugin;
use bevy_marching_cubes::{Chunk, ComputeShader, ShaderRef};

use crate::gridbox_material_direct;
