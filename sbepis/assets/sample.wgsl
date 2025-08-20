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
var<uniform> poi_positions: array<vec3f, 6>;

@group(0) @binding(6)
var<storage, read_write> poi_positions_final: array<vec3f, 6>;

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

	let height_density = sample_height(surface_coord * 0.003) * 250 - 150 - height;

	let cheese_caves = fbm(coord * 0.01) + 0.5;

	let spaghetti_caves_a = pow(fbm(coord * 0.004), 2.0);
	let spaghetti_caves_b = pow(fbm(coord * 0.004 + 1), 2.0);
	let spaghetti_caves = pow(spaghetti_caves_a + spaghetti_caves_b, 0.5) - 0.03;

	var poi_platforms = -1.0e38;
	for (var i = 0u; i < 6; i++) {
		if poi_positions_final[i].x == 0.0 && poi_positions_final[i].y == 0.0 && poi_positions_final[i].z == 0.0 {
			let poi_surface_position = normalize(poi_positions[i] + vec3f(0.0, radius, 0.0)) * radius;
			let poi_height = sample_height(poi_surface_position * 0.003) * 250 - 150 + radius;
			poi_positions_final[i] = poi_surface_position / radius * poi_height - vec3f(0.0, radius, 0.0);
		}
		
		let relative_y = height - poi_positions_final[i].y;
		if length(coord.xz - poi_positions_final[i].xz) < 10.0 && relative_y < 5.0 {
			if relative_y > 0.0 {
				poi_platforms = -1.0 + fbm(coord * 0.001) * 0.01;
			} else if relative_y > -2.0 {
				poi_platforms = 1.0 + fbm(coord * 0.001) * 0.01;
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
	let fbm_noise = fbm(coord) * 0.5 + 0.5;
	let worley_noise = worley(coord);
	let edge_dist = distanceToEdge(worley_noise);
	let sea_level = 0.39;
	let cool_edge_dist = pow(edge_dist, 3.45) * 100 + sea_level;
	let rivers = min(fbm_noise, cool_edge_dist);
	let seas = select(rivers, sea_level, worley_noise.cell < 0.29);
	return seas;
}