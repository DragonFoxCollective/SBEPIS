#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_pbr::mesh_view_bindings::view
#import bevy_pbr::view_transformations::depth_ndc_to_view_z
#import bevy_render::view::View

struct PostProcessSettings {
	k: u32,
}

struct QuantizeCluster {
    color: vec3<f32>,
    centroid_sum_r: atomic<u32>,
    centroid_sum_g: atomic<u32>,
    centroid_sum_b: atomic<u32>,
	len_centroid_sum: atomic<u32>,
}

struct FurthestPoint {
	color_r: atomic<u32>,
	color_g: atomic<u32>,
	color_b: atomic<u32>,
	distance: atomic<u32>,
}

// @group(0) @binding(0) is `mesh_view_bindings::view`.

@group(0) @binding(1) var screen_texture: texture_2d<f32>;
@group(0) @binding(2) var screen_texture_sampler: sampler;

@group(0) @binding(3) var<uniform> settings: PostProcessSettings;

@group(0) @binding(4) var<storage, read_write> k: u32;
@group(0) @binding(5) var<storage, read_write> clusters: array<QuantizeCluster, 256>;

@group(0) @binding(6) var<storage, read_write> furthest_point: FurthestPoint;

@fragment
fn main(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
	let rgb = textureSample(screen_texture, screen_texture_sampler, in.uv).rgb;

	const candidates_max_length = BAYER_MAP_LENGTH;
	var candidates_current_length = 0u;
	var candidates = array<u32, candidates_max_length>();
	var candidates_sum = vec3<f32>();

	while candidates_current_length < candidates_max_length {
		var candidate_amount = 0u;
		var candidate = 0u;
		var candidate_color = vec3<f32>();
		var candidate_max_amount = max(candidates_current_length, 1u);
		var candidate_distance = -1.0;

		for (var cluster = 0u; cluster < k; cluster++) {
			let cluster_color = clusters[cluster].color;

			for (var cluster_amount = 1u; cluster_amount <= candidate_max_amount; cluster_amount *= 2) {
				let cluster_test = (candidates_sum + cluster_color * f32(cluster_amount)) / (f32(candidates_current_length) + f32(cluster_amount));
				let cluster_distance = distance(rgb, cluster_test);

				if cluster_distance < candidate_distance || candidate_distance < 0.0 {
					candidate_distance = cluster_distance;
					candidate = cluster;
					candidate_color = cluster_color;
					candidate_amount = cluster_amount;
				}
			}
		}

		for (var i = 0u; i < candidate_amount && candidates_current_length < candidates_max_length; i++) {
			candidates[candidates_current_length] = candidate;
			candidates_current_length++;
			candidates_sum += candidate_color;
		}
	}

	// TODO: sort candidates by luminance here

	let bayer_value = f32(BAYER_MAP[(u32(in.position.x) % BAYER_MAP_DIM) + (u32(in.position.y) % BAYER_MAP_DIM) * BAYER_MAP_DIM]) / f32(BAYER_MAP_LENGTH);
	let closest_cluster = candidates[u32(bayer_value * f32(candidates_current_length))];
	let closest_cluster_color = clusters[closest_cluster].color;

	atomicAdd(&clusters[closest_cluster].centroid_sum_r, u32(rgb.r * 256.0));
	atomicAdd(&clusters[closest_cluster].centroid_sum_g, u32(rgb.g * 256.0));
	atomicAdd(&clusters[closest_cluster].centroid_sum_b, u32(rgb.b * 256.0));
	atomicAdd(&clusters[closest_cluster].len_centroid_sum, 1u);


	let debug_palette = clusters[u32(in.uv.x * f32(k))].color;
	let cluster_color = clusters[closest_cluster].color;
    return vec4<f32>(select(debug_palette, cluster_color, in.uv.y > 0.1), 1.0);
}

const BAYER_MAP_DIM = 8u;
const BAYER_MAP_LENGTH = 64u;
const BAYER_MAP = array<u32, BAYER_MAP_LENGTH>(
	 0,48,12,60, 3,51,15,63,
    32,16,44,28,35,19,47,31,
     8,56, 4,52,11,59, 7,55,
    40,24,36,20,43,27,39,23,
     2,50,14,62, 1,49,13,61,
    34,18,46,30,33,17,45,29,
    10,58, 6,54, 9,57, 5,53,
    42,26,38,22,41,25,37,21,
);