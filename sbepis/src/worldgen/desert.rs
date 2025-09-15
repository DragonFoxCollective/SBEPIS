use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_butler::*;
use bevy_marching_cubes::chunk_generator::{
    ChunkComputeShader, ChunkComputeWorker, ChunkGenSystems, ChunkMaterial,
};
use bevy_marching_cubes::{
    AppComputeWorkerBuilder, Chunk, ComputeShader, ComputeWorker, ShaderRef,
};

use crate::gridbox_material;
use crate::prelude::*;

#[butler_plugin]
#[add_plugin(to_plugin = crate::worldgen::WorldGenPlugin)]
pub struct DesertWorldGenPlugin;

#[add_plugin(to_plugin = DesertWorldGenPlugin, generics = <DesertWorldGen, StandardMaterial>)]
use bevy_marching_cubes::chunk_generator::MarchingCubesPlugin;

#[insert_resource(
	plugin = DesertWorldGenPlugin, generics = <DesertWorldGen>,
	init = ChunkGenerator::<DesertWorldGen>::new(50, 50.0)
		.with_bounds(vec3(-50.0, -50.0, -50.0), vec3(1100.0, 50.0, 1100.0))
)]
use bevy_marching_cubes::chunk_generator::ChunkGenerator;

#[derive(TypePath)]
pub struct DesertWorldGen;
impl ComputeShader for DesertWorldGen {
    fn shader() -> ShaderRef {
        "sample desert.wgsl".into()
    }
}
impl ChunkComputeShader for DesertWorldGen {
    fn build_worker_extra<W: ComputeWorker>(compute_worker: &mut AppComputeWorkerBuilder<W>) {
        compute_worker.add_uniform("poi_positions", &[vec3(-8.0, 0.0, -4.0); 1]);
        compute_worker.add_staging("poi_positions_final", &[Vec3::ZERO; 1]);
    }

    fn extra_sample_bindings() -> Vec<&'static str> {
        vec!["poi_positions", "poi_positions_final"]
    }
}

#[add_system(plugin = DesertWorldGenPlugin, schedule = Startup)]
fn setup_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(ChunkMaterial::<DesertWorldGen, StandardMaterial>::new(
        gridbox_material("yellow", &mut materials, &asset_server),
    ));
}

#[add_system(plugin = DesertWorldGenPlugin, schedule = Update, after = ChunkGenSystems)]
fn add_components(
    mut commands: Commands,
    chunks: Query<Entity, (With<Chunk<DesertWorldGen>>, Without<FinalizedChunk>)>,
) {
    for chunk in chunks.iter() {
        commands
            .entity(chunk)
            .insert((FinalizedChunk, StateScoped(GameState::MainMenu)));
    }
}

#[derive(Component, Debug)]
struct FinalizedChunk;

#[derive(Resource, Debug)]
struct DesertPOIStructures {
    command_station: Handle<Scene>,
}

#[add_system(plugin = DesertWorldGenPlugin, schedule = Startup)]
fn load_poi_structures(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.insert_resource(DesertPOIStructures {
        command_station: asset_server
            .load(GltfAssetLabel::Scene(0).from_asset("command station.glb")),
    });
}

#[add_system(plugin = DesertWorldGenPlugin, schedule = Update, after = ChunkGenSystems)]
fn place_poi_structures(
    compute_worker: Res<ChunkComputeWorker<DesertWorldGen>>,
    mut done: Local<bool>,
    poi_structures: Res<DesertPOIStructures>,
    mut commands: Commands,
) {
    if *done {
        return;
    }

    if !compute_worker.ready() {
        return;
    }

    *done = true;

    let poi_positions = compute_worker
        .read_vec::<Vec4>("poi_positions_final")
        .iter()
        .cloned()
        .map(Vec4::xyz)
        .collect::<Vec<_>>();
    debug!(
        "Reading POI positions from compute worker: {:?}",
        poi_positions
    );

    for position in poi_positions.iter() {
        let poi_structure = &poi_structures.command_station;

        commands.spawn((
            SceneRoot(poi_structure.clone()),
            Transform::from_translation(*position).with_rotation(Quat::from_rotation_y(PI)),
        ));
    }
}
