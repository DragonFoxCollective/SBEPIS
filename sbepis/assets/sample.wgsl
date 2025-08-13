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
	let radius = 1000.0;
	let actual_coord = coord + vec3f(0.0, radius, 0.0);
	let height = length(actual_coord) - radius;
	let surface_coord = normalize(actual_coord) * radius;

	let height_density = sample_height(vec3f(surface_coord) * 0.003, height) * 250 - 150 - height;

	let cheese_caves = fbm(coord * 0.01) + 0.5;

	let spaghetti_caves_a = pow(fbm(coord * 0.004), 2.0);
	let spaghetti_caves_b = pow(fbm(coord * 0.004 + 1), 2.0);
	let spaghetti_caves = pow(spaghetti_caves_a + spaghetti_caves_b, 0.5) - 0.03;

	// return min(height_density, min(cheese_caves * 3000 + height, spaghetti_caves * 2000 + height));
	// return height_density;
	// return min(cheese_caves, spaghetti_caves);
	// return cheese_caves;
	return min(height_density, min(cheese_caves, spaghetti_caves) * 100);
}

fn sample_height(coord: vec3f, height: f32) -> f32 {
	let fbm_noise = fbm(coord) * 0.5 + 0.5;
	let worley_noise = worley(coord);
	let edge_dist = distanceToEdge(worley_noise);
	let sea_level = 0.39;
	let cool_edge_dist = pow(edge_dist, 3.45) * 100 + sea_level;
	let rivers = min(fbm_noise, cool_edge_dist);
	let seas = select(rivers, sea_level, worley_noise.cell < 0.29);
	return seas;
}