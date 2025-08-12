// Most of this code is from https://github.com/SebLague/Terraforming
// with a hint of https://github.com/qhdwight/voxel-game-rs

struct Vertex {
	position: vec3<f32>,
	normal: vec3<f32>,
}

struct Triangle {
	vertex_a: u32,
	vertex_b: u32,
	vertex_c: u32,
}

@group(0) @binding(0)
var<storage, read> densities: array<f32>;

@group(0) @binding(1)
var<uniform> num_voxels_per_axis: u32;

@group(0) @binding(2)
var<uniform> num_samples_per_axis: u32;

@group(0) @binding(3)
var<uniform> chunk_size: f32;

@group(0) @binding(4)
var<uniform> surface_threshold: f32;

@group(0) @binding(5)
var<storage, read_write> out_vertices: array<Vertex>;

@group(0) @binding(6)
var<storage, read_write> out_vertices_len: atomic<u32>;

@group(0) @binding(7)
var<storage, read_write> out_triangles: array<Triangle>;

@group(0) @binding(8)
var<storage, read_write> out_triangles_len: atomic<u32>;

fn coord_to_world(coord: vec3<i32>) -> vec3<f32> {
	return vec3<f32>(coord) / vec3<f32>(num_voxels_per_axis) * chunk_size;
}

fn sample_density(coord: vec3<i32>) -> f32 {
	let shifted_coord = coord + vec3<i32>(1, 1, 1); // Shift to account for the fact that we sample from the next chunk over too for normals
	return densities[shifted_coord.x * i32(num_samples_per_axis) * i32(num_samples_per_axis) + shifted_coord.y * i32(num_samples_per_axis) + shifted_coord.z];
}

fn calculate_normal(coord: vec3<i32>) -> vec3<f32> {
	let offset_x = vec3<i32>(1, 0, 0);
	let offset_y = vec3<i32>(0, 1, 0);
	let offset_z = vec3<i32>(0, 0, 1);

	let dx = sample_density(coord - offset_x) - sample_density(coord + offset_x);
	let dy = sample_density(coord - offset_y) - sample_density(coord + offset_y);
	let dz = sample_density(coord - offset_z) - sample_density(coord + offset_z);

	return normalize(vec3<f32>(dx, dy, dz));
}

// Calculate the position of the vertex
// The position lies somewhere along the edge defined by the two corner points.
// Where exactly along the edge is determined by the values of each corner point.
fn create_vertex(coord_a: vec3<i32>, coord_b: vec3<i32>) -> Vertex {

	let pos_a = coord_to_world(coord_a);
	let pos_b = coord_to_world(coord_b);
	let density_a = sample_density(coord_a);
	let density_b = sample_density(coord_b);

	// Interpolate between the two corner points based on the density
	let t = (surface_threshold - density_a) / (density_b - density_a);
	let position = pos_a + t * (pos_b - pos_a);

	// Normal:
	let normal_a = calculate_normal(coord_a);
	let normal_b = calculate_normal(coord_b);
	let normal = normalize(normal_a + t * (normal_b - normal_a));

	// Create vertex
	let vertex = Vertex(
		position,
		normal,
	);

	return vertex;
}

@compute @workgroup_size(8, 8, 8)
fn main2(
	@builtin(global_invocation_id) coord: vec3<u32>
) {
	let _density = densities[0];
	let _num_voxels_per_axis = num_voxels_per_axis;
	let _chunk_size = chunk_size;
	let _surface_threshold = surface_threshold;

	let vertex_index = atomicAdd(&out_vertices_len, 3u);
	out_vertices[vertex_index] = Vertex(
		vec3<f32>(coord),
		vec3<f32>(0.0, 1.0, 0.0),
	);
	out_vertices[vertex_index + 1] = Vertex(
		vec3<f32>(coord + vec3<u32>(1, 0, 0)),
		vec3<f32>(0.0, 1.0, 0.0),
	);
	out_vertices[vertex_index + 2] = Vertex(
		vec3<f32>(coord + vec3<u32>(0, 0, 1)),
		vec3<f32>(0.0, 1.0, 0.0),
	);

	let triangle_index = atomicAdd(&out_triangles_len, 1u);
	out_triangles[triangle_index] = Triangle(
		u32(vertex_index),
		u32(vertex_index + 1),
		u32(vertex_index + 2),
	);
}

