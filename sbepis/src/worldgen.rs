use bevy::prelude::*;
use bevy_butler::*;
use bevy_marching_cubes::chunk_generator::ChunkGenerator;
use bevy_marching_cubes::height_sampler::{HeightSampler, NoiseHeightSampler};
use fastnoise_lite::*;

use crate::{gridbox_material, prelude::*};

#[add_system(
	plugin = SbepisPlugin, schedule = Startup,
)]
fn spawn_worldgen_test(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) -> Result {
    let mut noise = FastNoiseLite::with_seed(1);
    noise.set_frequency(Some(0.3));
    noise.set_noise_type(Some(NoiseType::Cellular));
    noise.set_cellular_return_type(Some(CellularReturnType::Distance2Sub));

    let chunk_generator = ChunkGenerator {
        surface_threshold: 0.5,
        num_voxels: 50,
        chunk_size: 10.0,
        terrain_sampler: NoiseHeightSampler(noise)
            .scaled(5.0)
            .offset(5.0)
            .build_radius_density(),
    };

    commands.spawn((
        Name::new("Worldgen"),
        Mesh3d(meshes.add(chunk_generator.generate_chunk(IVec3::ZERO))),
        MeshMaterial3d(gridbox_material("white", &mut materials, &asset_server)),
        Transform::from_xyz(0.0, 4.0, 0.0),
    ));

    Ok(())
}
