use std::sync::{Arc, Mutex};

use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use bevy_butler::*;
use bevy_marching_cubes::chunk_generator::ChunkGenerator;
use bevy_marching_cubes::height_sampler::*;
use fastnoise_lite::*;

use crate::{gridbox_material, prelude::*};

#[derive(Resource, Default, Debug)]
#[insert_resource(plugin = SbepisPlugin, init = ChunkLoading {
	loading_radius: 3,
	..default()
})]
pub struct ChunkLoading {
    pub loading_radius: i32,
    pub loaded_chunks: HashSet<IVec3>,
    pub chunks_to_insert: Arc<Mutex<Vec<(IVec3, Vec3, Mesh)>>>,
}

pub type PlanetGenerator = ChunkGenerator<
    HeightDensitySampler<OffsetHeightSampler<ScaleHeightSampler<NoiseHeightSampler>>>,
>;

#[derive(Resource, Debug)]
#[insert_resource(plugin = SbepisPlugin, init = planet_worldgen())]
pub struct Worldgen {
    pub generator: Arc<PlanetGenerator>,
}

fn planet_worldgen() -> Worldgen {
    let mut noise = FastNoiseLite::with_seed(1);
    noise.set_frequency(Some(0.3));
    noise.set_noise_type(Some(NoiseType::Cellular));
    noise.set_cellular_return_type(Some(CellularReturnType::Distance2Sub));

    let generator = PlanetGenerator {
        surface_threshold: 0.5,
        num_voxels: 50,
        chunk_size: 10.0,
        terrain_sampler: NoiseHeightSampler(noise)
            .scaled(5.0)
            .offset(5.0)
            .build_density(),
    };

    Worldgen {
        generator: Arc::new(generator),
    }
}

#[derive(Component, Default, Debug)]
pub struct InChunk(pub IVec3);

#[add_system(
	plugin = SbepisPlugin, schedule = Update,
)]
fn update_in_chunks(
    worldgen: Res<Worldgen>,
    mut players: Query<(&mut InChunk, &GlobalTransform), Changed<GlobalTransform>>,
) {
    for (mut in_chunk, player_transform) in players.iter_mut() {
        let chunk_position = (player_transform.translation() / worldgen.generator.chunk_size)
            .floor()
            .as_ivec3();

        // Properly update change detection
        if in_chunk.0 != chunk_position {
            in_chunk.0 = chunk_position;
        }
    }
}

#[add_system(
	plugin = SbepisPlugin, schedule = Update,
)]
fn start_loading_chunks(
    worldgen: Res<Worldgen>,
    mut chunk_loading: ResMut<ChunkLoading>,
    players: Query<&InChunk, Changed<InChunk>>,
) {
    let chunk_size = worldgen.generator.chunk_size;
    for in_chunk in players.iter() {
        for x in -chunk_loading.loading_radius..=chunk_loading.loading_radius {
            for y in -chunk_loading.loading_radius..=chunk_loading.loading_radius {
                for z in -chunk_loading.loading_radius..=chunk_loading.loading_radius {
                    let chunk_position = in_chunk.0 + IVec3::new(x, y, z);

                    if !chunk_loading.loaded_chunks.contains(&chunk_position) {
                        chunk_loading.loaded_chunks.insert(chunk_position);

                        debug!("Generating chunk at {chunk_position:?}");

                        let generator = worldgen.generator.clone();
                        let chunks_to_insert = chunk_loading.chunks_to_insert.clone();

                        AsyncComputeTaskPool::get()
                            .spawn(async move {
                                let mesh = generator.generate_chunk(chunk_position);
                                chunks_to_insert.lock().unwrap().push((
                                    chunk_position,
                                    chunk_position.as_vec3() * chunk_size,
                                    mesh,
                                ));
                            })
                            .detach();
                    }
                }
            }
        }
    }
}

#[add_system(
	plugin = SbepisPlugin, schedule = Update,
)]
fn finish_loading_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    chunk_loading: Res<ChunkLoading>,
) {
    for (id, offset, mesh) in chunk_loading.chunks_to_insert.lock().unwrap().drain(..) {
        debug!("Generated chunk at {id:?}");

        commands.spawn((
            Name::new(format!("Chunk {id:?}")),
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(gridbox_material("white", &mut materials, &asset_server)),
            Transform::from_translation(offset),
        ));
    }
}