@compute @workgroup_size(8, 8, 8)
fn main(
	@builtin(global_invocation_id) coord: vec3<u32>
) {
	if coord.x >= num_voxels_per_axis || coord.y >= num_voxels_per_axis || coord.z >= num_voxels_per_axis {
		return;
	}

	// Calculate coordinates of each corner of the current cube
	// let corner_coords = array<vec3<i32>, 8>(
	// 	vec3<i32>(coord) + vec3<i32>(0, 0, 0),
	// 	vec3<i32>(coord) + vec3<i32>(1, 0, 0),
	// 	vec3<i32>(coord) + vec3<i32>(1, 0, 1),
	// 	vec3<i32>(coord) + vec3<i32>(0, 0, 1),
	// 	vec3<i32>(coord) + vec3<i32>(0, 1, 0),
	// 	vec3<i32>(coord) + vec3<i32>(1, 1, 0),
	// 	vec3<i32>(coord) + vec3<i32>(1, 1, 1),
	// 	vec3<i32>(coord) + vec3<i32>(0, 1, 1),
	// );
	// note to self: naga segfaults when this is accessed by a non-constant

	// Calculate unique index for each cube configuration.
	// There are 256 possible values (cube has 8 corners, so 2^8 possibilites).
	// A value of 0 means cube is entirely inside the surface; 255 entirely outside.
	// The value is used to look up the edge table, which indicates which edges of the cube the surface passes through.
	var cube_configuration = 0u;
	for (var i = 0; i < 8; i++) {
		// Think of the configuration as an 8-bit binary number (each bit represents the state of a corner point).
		// The state of each corner point is either 0: above the surface, or 1: below the surface.
		// The code below sets the corresponding bit to 1, if the point is below the surface.
		if sample_density(vec3<i32>(coord) + CORNER_COORDS[i]) < surface_threshold {
			cube_configuration |= 1u << u32(i);
		}
	}
	
	// Get array of the edges of the cube that the surface passes through.
	var edge_indices = TRIANGULATION[cube_configuration];

	// Create vertices for the edges that the surface passes through.
	var edge_vertices = array<Vertex, 12>();
	var edge_vertices_created = array<bool, 12>();
	var num_vertices = 0u;
	var num_vertices_for_triangles = 0u;
	for (var i = 0; i < 15; i++) {
		// If edge index is -1, then no further vertices exist in this configuration
		let edge_index = edge_indices[i];
		if edge_index == -1 { break; }
		num_vertices_for_triangles += 1;

		// If the edge has already been created, skip it.
		if edge_vertices_created[edge_index] { continue; }
		num_vertices += 1;

		// Get indices of the two corner points defining the edge that the surface passes through.
		let coord_a = vec3<i32>(coord) + CORNER_COORDS[CORNER_INDEX_A_FROM_EDGE[edge_index]];
		let coord_b = vec3<i32>(coord) + CORNER_COORDS[CORNER_INDEX_B_FROM_EDGE[edge_index]];
		edge_vertices[edge_index] = create_vertex(coord_a, coord_b);
		edge_vertices_created[edge_index] = true;
	}

	// Map the vertices and their indices to the output buffer.
	var current_vertex_index = atomicAdd(&out_vertices_len, num_vertices);
	for (var i = 0; i < 12; i++) {
		// If the edge has not been created, skip it.
		if !edge_vertices_created[i] { continue; }

		out_vertices[current_vertex_index] = edge_vertices[i];
		for (var j = 0; j < 15; j++) {
			if edge_indices[j] == i {
				edge_indices[j] = i32(current_vertex_index);
			}
		}
		current_vertex_index += 1;
	}

	// Create triangles for the current cube configuration
	var current_triangle_index = atomicAdd(&out_triangles_len, num_vertices_for_triangles / 3);
	for (var i = 0; i < 15; i += 3) {
		// If edge index is -1, then no further vertices exist in this configuration
		if edge_indices[i] == -1 { break; }

		// Create triangle
		out_triangles[current_triangle_index] = Triangle(
			u32(edge_indices[i]),
			u32(edge_indices[i + 1]),
			u32(edge_indices[i + 2]),
		);
		current_triangle_index += 1;
	}
}

