use bevy::ecs::schedule::ScheduleConfigs;
use bevy::ecs::system::ScheduleSystem;
use bevy::prelude::*;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::gpu_readback::{Readback, ReadbackComplete};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::binding_types::{storage_buffer, uniform_buffer};
use bevy::render::render_resource::{BindGroupLayoutEntryBuilder, BufferUsages, UniformBuffer};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::storage::{GpuShaderStorageBuffer, ShaderStorageBuffer};
use bevy_auto_plugin::prelude::*;
use bevy_marching_cubes::*;
use bevy_rapier3d::prelude::{Collider, ComputedColliderShape, RigidBody, TriMeshFlags};
use rand::{Rng, SeedableRng as _};

use crate::gridbox_material;
use crate::prelude::*;

#[derive(AutoPlugin)]
#[auto_add_plugin(plugin = crate::worldgen::WorldGenPlugin)]
pub struct TerrainWorldGenPlugin;

#[auto_plugin(plugin = TerrainWorldGenPlugin)]
fn build(app: &mut App) {
    app.add_plugins((
        MarchingCubesPlugin::<WorldGen, StandardMaterial>::default(),
        ExtractResourcePlugin::<Poi>::default(),
    ))
    .insert_resource(
        ChunkGeneratorSettings::<WorldGen>::new(50, 50.0)
            .with_bounds(vec3(-1100.0, -2100.0, -1100.0), vec3(1100.0, 100.0, 1100.0)),
    );
}

pub struct WorldGen;
impl ChunkComputeShader for WorldGen {
    fn shader_path() -> String {
        "sample.wgsl".into()
    }

    fn prepare_extra_buffers() -> ScheduleConfigs<ScheduleSystem> {
        IntoSystem::into_system(
            |render_device: Res<RenderDevice>,
             render_queue: Res<RenderQueue>,
             chunks: Query<Entity, With<ChunkRenderData<WorldGen>>>,
             mut commands: Commands,
             poi: Res<Poi>,
             buffers: Res<RenderAssets<GpuShaderStorageBuffer>>| {
                let mut poi_positions_buffer = UniformBuffer::from(&poi.positions);
                poi_positions_buffer.write_buffer(&render_device, &render_queue);

                for chunk in chunks.iter() {
                    commands.entity(chunk).insert(ChunkRenderExtraBuffers {
                        buffers: vec![
                            poi_positions_buffer.buffer().unwrap().clone(),
                            buffers.get(&poi.positions_final).unwrap().buffer.clone(),
                        ],
                    });
                }
            },
        )
        .into_configs()
    }

    fn define_extra_buffers() -> Vec<BindGroupLayoutEntryBuilder> {
        vec![
            uniform_buffer::<[Vec3; 6]>(false),
            storage_buffer::<[Vec3; 6]>(false),
        ]
    }
}

#[auto_resource(plugin = TerrainWorldGenPlugin, derive(ExtractResource, Clone), reflect)]
pub struct Poi {
    positions: [Vec3; 6],
    positions_final: Handle<ShaderStorageBuffer>,
}

#[auto_system(plugin = TerrainWorldGenPlugin, schedule = Startup)]
fn setup_poi(mut commands: Commands, mut buffers: ResMut<Assets<ShaderStorageBuffer>>) {
    let radius = 200.0;
    let mut rand = rand::prelude::StdRng::seed_from_u64(159);
    let poi_positions = [Vec3::ZERO; 6].map(|_| {
        Vec3::new(
            rand.random_range(-radius..radius),
            0.0,
            rand.random_range(-radius..radius),
        )
    });
    debug!("Generated POI positions: {:?}", poi_positions);

    let mut poi_positions_final_buffer = ShaderStorageBuffer::from([Vec3::ZERO; 6]);
    poi_positions_final_buffer.buffer_description.usage |= BufferUsages::COPY_SRC;
    commands.insert_resource(Poi {
        positions: poi_positions,
        positions_final: buffers.add(poi_positions_final_buffer),
    });
}

#[auto_system(plugin = TerrainWorldGenPlugin, schedule = Startup)]
fn setup_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(ChunkMaterial::<WorldGen, StandardMaterial>::new(
        gridbox_material("grey2", &mut materials, &asset_server),
    ));
}

#[auto_system(plugin = TerrainWorldGenPlugin, schedule = Update, config(
	after = ChunkGenSystems,
))]
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
            DespawnOnExit(GameState::InGame),
        ));
    }
}

#[auto_component(plugin = TerrainWorldGenPlugin, derive(Debug), reflect, register)]
struct FinalizedChunk;

#[auto_component(plugin = TerrainWorldGenPlugin, derive(Debug), reflect, register)]
struct SleepingFromUnloaded;

#[auto_system(plugin = TerrainWorldGenPlugin, schedule = Update, config(
	after = ChunkGenSystems, run_if = in_state(GameState::InGame),
))]
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

#[auto_system(plugin = TerrainWorldGenPlugin, schedule = Update, config(
	after = ChunkGenSystems, run_if = in_state(GameState::InGame),
))]
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
struct PoiStructures {
    consort_village: Handle<Scene>,
    imp_arena: Handle<Scene>,
}

#[auto_system(plugin = TerrainWorldGenPlugin, schedule = Startup)]
fn load_poi_structures(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.insert_resource(PoiStructures {
        consort_village: asset_server
            .load(GltfAssetLabel::Scene(0).from_asset("consort village.glb")),
        imp_arena: asset_server.load(GltfAssetLabel::Scene(0).from_asset("imp arena.glb")),
    });
}

#[auto_system(plugin = TerrainWorldGenPlugin, schedule = OnEnter(GameState::InGame))]
fn place_poi_structures(poi: Res<Poi>, poi_structures: Res<PoiStructures>, mut commands: Commands) {
    for (i, position) in poi.positions.iter().enumerate() {
        let poi_structure = match i % 2 {
            0 => &poi_structures.consort_village,
            1 => &poi_structures.imp_arena,
            _ => continue,
        };

        commands
            .spawn((
                SceneRoot(poi_structure.clone()),
                Transform::from_translation(*position).with_rotation(Quat::from_rotation_arc(
                    Vec3::Y,
                    (*position - Vec3::NEG_Y * 1000.0).normalize(),
                )),
                DespawnOnExit(GameState::InGame),
                Readback::buffer(poi.positions_final.clone()),
            ))
            .observe(
                move |readback: On<ReadbackComplete>,
                      mut poi: Query<&mut Transform>,
                      mut commands: Commands|
                      -> Result {
                    let positions: [Vec3; 6] = readback.to_shader_type();
                    let position = positions[i];
                    if position == Vec3::ZERO {
                        return Ok(());
                    }
                    let mut transform = poi.get_mut(readback.entity)?;
                    transform.translation = position;
                    commands.entity(readback.entity).remove::<Readback>();
                    Ok(())
                },
            );
    }
}

#[auto_system(plugin = TerrainWorldGenPlugin, schedule = OnEnter(GameState::InGame))]
fn add_cache(mut commands: Commands) {
    commands.init_resource::<ChunkGeneratorCache<WorldGen>>();
}

#[auto_system(plugin = TerrainWorldGenPlugin, schedule = OnExit(GameState::InGame))]
fn remove_cache(mut commands: Commands) {
    commands.remove_resource::<ChunkGeneratorCache<WorldGen>>();
}
