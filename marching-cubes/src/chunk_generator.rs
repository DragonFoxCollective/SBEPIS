// Most of this code is from https://github.com/SebLague/Terraforming

use bevy::math::{IVec2, IVec3, Vec3};
use bevy::render::mesh::Mesh;
use bevy::utils::HashMap;

use crate::march_tables::*;
use crate::terrain_sampler::TerrainSampler;

#[derive(Debug, Clone)]
struct Vertex {
    position: Vec3,
    normal: Vec3,
    id: IVec2,
}

#[derive(Debug, Clone)]
struct Triangle {
    vertex_a: Vertex,
    vertex_b: Vertex,
    vertex_c: Vertex,
}

#[derive(Debug, Clone)]
pub struct ChunkGenerator<T> {
    pub surface_threshold: f32,
    pub num_voxels: i32,
    pub terrain_sampler: T,
}

impl<T: Default> Default for ChunkGenerator<T> {
    fn default() -> Self {
        Self {
            surface_threshold: 0.5,
            num_voxels: 32,
            terrain_sampler: T::default(),
        }
    }
}

impl<T: TerrainSampler> ChunkGenerator<T> {
    fn coord_to_world(&self, coord: IVec3) -> Vec3 {
        coord.as_vec3()
    }

    fn index_from_coord(&self, coord: IVec3) -> usize {
        let num_samples = self.num_voxels + 1; // +1 because we need to include the last sample
        (coord.z * num_samples * num_samples + coord.y * num_samples + coord.x) as usize
    }

    fn calculate_normal(&self, coord: IVec3) -> Vec3 {
        let offset_x = IVec3::new(1, 0, 0);
        let offset_y = IVec3::new(0, 1, 0);
        let offset_z = IVec3::new(0, 0, 1);

        let x1 = self
            .terrain_sampler
            .sample_density(self.coord_to_world(coord + offset_x));
        let x2 = self
            .terrain_sampler
            .sample_density(self.coord_to_world(coord - offset_x));
        let dx = x2 - x1;

        let y1 = self
            .terrain_sampler
            .sample_density(self.coord_to_world(coord + offset_y));
        let y2 = self
            .terrain_sampler
            .sample_density(self.coord_to_world(coord - offset_y));
        let dy = y2 - y1;

        let z1 = self
            .terrain_sampler
            .sample_density(self.coord_to_world(coord + offset_z));
        let z2 = self
            .terrain_sampler
            .sample_density(self.coord_to_world(coord - offset_z));
        let dz = z2 - z1;

        Vec3::new(dx, dy, dz).normalize()
    }

    // Calculate the position of the vertex
    // The position lies somewhere along the edge defined by the two corner points.
    // Where exactly along the edge is determined by the values of each corner point.
    fn create_vertex(&self, coord_a: IVec3, coord_b: IVec3) -> Vertex {
        let pos_a = self.coord_to_world(coord_a);
        let pos_b = self.coord_to_world(coord_b);
        let density_a = self.terrain_sampler.sample_density(pos_a);
        let density_b = self.terrain_sampler.sample_density(pos_b);

        // Interpolate between the two corner points based on the density
        let t = (self.surface_threshold - density_a) / (density_b - density_a);
        let position = pos_a + t * (pos_b - pos_a);

        // Normal:
        let normal_a = self.calculate_normal(coord_a);
        let normal_b = self.calculate_normal(coord_b);
        let normal = (normal_a + t * (normal_b - normal_a)).normalize();

        // ID
        let index_a = self.index_from_coord(coord_a);
        let index_b = self.index_from_coord(coord_b);

        // Create vertex
        Vertex {
            position,
            normal,
            id: IVec2::new(index_a.min(index_b) as i32, index_a.max(index_b) as i32),
        }
    }