// This array contains the coordinates of the corners indexed by CORNER_INDEX_A_FROM_EDGE and CORNER_INDEX_B_FROM_EDGE.
const CORNER_COORDS = array<vec3<i32>, 8>(
	vec3<i32>(0, 0, 0),
	vec3<i32>(1, 0, 0),
	vec3<i32>(1, 0, 1),
	vec3<i32>(0, 0, 1),
	vec3<i32>(0, 1, 0),
	vec3<i32>(1, 1, 0),
	vec3<i32>(1, 1, 1),
	vec3<i32>(0, 1, 1),
);

// These two arrays allow for easy lookup of the indices of the two corner points that form an edge.
// The edge index can be obtained from the triangulation table further below.
const CORNER_INDEX_A_FROM_EDGE = array<u32, 12>(0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3);
const CORNER_INDEX_B_FROM_EDGE = array<u32, 12>(1, 2, 3, 0, 5, 6, 7, 4, 4, 5, 6, 7);

// Values from http://paulbourke.net/geometry/polygonise/
// Lookup table giving the index of the edge that each vertex lies on for any cube configuration.

// The first index is the cube configuration.
// Since a cube has 8 corners, there are 2^8 = 256 possible configurations.

// The second index is the vertex index. No configuration has more than 15 vertices.
// An entry of -1 means that there are no further vertices in the configuration.
const TRIANGULATION = array<array<i32, 15>, 256>(
	array<i32, 15>(-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(0, 8, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(0, 1, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(1, 8, 3, 9, 8, 1, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(1, 2, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(0, 8, 3, 1, 2, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(9, 2, 10, 0, 2, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(2, 8, 3, 2, 10, 8, 10, 9, 8, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(3, 11, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(0, 11, 2, 8, 11, 0, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(1, 9, 0, 2, 3, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(1, 11, 2, 1, 9, 11, 9, 8, 11, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(3, 10, 1, 11, 10, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(0, 10, 1, 0, 8, 10, 8, 11, 10, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(3, 9, 0, 3, 11, 9, 11, 10, 9, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(9, 8, 10, 10, 8, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(4, 7, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(4, 3, 0, 7, 3, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(0, 1, 9, 8, 4, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(4, 1, 9, 4, 7, 1, 7, 3, 1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(1, 2, 10, 8, 4, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(3, 4, 7, 3, 0, 4, 1, 2, 10, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(9, 2, 10, 9, 0, 2, 8, 4, 7, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(2, 10, 9, 2, 9, 7, 2, 7, 3, 7, 9, 4, -1, -1, -1),
	array<i32, 15>(8, 4, 7, 3, 11, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(11, 4, 7, 11, 2, 4, 2, 0, 4, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(9, 0, 1, 8, 4, 7, 2, 3, 11, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(4, 7, 11, 9, 4, 11, 9, 11, 2, 9, 2, 1, -1, -1, -1),
	array<i32, 15>(3, 10, 1, 3, 11, 10, 7, 8, 4, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(1, 11, 10, 1, 4, 11, 1, 0, 4, 7, 11, 4, -1, -1, -1),
	array<i32, 15>(4, 7, 8, 9, 0, 11, 9, 11, 10, 11, 0, 3, -1, -1, -1),
	array<i32, 15>(4, 7, 11, 4, 11, 9, 9, 11, 10, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(9, 5, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(9, 5, 4, 0, 8, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(0, 5, 4, 1, 5, 0, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(8, 5, 4, 8, 3, 5, 3, 1, 5, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(1, 2, 10, 9, 5, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(3, 0, 8, 1, 2, 10, 4, 9, 5, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(5, 2, 10, 5, 4, 2, 4, 0, 2, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(2, 10, 5, 3, 2, 5, 3, 5, 4, 3, 4, 8, -1, -1, -1),
	array<i32, 15>(9, 5, 4, 2, 3, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(0, 11, 2, 0, 8, 11, 4, 9, 5, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(0, 5, 4, 0, 1, 5, 2, 3, 11, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(2, 1, 5, 2, 5, 8, 2, 8, 11, 4, 8, 5, -1, -1, -1),
	array<i32, 15>(10, 3, 11, 10, 1, 3, 9, 5, 4, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(4, 9, 5, 0, 8, 1, 8, 10, 1, 8, 11, 10, -1, -1, -1),
	array<i32, 15>(5, 4, 0, 5, 0, 11, 5, 11, 10, 11, 0, 3, -1, -1, -1),
	array<i32, 15>(5, 4, 8, 5, 8, 10, 10, 8, 11, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(9, 7, 8, 5, 7, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(9, 3, 0, 9, 5, 3, 5, 7, 3, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(0, 7, 8, 0, 1, 7, 1, 5, 7, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(1, 5, 3, 3, 5, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(9, 7, 8, 9, 5, 7, 10, 1, 2, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(10, 1, 2, 9, 5, 0, 5, 3, 0, 5, 7, 3, -1, -1, -1),
	array<i32, 15>(8, 0, 2, 8, 2, 5, 8, 5, 7, 10, 5, 2, -1, -1, -1),
	array<i32, 15>(2, 10, 5, 2, 5, 3, 3, 5, 7, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(7, 9, 5, 7, 8, 9, 3, 11, 2, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(9, 5, 7, 9, 7, 2, 9, 2, 0, 2, 7, 11, -1, -1, -1),
	array<i32, 15>(2, 3, 11, 0, 1, 8, 1, 7, 8, 1, 5, 7, -1, -1, -1),
	array<i32, 15>(11, 2, 1, 11, 1, 7, 7, 1, 5, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(9, 5, 8, 8, 5, 7, 10, 1, 3, 10, 3, 11, -1, -1, -1),
	array<i32, 15>(5, 7, 0, 5, 0, 9, 7, 11, 0, 1, 0, 10, 11, 10, 0),
	array<i32, 15>(11, 10, 0, 11, 0, 3, 10, 5, 0, 8, 0, 7, 5, 7, 0),
	array<i32, 15>(11, 10, 5, 7, 11, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(10, 6, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(0, 8, 3, 5, 10, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(9, 0, 1, 5, 10, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(1, 8, 3, 1, 9, 8, 5, 10, 6, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(1, 6, 5, 2, 6, 1, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(1, 6, 5, 1, 2, 6, 3, 0, 8, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(9, 6, 5, 9, 0, 6, 0, 2, 6, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(5, 9, 8, 5, 8, 2, 5, 2, 6, 3, 2, 8, -1, -1, -1),
	array<i32, 15>(2, 3, 11, 10, 6, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(11, 0, 8, 11, 2, 0, 10, 6, 5, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(0, 1, 9, 2, 3, 11, 5, 10, 6, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(5, 10, 6, 1, 9, 2, 9, 11, 2, 9, 8, 11, -1, -1, -1),
	array<i32, 15>(6, 3, 11, 6, 5, 3, 5, 1, 3, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(0, 8, 11, 0, 11, 5, 0, 5, 1, 5, 11, 6, -1, -1, -1),
	array<i32, 15>(3, 11, 6, 0, 3, 6, 0, 6, 5, 0, 5, 9, -1, -1, -1),
	array<i32, 15>(6, 5, 9, 6, 9, 11, 11, 9, 8, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(5, 10, 6, 4, 7, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(4, 3, 0, 4, 7, 3, 6, 5, 10, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(1, 9, 0, 5, 10, 6, 8, 4, 7, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(10, 6, 5, 1, 9, 7, 1, 7, 3, 7, 9, 4, -1, -1, -1),
	array<i32, 15>(6, 1, 2, 6, 5, 1, 4, 7, 8, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(1, 2, 5, 5, 2, 6, 3, 0, 4, 3, 4, 7, -1, -1, -1),
	array<i32, 15>(8, 4, 7, 9, 0, 5, 0, 6, 5, 0, 2, 6, -1, -1, -1),
	array<i32, 15>(7, 3, 9, 7, 9, 4, 3, 2, 9, 5, 9, 6, 2, 6, 9),
	array<i32, 15>(3, 11, 2, 7, 8, 4, 10, 6, 5, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(5, 10, 6, 4, 7, 2, 4, 2, 0, 2, 7, 11, -1, -1, -1),
	array<i32, 15>(0, 1, 9, 4, 7, 8, 2, 3, 11, 5, 10, 6, -1, -1, -1),
	array<i32, 15>(9, 2, 1, 9, 11, 2, 9, 4, 11, 7, 11, 4, 5, 10, 6),
	array<i32, 15>(8, 4, 7, 3, 11, 5, 3, 5, 1, 5, 11, 6, -1, -1, -1),
	array<i32, 15>(5, 1, 11, 5, 11, 6, 1, 0, 11, 7, 11, 4, 0, 4, 11),
	array<i32, 15>(0, 5, 9, 0, 6, 5, 0, 3, 6, 11, 6, 3, 8, 4, 7),
	array<i32, 15>(6, 5, 9, 6, 9, 11, 4, 7, 9, 7, 11, 9, -1, -1, -1),
	array<i32, 15>(10, 4, 9, 6, 4, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(4, 10, 6, 4, 9, 10, 0, 8, 3, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(10, 0, 1, 10, 6, 0, 6, 4, 0, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(8, 3, 1, 8, 1, 6, 8, 6, 4, 6, 1, 10, -1, -1, -1),
	array<i32, 15>(1, 4, 9, 1, 2, 4, 2, 6, 4, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(3, 0, 8, 1, 2, 9, 2, 4, 9, 2, 6, 4, -1, -1, -1),
	array<i32, 15>(0, 2, 4, 4, 2, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(8, 3, 2, 8, 2, 4, 4, 2, 6, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(10, 4, 9, 10, 6, 4, 11, 2, 3, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(0, 8, 2, 2, 8, 11, 4, 9, 10, 4, 10, 6, -1, -1, -1),
	array<i32, 15>(3, 11, 2, 0, 1, 6, 0, 6, 4, 6, 1, 10, -1, -1, -1),
	array<i32, 15>(6, 4, 1, 6, 1, 10, 4, 8, 1, 2, 1, 11, 8, 11, 1),
	array<i32, 15>(9, 6, 4, 9, 3, 6, 9, 1, 3, 11, 6, 3, -1, -1, -1),
	array<i32, 15>(8, 11, 1, 8, 1, 0, 11, 6, 1, 9, 1, 4, 6, 4, 1),
	array<i32, 15>(3, 11, 6, 3, 6, 0, 0, 6, 4, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(6, 4, 8, 11, 6, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(7, 10, 6, 7, 8, 10, 8, 9, 10, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(0, 7, 3, 0, 10, 7, 0, 9, 10, 6, 7, 10, -1, -1, -1),
	array<i32, 15>(10, 6, 7, 1, 10, 7, 1, 7, 8, 1, 8, 0, -1, -1, -1),
	array<i32, 15>(10, 6, 7, 10, 7, 1, 1, 7, 3, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(1, 2, 6, 1, 6, 8, 1, 8, 9, 8, 6, 7, -1, -1, -1),
	array<i32, 15>(2, 6, 9, 2, 9, 1, 6, 7, 9, 0, 9, 3, 7, 3, 9),
	array<i32, 15>(7, 8, 0, 7, 0, 6, 6, 0, 2, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(7, 3, 2, 6, 7, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(2, 3, 11, 10, 6, 8, 10, 8, 9, 8, 6, 7, -1, -1, -1),
	array<i32, 15>(2, 0, 7, 2, 7, 11, 0, 9, 7, 6, 7, 10, 9, 10, 7),
	array<i32, 15>(1, 8, 0, 1, 7, 8, 1, 10, 7, 6, 7, 10, 2, 3, 11),
	array<i32, 15>(11, 2, 1, 11, 1, 7, 10, 6, 1, 6, 7, 1, -1, -1, -1),
	array<i32, 15>(8, 9, 6, 8, 6, 7, 9, 1, 6, 11, 6, 3, 1, 3, 6),
	array<i32, 15>(0, 9, 1, 11, 6, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(7, 8, 0, 7, 0, 6, 3, 11, 0, 11, 6, 0, -1, -1, -1),
	array<i32, 15>(7, 11, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(7, 6, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(3, 0, 8, 11, 7, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(0, 1, 9, 11, 7, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(8, 1, 9, 8, 3, 1, 11, 7, 6, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(10, 1, 2, 6, 11, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(1, 2, 10, 3, 0, 8, 6, 11, 7, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(2, 9, 0, 2, 10, 9, 6, 11, 7, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(6, 11, 7, 2, 10, 3, 10, 8, 3, 10, 9, 8, -1, -1, -1),
	array<i32, 15>(7, 2, 3, 6, 2, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(7, 0, 8, 7, 6, 0, 6, 2, 0, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(2, 7, 6, 2, 3, 7, 0, 1, 9, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(1, 6, 2, 1, 8, 6, 1, 9, 8, 8, 7, 6, -1, -1, -1),
	array<i32, 15>(10, 7, 6, 10, 1, 7, 1, 3, 7, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(10, 7, 6, 1, 7, 10, 1, 8, 7, 1, 0, 8, -1, -1, -1),
	array<i32, 15>(0, 3, 7, 0, 7, 10, 0, 10, 9, 6, 10, 7, -1, -1, -1),
	array<i32, 15>(7, 6, 10, 7, 10, 8, 8, 10, 9, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(6, 8, 4, 11, 8, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(3, 6, 11, 3, 0, 6, 0, 4, 6, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(8, 6, 11, 8, 4, 6, 9, 0, 1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(9, 4, 6, 9, 6, 3, 9, 3, 1, 11, 3, 6, -1, -1, -1),
	array<i32, 15>(6, 8, 4, 6, 11, 8, 2, 10, 1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(1, 2, 10, 3, 0, 11, 0, 6, 11, 0, 4, 6, -1, -1, -1),
	array<i32, 15>(4, 11, 8, 4, 6, 11, 0, 2, 9, 2, 10, 9, -1, -1, -1),
	array<i32, 15>(10, 9, 3, 10, 3, 2, 9, 4, 3, 11, 3, 6, 4, 6, 3),
	array<i32, 15>(8, 2, 3, 8, 4, 2, 4, 6, 2, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(0, 4, 2, 4, 6, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(1, 9, 0, 2, 3, 4, 2, 4, 6, 4, 3, 8, -1, -1, -1),
	array<i32, 15>(1, 9, 4, 1, 4, 2, 2, 4, 6, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(8, 1, 3, 8, 6, 1, 8, 4, 6, 6, 10, 1, -1, -1, -1),
	array<i32, 15>(10, 1, 0, 10, 0, 6, 6, 0, 4, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(4, 6, 3, 4, 3, 8, 6, 10, 3, 0, 3, 9, 10, 9, 3),
	array<i32, 15>(10, 9, 4, 6, 10, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(4, 9, 5, 7, 6, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(0, 8, 3, 4, 9, 5, 11, 7, 6, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(5, 0, 1, 5, 4, 0, 7, 6, 11, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(11, 7, 6, 8, 3, 4, 3, 5, 4, 3, 1, 5, -1, -1, -1),
	array<i32, 15>(9, 5, 4, 10, 1, 2, 7, 6, 11, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(6, 11, 7, 1, 2, 10, 0, 8, 3, 4, 9, 5, -1, -1, -1),
	array<i32, 15>(7, 6, 11, 5, 4, 10, 4, 2, 10, 4, 0, 2, -1, -1, -1),
	array<i32, 15>(3, 4, 8, 3, 5, 4, 3, 2, 5, 10, 5, 2, 11, 7, 6),
	array<i32, 15>(7, 2, 3, 7, 6, 2, 5, 4, 9, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(9, 5, 4, 0, 8, 6, 0, 6, 2, 6, 8, 7, -1, -1, -1),
	array<i32, 15>(3, 6, 2, 3, 7, 6, 1, 5, 0, 5, 4, 0, -1, -1, -1),
	array<i32, 15>(6, 2, 8, 6, 8, 7, 2, 1, 8, 4, 8, 5, 1, 5, 8),
	array<i32, 15>(9, 5, 4, 10, 1, 6, 1, 7, 6, 1, 3, 7, -1, -1, -1),
	array<i32, 15>(1, 6, 10, 1, 7, 6, 1, 0, 7, 8, 7, 0, 9, 5, 4),
	array<i32, 15>(4, 0, 10, 4, 10, 5, 0, 3, 10, 6, 10, 7, 3, 7, 10),
	array<i32, 15>(7, 6, 10, 7, 10, 8, 5, 4, 10, 4, 8, 10, -1, -1, -1),
	array<i32, 15>(6, 9, 5, 6, 11, 9, 11, 8, 9, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(3, 6, 11, 0, 6, 3, 0, 5, 6, 0, 9, 5, -1, -1, -1),
	array<i32, 15>(0, 11, 8, 0, 5, 11, 0, 1, 5, 5, 6, 11, -1, -1, -1),
	array<i32, 15>(6, 11, 3, 6, 3, 5, 5, 3, 1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(1, 2, 10, 9, 5, 11, 9, 11, 8, 11, 5, 6, -1, -1, -1),
	array<i32, 15>(0, 11, 3, 0, 6, 11, 0, 9, 6, 5, 6, 9, 1, 2, 10),
	array<i32, 15>(11, 8, 5, 11, 5, 6, 8, 0, 5, 10, 5, 2, 0, 2, 5),
	array<i32, 15>(6, 11, 3, 6, 3, 5, 2, 10, 3, 10, 5, 3, -1, -1, -1),
	array<i32, 15>(5, 8, 9, 5, 2, 8, 5, 6, 2, 3, 8, 2, -1, -1, -1),
	array<i32, 15>(9, 5, 6, 9, 6, 0, 0, 6, 2, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(1, 5, 8, 1, 8, 0, 5, 6, 8, 3, 8, 2, 6, 2, 8),
	array<i32, 15>(1, 5, 6, 2, 1, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(1, 3, 6, 1, 6, 10, 3, 8, 6, 5, 6, 9, 8, 9, 6),
	array<i32, 15>(10, 1, 0, 10, 0, 6, 9, 5, 0, 5, 6, 0, -1, -1, -1),
	array<i32, 15>(0, 3, 8, 5, 6, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(10, 5, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(11, 5, 10, 7, 5, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(11, 5, 10, 11, 7, 5, 8, 3, 0, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(5, 11, 7, 5, 10, 11, 1, 9, 0, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(10, 7, 5, 10, 11, 7, 9, 8, 1, 8, 3, 1, -1, -1, -1),
	array<i32, 15>(11, 1, 2, 11, 7, 1, 7, 5, 1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(0, 8, 3, 1, 2, 7, 1, 7, 5, 7, 2, 11, -1, -1, -1),
	array<i32, 15>(9, 7, 5, 9, 2, 7, 9, 0, 2, 2, 11, 7, -1, -1, -1),
	array<i32, 15>(7, 5, 2, 7, 2, 11, 5, 9, 2, 3, 2, 8, 9, 8, 2),
	array<i32, 15>(2, 5, 10, 2, 3, 5, 3, 7, 5, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(8, 2, 0, 8, 5, 2, 8, 7, 5, 10, 2, 5, -1, -1, -1),
	array<i32, 15>(9, 0, 1, 5, 10, 3, 5, 3, 7, 3, 10, 2, -1, -1, -1),
	array<i32, 15>(9, 8, 2, 9, 2, 1, 8, 7, 2, 10, 2, 5, 7, 5, 2),
	array<i32, 15>(1, 3, 5, 3, 7, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(0, 8, 7, 0, 7, 1, 1, 7, 5, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(9, 0, 3, 9, 3, 5, 5, 3, 7, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(9, 8, 7, 5, 9, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(5, 8, 4, 5, 10, 8, 10, 11, 8, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(5, 0, 4, 5, 11, 0, 5, 10, 11, 11, 3, 0, -1, -1, -1),
	array<i32, 15>(0, 1, 9, 8, 4, 10, 8, 10, 11, 10, 4, 5, -1, -1, -1),
	array<i32, 15>(10, 11, 4, 10, 4, 5, 11, 3, 4, 9, 4, 1, 3, 1, 4),
	array<i32, 15>(2, 5, 1, 2, 8, 5, 2, 11, 8, 4, 5, 8, -1, -1, -1),
	array<i32, 15>(0, 4, 11, 0, 11, 3, 4, 5, 11, 2, 11, 1, 5, 1, 11),
	array<i32, 15>(0, 2, 5, 0, 5, 9, 2, 11, 5, 4, 5, 8, 11, 8, 5),
	array<i32, 15>(9, 4, 5, 2, 11, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(2, 5, 10, 3, 5, 2, 3, 4, 5, 3, 8, 4, -1, -1, -1),
	array<i32, 15>(5, 10, 2, 5, 2, 4, 4, 2, 0, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(3, 10, 2, 3, 5, 10, 3, 8, 5, 4, 5, 8, 0, 1, 9),
	array<i32, 15>(5, 10, 2, 5, 2, 4, 1, 9, 2, 9, 4, 2, -1, -1, -1),
	array<i32, 15>(8, 4, 5, 8, 5, 3, 3, 5, 1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(0, 4, 5, 1, 0, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(8, 4, 5, 8, 5, 3, 9, 0, 5, 0, 3, 5, -1, -1, -1),
	array<i32, 15>(9, 4, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(4, 11, 7, 4, 9, 11, 9, 10, 11, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(0, 8, 3, 4, 9, 7, 9, 11, 7, 9, 10, 11, -1, -1, -1),
	array<i32, 15>(1, 10, 11, 1, 11, 4, 1, 4, 0, 7, 4, 11, -1, -1, -1),
	array<i32, 15>(3, 1, 4, 3, 4, 8, 1, 10, 4, 7, 4, 11, 10, 11, 4),
	array<i32, 15>(4, 11, 7, 9, 11, 4, 9, 2, 11, 9, 1, 2, -1, -1, -1),
	array<i32, 15>(9, 7, 4, 9, 11, 7, 9, 1, 11, 2, 11, 1, 0, 8, 3),
	array<i32, 15>(11, 7, 4, 11, 4, 2, 2, 4, 0, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(11, 7, 4, 11, 4, 2, 8, 3, 4, 3, 2, 4, -1, -1, -1),
	array<i32, 15>(2, 9, 10, 2, 7, 9, 2, 3, 7, 7, 4, 9, -1, -1, -1),
	array<i32, 15>(9, 10, 7, 9, 7, 4, 10, 2, 7, 8, 7, 0, 2, 0, 7),
	array<i32, 15>(3, 7, 10, 3, 10, 2, 7, 4, 10, 1, 10, 0, 4, 0, 10),
	array<i32, 15>(1, 10, 2, 8, 7, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(4, 9, 1, 4, 1, 7, 7, 1, 3, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(4, 9, 1, 4, 1, 7, 0, 8, 1, 8, 7, 1, -1, -1, -1),
	array<i32, 15>(4, 0, 3, 7, 4, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(4, 8, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(9, 10, 8, 10, 11, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(3, 0, 9, 3, 9, 11, 11, 9, 10, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(0, 1, 10, 0, 10, 8, 8, 10, 11, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(3, 1, 10, 11, 3, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(1, 2, 11, 1, 11, 9, 9, 11, 8, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(3, 0, 9, 3, 9, 11, 1, 2, 9, 2, 11, 9, -1, -1, -1),
	array<i32, 15>(0, 2, 11, 8, 0, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(3, 2, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(2, 3, 8, 2, 8, 10, 10, 8, 9, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(9, 10, 2, 0, 9, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(2, 3, 8, 2, 8, 10, 0, 1, 8, 1, 10, 8, -1, -1, -1),
	array<i32, 15>(1, 10, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(1, 3, 8, 9, 1, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(0, 9, 1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(0, 3, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1),
	array<i32, 15>(-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1),
);
