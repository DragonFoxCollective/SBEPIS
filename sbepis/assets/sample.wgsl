#import "noise.wgsl"::{worley, distanceToEdge};

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

	densities[density_index(coord)] = sample_noise(coord_to_world(coord) * 0.05);
}

fn sample_noise(coord: vec3f) -> f32 {
	return distanceToEdge(worley(vec3f(coord.x, 0, coord.z))) - coord.y;
}
