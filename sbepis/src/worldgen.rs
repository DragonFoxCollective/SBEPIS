use std::sync::{Arc, Mutex};

use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use bevy_butler::*;
use bevy_marching_cubes::chunk_generator::ChunkGenerator;
use bevy_marching_cubes::height_sampler::*;
use fastnoise_lite::*;

use crate::{gridbox_material, prelude::*};

#[derive(Resource, Default)]
#[insert_resource(plugin = SbepisPlugin)]
pub struct ChunkLoading {
    pub loaded_chunks: HashSet<IVec3>,
    pub chunks_to_insert: Arc<Mutex<Vec<(IVec3, Vec3, Mesh)>>>,
}

pub type PlanetGenerator = ChunkGenerator<
    RadiusDensitySampler<OffsetHeightSampler<ScaleHeightSampler<NoiseRadiusSampler>>>,
>;

#[derive(Resource)]
#[insert_resource(plugin = SbepisPlugin, init = planet_worldgen())]
pub struct Worldgen {
    pub generator: Arc<PlanetGenerator>,
}

fn planet_worldgen() -> Worldgen {
    let mut noise = FastNoiseLite::with_seed(1);
    noise.set_frequency(Some(1.5));
    noise.set_noise_type(Some(NoiseType::Cellular));
    noise.set_cellular_return_type(Some(CellularReturnType::Distance2Sub));

    Worldgen {
        generator: Arc::new(ChunkGenerator {
            surface_threshold: 0.5,
            num_voxels: 50,
            chunk_size: 10.0,
            terrain_sampler: NoiseRadiusSampler(noise)
                .scaled(5.0)
                .offset(5.0)
                .build_radius_density(),
        }),
    }
}

#[add_system(
	plugin = SbepisPlugin, schedule = Update,
)]
fn start_loading_chunks(
    worldgen: Res<Worldgen>,
    mut chunk_loading: ResMut<ChunkLoading>,
    players: Query<&Transform, With<PlayerBody>>,
) {
    let chunk_size = worldgen.generator.chunk_size;
    for player_transform in players.iter() {
        let chunk_position = (player_transform.translation / chunk_size)
            .floor()
            .as_ivec3();

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
