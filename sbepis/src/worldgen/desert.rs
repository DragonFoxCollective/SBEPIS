use std::f32::consts::PI;

use bevy::ecs::schedule::ScheduleConfigs;
use bevy::ecs::system::ScheduleSystem;
use bevy::prelude::*;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::gpu_readback::{Readback, ReadbackComplete};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::binding_types::{storage_buffer, uniform_buffer};
use bevy::render::render_resource::{
    BindGroupLayoutEntryBuilder, Buffer, BufferUsages, UniformBuffer,
};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::storage::{GpuShaderStorageBuffer, ShaderStorageBuffer};
use bevy::shader::ShaderRef;
use bevy_auto_plugin::prelude::*;
use bevy_marching_cubes::*;

use crate::gridbox_material;
use crate::prelude::*;

#[derive(AutoPlugin)]
#[auto_add_plugin(plugin = crate::worldgen::WorldGenPlugin)]
pub struct DesertWorldGenPlugin;

#[auto_plugin(plugin = DesertWorldGenPlugin)]
fn build(app: &mut App) {
    app.add_plugins((
        MarchingCubesPlugin::<DesertWorldGen, DesertPoiBuffers, StandardMaterial>::default(),
        ExtractResourcePlugin::<DesertPoi>::default(),
    ))
    .insert_resource(
        ChunkGeneratorSettings::<DesertWorldGen>::new(50, 50.0)
            .with_bounds(vec3(-50.0, -50.0, -50.0), vec3(1100.0, 50.0, 1100.0))
            .stopped(),
    );
}

pub struct DesertWorldGen;
impl ChunkComputeShader for DesertWorldGen {
    fn shader() -> ShaderRef {
        "sample desert.wgsl".into()
    }
}

struct DesertPoiBuffers {
    poi_positions: Buffer,
    poi_positions_final: Buffer,
}

impl GpuExtraBufferCache for DesertPoiBuffers {
    fn define_extra_buffers() -> Vec<BindGroupLayoutEntryBuilder> {
        vec![
            uniform_buffer::<[Vec3; 1]>(false),
            storage_buffer::<[Vec3; 1]>(false),
        ]
    }

    fn create_extra_buffers() -> ScheduleConfigs<ScheduleSystem> {
        IntoSystem::into_system(
            |render_device: Res<RenderDevice>,
             render_queue: Res<RenderQueue>,
             buffers: Res<RenderAssets<GpuShaderStorageBuffer>>,
             mut cache: ResMut<GpuChunkGeneratorCache<DesertWorldGen, DesertPoiBuffers>>,
             poi: Res<DesertPoi>| {
                for key in cache.drain_needed_extra_buffers() {
                    let mut poi_positions_buffer = UniformBuffer::from(&poi.positions);
                    poi_positions_buffer.write_buffer(&render_device, &render_queue);

                    cache.insert_extra_buffers(
                        key,
                        DesertPoiBuffers {
                            poi_positions: poi_positions_buffer.buffer().unwrap().clone(),
                            poi_positions_final: buffers
                                .get(&poi.positions_final)
                                .unwrap()
                                .buffer
                                .clone(),
                        },
                    );
                }
            },
        )
        .into_configs()
    }

    fn clear_extra_buffers() -> ScheduleConfigs<ScheduleSystem> {
        // Not needed since poi_positions remains constant through all buffers
        IntoSystem::into_system(|| {}).into_configs()
    }

    fn buffers(&self) -> Vec<Buffer> {
        vec![self.poi_positions.clone(), self.poi_positions_final.clone()]
    }

    fn num_extra_readbacks() -> usize {
        0
    }
}

#[auto_resource(plugin = DesertWorldGenPlugin, derive(ExtractResource, Clone), reflect)]
pub struct DesertPoi {
    positions: [Vec3; 1],
    positions_final: Handle<ShaderStorageBuffer>,
}

#[auto_system(plugin = DesertWorldGenPlugin, schedule = Startup)]
fn setup_poi(mut commands: Commands, mut buffers: ResMut<Assets<ShaderStorageBuffer>>) {
    let mut poi_positions_final_buffer = ShaderStorageBuffer::from([Vec3::ZERO]);
    poi_positions_final_buffer.buffer_description.usage |= BufferUsages::COPY_SRC;
    commands.insert_resource(DesertPoi {
        positions: [vec3(-8.0, 0.0, -4.0)],
        positions_final: buffers.add(poi_positions_final_buffer),
    });
}

#[auto_system(plugin = DesertWorldGenPlugin, schedule = Startup)]
fn setup_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(ChunkMaterial::<DesertWorldGen, StandardMaterial>::new(
        gridbox_material("yellow", &mut materials, &asset_server),
    ));
}

#[auto_system(plugin = DesertWorldGenPlugin, schedule = Update, config(
	after = ChunkGenSystems,
))]
fn add_components(
    mut commands: Commands,
    chunks: Query<Entity, (With<Chunk<DesertWorldGen>>, Without<FinalizedChunk>)>,
) {
    for chunk in chunks.iter() {
        commands
            .entity(chunk)
            .insert((FinalizedChunk, DespawnOnExit(GameState::MainMenu)));
    }
}

#[auto_component(plugin = DesertWorldGenPlugin, derive(Debug), reflect, register)]
struct FinalizedChunk;

#[derive(Resource, Debug)]
struct DesertPoiStructures {
    command_station: Handle<Scene>,
}

#[auto_system(plugin = DesertWorldGenPlugin, schedule = Startup)]
fn load_poi_structures(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.insert_resource(DesertPoiStructures {
        command_station: asset_server
            .load(GltfAssetLabel::Scene(0).from_asset("command station.glb")),
    });
}

#[auto_system(plugin = DesertWorldGenPlugin, schedule = OnEnter(GameState::MainMenu))]
fn setup_poi_structures(
    poi: Res<DesertPoi>,
    poi_structures: Res<DesertPoiStructures>,
    mut commands: Commands,
) {
    for (i, position) in poi.positions.iter().enumerate() {
        let poi_structure = &poi_structures.command_station;

        commands
            .spawn((
                SceneRoot(poi_structure.clone()),
                Transform::from_translation(*position).with_rotation(Quat::from_rotation_y(PI)),
                DespawnOnExit(GameState::MainMenu),
                Readback::buffer(poi.positions_final.clone()),
            ))
            .observe(
                move |readback: On<ReadbackComplete>,
                      mut poi: Query<&mut Transform>,
                      mut commands: Commands|
                      -> Result {
                    let positions: [Vec3; 1] = readback.to_shader_type();
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

#[auto_system(plugin = DesertWorldGenPlugin, schedule = OnEnter(GameState::MainMenu))]
fn start_chunks(mut settings: ResMut<ChunkGeneratorSettings<DesertWorldGen>>) {
    settings.running = ChunkGeneratorRunning::Run;
}

#[auto_system(plugin = DesertWorldGenPlugin, schedule = OnExit(GameState::MainMenu))]
fn stop_chunks(mut settings: ResMut<ChunkGeneratorSettings<DesertWorldGen>>) {
    settings.running = ChunkGeneratorRunning::Stop;
}
