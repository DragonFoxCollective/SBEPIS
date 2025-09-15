#import "noise.wgsl"::{worley, distanceToEdge, fbm, perlinNoise3};

@group(0) @binding(0)
var<uniform> chunk_position: vec3i;

@group(0) @binding(1)
var<uniform> num_voxels_per_axis: u32;

@group(0) @binding(2)
var<uniform> num_samples_per_axis: u32;

@group(0) @binding(3)
var<uniform> chunk_size: f32;

@group(0) @binding(4)
var<storage, read_write> densities: array<f32>;

@group(0) @binding(5)
var<uniform> poi_positions: array<vec3f, 1>;

@group(0) @binding(6)
var<storage, read_write> poi_positions_final: array<vec3f, 1>;

fn coord_to_world(coord: vec3u) -> vec3f {
	return (vec3f(chunk_position) + (vec3f(coord) - vec3f(1.0)) / f32(num_voxels_per_axis)) * chunk_size;
}

fn density_index(coord: vec3u) -> u32 {
	return coord.x * num_samples_per_axis * num_samples_per_axis + coord.y * num_samples_per_axis + coord.z;
}

@compute @workgroup_size(8, 8, 8)
fn main(
	@builtin(global_invocation_id) coord: vec3u
) {
	if coord.x >= num_samples_per_axis || coord.y >= num_samples_per_axis || coord.z >= num_samples_per_axis {
		return;
	}

	densities[density_index(coord)] = sample_noise(coord_to_world(coord));
}

fn sample_noise(coord: vec3f) -> f32 {
	let height = coord.y;
	let surface_coord = vec3f(coord.x, 0.0, coord.z);

	let sampled_height = sample_height(surface_coord * 0.03) * 5.0;
	let height_density = sampled_height - height;

	for (var i = 0u; i < 1; i++) {
		if poi_positions_final[i].x == 0.0 && poi_positions_final[i].y == 0.0 && poi_positions_final[i].z == 0.0 {
			let poi_surface_coord = vec3f(poi_positions[i].x, 0.0, poi_positions[i].z);
			let poi_height = sample_height(poi_surface_coord * 0.03) * 5.0;
			poi_positions_final[i] = vec3f(poi_positions[i].x, poi_height, poi_positions[i].z);
		}
	}

	var final_density = 1.0e38;
	final_density = min(final_density, height_density);
	return final_density;
}

fn sample_height(coord: vec3f) -> f32 {
	return fbm(coord);
}