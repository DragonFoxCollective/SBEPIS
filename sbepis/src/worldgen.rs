use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;
use bevy_butler::*;
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
    noise.set_frequency(Some(0.5));
    noise.set_noise_type(Some(NoiseType::Cellular));
    noise.set_cellular_return_type(Some(CellularReturnType::Distance2Sub));

    let mut mesh = Plane3d {
        half_size: Vec2::new(5.0, 5.0),
        normal: Dir3::Y,
    }
    .mesh()
    .subdivisions(100)
    .build();

    let mut positions = None;
    let mut normals = None;
    for (key, value) in mesh.attributes_mut() {
        if key.id == Mesh::ATTRIBUTE_POSITION.id {
            if let VertexAttributeValues::Float32x3(p) = value {
                positions = Some(p);
            }
        } else if key.id == Mesh::ATTRIBUTE_NORMAL.id {
            if let VertexAttributeValues::Float32x3(n) = value {
                normals = Some(n);
            }
        }
    }
    if let (Some(positions), Some(normals)) = (positions, normals) {
        for (p, n) in positions.iter_mut().zip(normals.iter()) {
            let distance = noise.get_noise_3d(p[0], p[1], p[2]);
            p[0] += n[0] * distance;
            p[1] += n[1] * distance;
            p[2] += n[2] * distance;
        }
    } else {
        return Err("Worldgen mesh does not have the right attributes".into());
    }
    mesh.compute_smooth_normals();

    let mesh = meshes.add(mesh);

    commands.spawn((
        Name::new("Worldgen"),
        Mesh3d(mesh),
        MeshMaterial3d(gridbox_material("white", &mut materials, &asset_server)),
        Transform::from_xyz(0.0, 4.0, 0.0),
    ));

    Ok(())
}
