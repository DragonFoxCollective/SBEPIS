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

	var closest_cluster = 0u;
	var min_distance = 10.0; // definitely greater than sqrt(3)
	for (var i = 0u; i < k; i++) {
		let cluster = i;
		let distance = distance(rgb, clusters[cluster].color);
		if distance < min_distance {
			closest_cluster = cluster;
			min_distance = distance;
		}
	}

	atomicAdd(&clusters[closest_cluster].centroid_sum_r, u32(rgb.r * 256.0));
	atomicAdd(&clusters[closest_cluster].centroid_sum_g, u32(rgb.g * 256.0));
	atomicAdd(&clusters[closest_cluster].centroid_sum_b, u32(rgb.b * 256.0));
	atomicAdd(&clusters[closest_cluster].len_centroid_sum, 1u);

	let min_distance_u = u32(min_distance * 100000.0);
	let old_furthest_distance = atomicMax(&furthest_point.distance, min_distance_u); // ehhhhhhh still very undefined but ok,,,
	if old_furthest_distance < min_distance_u {
		atomicStore(&furthest_point.color_r, u32(rgb.r * 256.0));
		atomicStore(&furthest_point.color_g, u32(rgb.g * 256.0));
		atomicStore(&furthest_point.color_b, u32(rgb.b * 256.0));
	}

	let debug_palette = clusters[u32(in.uv.x * f32(k))].color;
	let cluster_color = clusters[closest_cluster].color;
	let distance_color = vec3<f32>(min_distance);
	let sum_color = vec3<f32>(
		f32(atomicLoad(&clusters[closest_cluster].centroid_sum_r)),
		f32(atomicLoad(&clusters[closest_cluster].centroid_sum_g)),
		f32(atomicLoad(&clusters[closest_cluster].centroid_sum_b)),
	) / f32(atomicLoad(&clusters[closest_cluster].len_centroid_sum)) / 256.0;
	let furthest_color = vec3<f32>(
		f32(atomicLoad(&furthest_point.color_r)),
		f32(atomicLoad(&furthest_point.color_g)),
		f32(atomicLoad(&furthest_point.color_b)),
	) / 256.0;
	let furthest_distance = vec3<f32>(f32(old_furthest_distance) / 256.0);
    return vec4<f32>(select(debug_palette, cluster_color, in.uv.y > 0.1), 1.0);
}
