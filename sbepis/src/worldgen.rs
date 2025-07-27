use bevy::prelude::*;
use bevy_butler::*;
use bevy_marching_cubes::chunk_generator::ChunkGenerator;
use bevy_marching_cubes::height_sampler::{HeightSampler, NoiseRadiusSampler};
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
    noise.set_frequency(Some(1.5));
    noise.set_noise_type(Some(NoiseType::Cellular));
    noise.set_cellular_return_type(Some(CellularReturnType::Distance2Sub));

    let chunk_generator = ChunkGenerator {
        surface_threshold: 0.5,
        num_voxels: 50,
        chunk_size: 10.0,
        terrain_sampler: NoiseRadiusSampler(noise)
            .scaled(5.0)
            .offset(5.0)
            .build_radius_density(),
    };

    for offset in [
        IVec3::new(0, 0, 0),
        IVec3::new(-1, 0, 0),
        IVec3::new(0, -1, 0),
        IVec3::new(0, 0, -1),
        IVec3::new(-1, -1, 0),
        IVec3::new(-1, 0, -1),
        IVec3::new(0, -1, -1),
        IVec3::new(-1, -1, -1),
    ] {
        commands.spawn((
            Name::new("Worldgen"),
            Mesh3d(meshes.add(chunk_generator.generate_chunk(offset))),
            MeshMaterial3d(gridbox_material("white", &mut materials, &asset_server)),
            Transform::from_translation(
                Vec3::new(0.0, 4.0, 0.0) + offset.as_vec3() * chunk_generator.chunk_size,
            ),
        ));
    }

    Ok(())
}
