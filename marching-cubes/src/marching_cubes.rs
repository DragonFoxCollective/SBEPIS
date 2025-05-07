use std::f32::consts::PI;

use bevy::math::{IVec2, IVec3, Quat, Vec3};

use crate::march_tables::*;

const SURFACE_THRESHOLD: f32 = 0.5;
pub const NUM_SAMPLES: i32 = 32;
pub const NUM_VOXELS: i32 = NUM_SAMPLES - 1;

pub static CHUNK_COORD: IVec3 = IVec3::new(0, 0, 0);

#[derive(Debug, Clone)]
pub struct Vertex {
	pub position: Vec3,
	pub normal: Vec3,
	pub id: IVec2,
}

#[derive(Debug, Clone)]
pub struct Triangle {
	pub vertex_a: Vertex,
	pub vertex_b: Vertex,
	pub vertex_c: Vertex,
}

fn coord_to_world(coord: IVec3) -> Vec3 {
	// (coord / (textureSize - 1.0) - 0.5f) * planetSize
	coord.as_vec3()
}

fn index_from_coord(coord: IVec3) -> usize {
	let coord = coord - CHUNK_COORD;
	(coord.z * NUM_SAMPLES * NUM_SAMPLES + coord.y * NUM_SAMPLES + coord.x) as usize
}

fn sample_density(position: IVec3) -> f32 {
	let center = CHUNK_COORD.as_vec3() + Vec3::splat(NUM_SAMPLES as f32 / 2.0);

	let position = position.as_vec3() - center;
	let position = Quat::from_axis_angle(Vec3::new(1., 1., 1.), PI / 3.0) * position;

	let dimensions = Vec3::new(9.5, 20.5, 24.0);
	let half_dimensions = dimensions / 2.0;

	let distance = position.abs();
	let normalized_distance = distance / half_dimensions;
	let max_distance = normalized_distance.max_element();

	1.0 - max_distance
}

fn calculate_normal(coord: IVec3) -> Vec3 {
	let offset_x = IVec3::new(1, 0, 0);
	let offset_y = IVec3::new(0, 1, 0);
	let offset_z = IVec3::new(0, 0, 1);

	let dx = sample_density(coord + offset_x) - sample_density(coord - offset_x);
	let dy = sample_density(coord + offset_y) - sample_density(coord - offset_y);
	let dz = sample_density(coord + offset_z) - sample_density(coord - offset_z);

	Vec3::new(dx, dy, dz).normalize()
}

// Calculate the position of the vertex
// The position lies somewhere along the edge defined by the two corner points.
// Where exactly along the edge is determined by the values of each corner point.
fn create_vertex(coord_a: IVec3, coord_b: IVec3) -> Vertex {
	let pos_a = coord_to_world(coord_a);
	let pos_b = coord_to_world(coord_b);
	let density_a = sample_density(coord_a);
	let density_b = sample_density(coord_b);

	// Interpolate between the two corner points based on the density
	let t = (SURFACE_THRESHOLD - density_a) / (density_b - density_a);
	let position = pos_a + t * (pos_b - pos_a);

	// Normal:
	let normal_a = calculate_normal(coord_a);
	let normal_b = calculate_normal(coord_b);
	let normal = (normal_a + t * (normal_b - normal_a)).normalize();

	// ID
	let index_a = index_from_coord(coord_a);
	let index_b = index_from_coord(coord_b);

	// Create vertex
	Vertex {
		position,
		normal,
		id: IVec2::new(index_a.min(index_b) as i32, index_a.max(index_b) as i32),
	}
}

pub fn process_cube(id: IVec3) -> Vec<Triangle> {
	let mut triangles = Vec::new();

	if id.x >= NUM_VOXELS || id.y >= NUM_VOXELS || id.z >= NUM_VOXELS {
		return triangles;
	}

	let coord = id + CHUNK_COORD;

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
		if sample_density(*corner_coord) < SURFACE_THRESHOLD {
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
		let vertex_a = create_vertex(corner_coords[a0], corner_coords[a1]);
		let vertex_b = create_vertex(corner_coords[b0], corner_coords[b1]);
		let vertex_c = create_vertex(corner_coords[c0], corner_coords[c1]);

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