    fn process_cube(&self, id: IVec3) -> Vec<Triangle> {
        let mut triangles = Vec::new();

        if id.x >= self.num_voxels || id.y >= self.num_voxels || id.z >= self.num_voxels {
            return triangles;
        }

        let coord = id;

        // Calculate coordinates of each corner of the current cube
        let corner_coords = [
            coord + IVec3::new(0, 0, 0),
            coord + IVec3::new(1, 0, 0),
            coord + IVec3::new(1, 0, 1),
            coord + IVec3::new(0, 0, 1),
            coord + IVec3::new(0, 1, 0),
            coord + IVec3::new(1, 1, 0),
            coord + IVec3::new(1, 1, 1),
            coord + IVec3::new(0, 1, 1),
        ];
        // Calculate unique index for each cube configuration.
        // There are 256 possible values (cube has 8 corners, so 2^8 possibilites).
        // A value of 0 means cube is entirely inside the surface; 255 entirely outside.
        // The value is used to look up the edge table, which indicates which edges of the cube the surface passes through.
        let mut cube_configuration = 0;
        for (i, corner_coord) in corner_coords.iter().enumerate() {
            // Think of the configuration as an 8-bit binary number (each bit represents the state of a corner point).
            // The state of each corner point is either 0: above the surface, or 1: below the surface.
            // The code below sets the corresponding bit to 1, if the point is below the surface.
            if self
                .terrain_sampler
                .sample_density(self.coord_to_world(*corner_coord))
                < self.surface_threshold
            {
                cube_configuration |= 1 << i;
            }
        }

        // Get array of the edges of the cube that the surface passes through.
        let edge_indices = TRIANGULATION[cube_configuration];

        // Create triangles for the current cube configuration
        for i in (0..16).step_by(3) {
            // If edge index is -1, then no further vertices exist in this configuration
            if edge_indices[i] == -1 {
                break;
            }

            // Get indices of the two corner points defining the edge that the surface passes through.
            // (Do this for each of the three edges we're currently looking at).
            let edge_index_a = edge_indices[i] as usize;
            let a0 = CORNER_INDEX_A_FROM_EDGE[edge_index_a];
            let a1 = CORNER_INDEX_B_FROM_EDGE[edge_index_a];

            let edge_index_b = edge_indices[i + 1] as usize;
            let b0 = CORNER_INDEX_A_FROM_EDGE[edge_index_b];
            let b1 = CORNER_INDEX_B_FROM_EDGE[edge_index_b];

            let edge_index_c = edge_indices[i + 2] as usize;
            let c0 = CORNER_INDEX_A_FROM_EDGE[edge_index_c];
            let c1 = CORNER_INDEX_B_FROM_EDGE[edge_index_c];

            // Calculate positions of each vertex.
            let vertex_a = self.create_vertex(corner_coords[a0], corner_coords[a1]);
            let vertex_b = self.create_vertex(corner_coords[b0], corner_coords[b1]);
            let vertex_c = self.create_vertex(corner_coords[c0], corner_coords[c1]);

            // Create triangle
            let tri = Triangle {
                vertex_a,
                vertex_b,
                vertex_c,
            };
            triangles.push(tri);
        }

        triangles
    }

    pub fn generate_chunk(&self) -> Mesh {
        let mut triangles = Vec::<Triangle>::new();
        for x in 0..self.num_voxels {
            for y in 0..self.num_voxels {
                for z in 0..self.num_voxels {
                    triangles.extend(self.process_cube(IVec3::new(x, y, z)));
                }
            }
        }

        let vertex_data: Vec<Vertex> = triangles
            .iter()
            .flat_map(|t| [t.vertex_c.clone(), t.vertex_b.clone(), t.vertex_a.clone()])
            .collect();

        let mut vertex_positions: Vec<Vec3> = vec![];
        let mut vertex_normals: Vec<Vec3> = vec![];
        let mut indices: Vec<u32> = vec![];
        let mut vertex_index_map: HashMap<IVec2, u32> = HashMap::new();

        let mut vertex_index = 0;
        for data in vertex_data.iter() {
            if let Some(shared_vertex_index) = vertex_index_map.get(&data.id) {
                indices.push(*shared_vertex_index);
            } else {
                vertex_index_map.insert(data.id, vertex_index);
                vertex_positions.push(data.position);
                vertex_normals.push(-data.normal);
                indices.push(vertex_index);
                vertex_index += 1;
            }
        }

        Mesh::new(
            bevy::render::mesh::PrimitiveTopology::TriangleList,
            bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_indices(bevy::render::mesh::Indices::U32(indices))
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertex_positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vertex_normals)
    }
}
