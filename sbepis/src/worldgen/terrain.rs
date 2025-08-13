use bevy::prelude::*;
use bevy_butler::*;
use bevy_marching_cubes::chunk_generator::{ChunkGenSystems, ChunkMaterial};
use bevy_marching_cubes::{Chunk, ComputeShader, ShaderRef};
use bevy_rapier3d::prelude::{Collider, ComputedColliderShape, RigidBody, TriMeshFlags};

use crate::gridbox_material;

#[butler_plugin]
#[add_plugin(to_plugin = crate::worldgen::WorldGenPlugin)]
pub struct TerrainWorldGenPlugin;

#[add_plugin(to_plugin = TerrainWorldGenPlugin, generics = <WorldGen, StandardMaterial>)]
use bevy_marching_cubes::chunk_generator::MarchingCubesPlugin;

#[insert_resource(plugin = TerrainWorldGenPlugin, generics = <WorldGen>, init = ChunkGenerator::<WorldGen>::new(0.0, 50, 50.0))]
use bevy_marching_cubes::chunk_generator::ChunkGenerator;

#[derive(TypePath)]
pub struct WorldGen;
impl ComputeShader for WorldGen {
    fn shader() -> ShaderRef {
        "sample.wgsl".into()
    }
}

#[add_system(plugin = TerrainWorldGenPlugin, schedule = Startup)]
fn setup_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(ChunkMaterial::<WorldGen, StandardMaterial>::new(
        gridbox_material("white", &mut materials, &asset_server),
    ));
}

#[add_system(plugin = TerrainWorldGenPlugin, schedule = Update, after = ChunkGenSystems)]
fn add_components(
    mut commands: Commands,
    chunks: Query<(Entity, &Mesh3d), (With<Chunk<WorldGen>>, Without<FinalizedChunk>)>,
    meshes: Res<Assets<Mesh>>,
) {
    for (chunk, mesh) in chunks.iter() {
        commands.entity(chunk).insert(FinalizedChunk);

        let mesh = meshes.get(mesh).expect("Failed to get mesh");
        commands.entity(chunk).insert(
            Collider::from_bevy_mesh(mesh, &ComputedColliderShape::TriMesh(TriMeshFlags::empty()))
                .expect("Failed to create chunk collider"),
        );
    }
}

#[derive(Component, Debug)]
struct FinalizedChunk;

#[derive(Component, Debug)]
struct SleepingFromUnloaded;

#[add_system(plugin = TerrainWorldGenPlugin, schedule = Update, after = ChunkGenSystems)]
fn sleep_unloaded_entities(
    mut commands: Commands,
    sleeping_entities: Query<(Entity, &GlobalTransform, &RigidBody), Without<SleepingFromUnloaded>>,
    chunks: Res<ChunkGenerator<WorldGen>>,
) {
    for (entity, transform, rigidbody) in sleeping_entities.iter() {
        if !chunks.is_chunk_with_position_generated(transform.translation())
            && *rigidbody == RigidBody::Dynamic
        {
            commands
                .entity(entity)
                .insert(RigidBody::Fixed)
                .insert(SleepingFromUnloaded);
        }
    }
}

#[add_system(plugin = TerrainWorldGenPlugin, schedule = Update, after = ChunkGenSystems)]
fn wake_loaded_entities(
    mut commands: Commands,
    sleeping_entities: Query<(Entity, &GlobalTransform), With<SleepingFromUnloaded>>,
    chunks: Res<ChunkGenerator<WorldGen>>,
) {
    for (entity, transform) in sleeping_entities.iter() {
        if chunks.is_chunk_with_position_generated(transform.translation()) {
            commands
                .entity(entity)
                .insert(RigidBody::Dynamic)
                .remove::<SleepingFromUnloaded>();
        }
    }
}
