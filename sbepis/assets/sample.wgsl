#import "noise.wgsl"::{worley, distanceToEdge, fbm, perlinNoise3};

struct MeshSettings {
    num_voxels_per_axis: u32,
    num_samples_per_axis: u32,
    chunk_size: f32,
    surface_threshold: f32,
}

@group(0) @binding(0)
var<uniform> chunk_position: vec3<i32>;

@group(0) @binding(1)
var<uniform> settings: MeshSettings;

@group(0) @binding(2)
var<storage, read_write> densities: array<f32>;

@group(0) @binding(3)
var<uniform> poi_positions: array<vec3f, 6>;

@group(0) @binding(4)
var<storage, read_write> poi_positions_final: array<vec3f, 6>;

fn coord_to_world(coord: vec3u) -> vec3f {
	return (vec3f(chunk_position) + (vec3f(coord) - vec3f(1.0)) / f32(settings.num_voxels_per_axis)) * settings.chunk_size;
}

fn density_index(coord: vec3u) -> u32 {
	return coord.x * settings.num_samples_per_axis * settings.num_samples_per_axis + coord.y * settings.num_samples_per_axis + coord.z;
}

@compute @workgroup_size(8, 8, 8)
fn main(
	@builtin(global_invocation_id) coord: vec3u
) {
	if coord.x >= settings.num_samples_per_axis || coord.y >= settings.num_samples_per_axis || coord.z >= settings.num_samples_per_axis {
		return;
	}

	densities[density_index(coord)] = sample_noise(coord_to_world(coord));
}

fn sample_noise(coord: vec3f) -> f32 {
	let radius = 1000.0;
	let actual_coord = coord + vec3f(0.0, radius, 0.0);
	let height_coord = length(actual_coord) - radius; // 0 at sea level
	let surface_coord = normalize(actual_coord) * radius;
	let sampled_height = sample_height(surface_coord); // 0 at sea level
	let height_density = sampled_height - height_coord;

	let cheese_caves = fbm(coord * 0.01) + 0.5;

	let spaghetti_caves_a = pow(fbm(coord * 0.004), 2.0);
	let spaghetti_caves_b = pow(fbm(coord * 0.004 + 1), 2.0);
	let spaghetti_caves = pow(spaghetti_caves_a + spaghetti_caves_b, 0.5) - 0.03;

	var poi_platforms = -1.0e38;
	for (var i = 0u; i < 6; i++) {
		if poi_positions_final[i].x == 0.0 && poi_positions_final[i].y == 0.0 && poi_positions_final[i].z == 0.0 {
			let poi_actual_coord = poi_positions[i] + vec3f(0.0, radius, 0.0);
			let poi_height_coord = length(poi_actual_coord) - radius; // 0 at sea level
			let poi_surface_coord = normalize(poi_actual_coord) * radius;
			let poi_sampled_height = sample_height(poi_surface_coord);  // 0 at sea level
			poi_positions_final[i] = normalize(poi_actual_coord) * (poi_sampled_height + radius) - vec3f(0.0, radius, 0.0);
		}
		
		{
			let poi_actual_coord = poi_positions_final[i] + vec3f(0.0, radius, 0.0);
			let poi_surface_coord = normalize(poi_actual_coord) * radius;
			let poi_height_coord = length(poi_actual_coord) - radius; // 0 at sea level

			let relative_surface_coord = surface_coord - poi_surface_coord;
			let relative_height = height_coord - poi_height_coord;
			if length(relative_surface_coord) < 10.0 && relative_height < 5.0 && relative_height > -2.0 {
				poi_platforms = -relative_height + fbm(coord * 0.001) * 0.01;
			}
		}
	}

	var final_density = 1.0e38;
	final_density = min(final_density, height_density);
	final_density = min(final_density, cheese_caves * 100);
	final_density = min(final_density, spaghetti_caves * 100);
	if poi_platforms > -1.0e38 {
		final_density = poi_platforms; 
	}
	return final_density;
}

fn sample_height(coord: vec3f) -> f32 {
	let scaled_coord = coord * 0.003;
	let fbm_noise = fbm(scaled_coord) * 0.5 + 0.5;
	let worley_noise = worley(scaled_coord);
	let edge_dist = distanceToEdge(worley_noise);
	let sea_level = 0.39;
	let cool_edge_dist = pow(edge_dist, 3.45) * 100 + sea_level;
	let rivers = min(fbm_noise, cool_edge_dist);
	let seas = select(rivers, sea_level, worley_noise.cell < 0.29);
	let final_height = seas * 250 - 150;
	return final_height;
}