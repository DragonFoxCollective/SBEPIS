use bevy::prelude::*;
use bevy_butler::*;
use bevy_marching_cubes::chunk_generator::{
    ChunkComputeShader, ChunkComputeWorker, ChunkGenSystems, ChunkGeneratorCache, ChunkMaterial,
};
use bevy_marching_cubes::{
    AppComputeWorkerBuilder, Chunk, ComputeShader, ComputeWorker, ShaderRef,
};
use bevy_rapier3d::prelude::{Collider, ComputedColliderShape, RigidBody, TriMeshFlags};
use rand::{Rng, SeedableRng as _};

use crate::gridbox_material;
use crate::prelude::*;

#[butler_plugin]
#[add_plugin(to_plugin = crate::worldgen::WorldGenPlugin)]
pub struct TerrainWorldGenPlugin;

#[add_plugin(to_plugin = TerrainWorldGenPlugin, generics = <WorldGen, StandardMaterial>)]
use bevy_marching_cubes::chunk_generator::MarchingCubesPlugin;

#[insert_resource(
	plugin = TerrainWorldGenPlugin, generics = <WorldGen>,
	init = ChunkGeneratorSettings::<WorldGen>::new(50, 50.0)
		.with_bounds(vec3(-1100.0, -2100.0, -1100.0), vec3(1100.0, 100.0, 1100.0))
)]
use bevy_marching_cubes::chunk_generator::ChunkGeneratorSettings;

#[derive(TypePath)]
pub struct WorldGen;
impl ComputeShader for WorldGen {
    fn shader() -> ShaderRef {
        "sample.wgsl".into()
    }
}
impl ChunkComputeShader for WorldGen {
    fn build_worker_extra<W: ComputeWorker>(compute_worker: &mut AppComputeWorkerBuilder<W>) {
        let radius = 200.0;
        let mut rand = rand::prelude::StdRng::seed_from_u64(159);
        let poi_positions = [Vec3::ZERO; 6].map(|_| {
            Vec3::new(
                rand.gen_range(-radius..radius),
                0.0,
                rand.gen_range(-radius..radius),
            )
        });
        debug!("Generated POI positions: {:?}", poi_positions);
        compute_worker.add_uniform("poi_positions", &poi_positions);
        compute_worker.add_staging("poi_positions_final", &[Vec3::ZERO; 6]);
    }

    fn extra_sample_bindings() -> Vec<&'static str> {
        vec!["poi_positions", "poi_positions_final"]
    }
}

#[add_system(plugin = TerrainWorldGenPlugin, schedule = Startup)]
fn setup_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(ChunkMaterial::<WorldGen, StandardMaterial>::new(
        gridbox_material("grey2", &mut materials, &asset_server),
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
        commands.entity(chunk).insert((
            Collider::from_bevy_mesh(mesh, &ComputedColliderShape::TriMesh(TriMeshFlags::empty()))
                .expect("Failed to create chunk collider"),
            StateScoped(GameState::InGame),
        ));
    }
}

#[derive(Component, Debug)]
struct FinalizedChunk;

#[derive(Component, Debug)]
struct SleepingFromUnloaded;

#[add_system(plugin = TerrainWorldGenPlugin, schedule = Update, after = ChunkGenSystems, run_if = in_state(GameState::InGame))]
fn sleep_unloaded_entities(
    mut commands: Commands,
    sleeping_entities: Query<(Entity, &GlobalTransform, &RigidBody), Without<SleepingFromUnloaded>>,
    settings: Res<ChunkGeneratorSettings<WorldGen>>,
    cache: Res<ChunkGeneratorCache<WorldGen>>,
) {
    for (entity, transform, rigidbody) in sleeping_entities.iter() {
        if !cache.is_chunk_with_position_generated(&settings, transform.translation())
            && *rigidbody == RigidBody::Dynamic
        {
            commands
                .entity(entity)
                .insert(RigidBody::Fixed)
                .insert(SleepingFromUnloaded);
        }
    }
}

#[add_system(plugin = TerrainWorldGenPlugin, schedule = Update, after = ChunkGenSystems, run_if = in_state(GameState::InGame))]
fn wake_loaded_entities(
    mut commands: Commands,
    sleeping_entities: Query<(Entity, &GlobalTransform), With<SleepingFromUnloaded>>,
    settings: Res<ChunkGeneratorSettings<WorldGen>>,
    cache: Res<ChunkGeneratorCache<WorldGen>>,
) {
    for (entity, transform) in sleeping_entities.iter() {
        if cache.is_chunk_with_position_generated(&settings, transform.translation()) {
            commands
                .entity(entity)
                .insert(RigidBody::Dynamic)
                .remove::<SleepingFromUnloaded>();
        }
    }
}

#[derive(Resource, Debug)]
struct POIStructures {
    consort_village: Handle<Scene>,
    imp_arena: Handle<Scene>,
}

#[add_system(plugin = TerrainWorldGenPlugin, schedule = Startup)]
fn load_poi_structures(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.insert_resource(POIStructures {
        consort_village: asset_server
            .load(GltfAssetLabel::Scene(0).from_asset("consort village.glb")),
        imp_arena: asset_server.load(GltfAssetLabel::Scene(0).from_asset("imp arena.glb")),
    });
}

#[add_system(plugin = TerrainWorldGenPlugin, schedule = Update, after = ChunkGenSystems)]
fn place_poi_structures(
    compute_worker: Res<ChunkComputeWorker<WorldGen>>,
    mut done: Local<bool>,
    poi_structures: Res<POIStructures>,
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

    for (i, position) in poi_positions.iter().enumerate() {
        let poi_structure = match i % 2 {
            0 => &poi_structures.consort_village,
            1 => &poi_structures.imp_arena,
            _ => continue,
        };

        commands.spawn((
            SceneRoot(poi_structure.clone()),
            Transform::from_translation(*position).with_rotation(Quat::from_rotation_arc(
                Vec3::Y,
                (*position - Vec3::NEG_Y * 1000.0).normalize(),
            )),
            StateScoped(GameState::InGame),
        ));
    }
}

#[add_system(plugin = TerrainWorldGenPlugin, schedule = OnEnter(GameState::InGame))]
fn add_cache(mut commands: Commands) {
    commands.init_resource::<ChunkGeneratorCache<WorldGen>>();
}

#[add_system(plugin = TerrainWorldGenPlugin, schedule = OnExit(GameState::InGame))]
fn remove_cache(mut commands: Commands) {
    commands.remove_resource::<ChunkGeneratorCache<WorldGen>>();
}
