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

@group(0) @binding(0) var<uniform> settings: PostProcessSettings;

@group(0) @binding(1) var<storage, read_write> k: u32;
@group(0) @binding(2) var<storage, read_write> clusters: array<QuantizeCluster, 256>;

@group(0) @binding(3) var<storage, read_write> furthest_point: FurthestPoint;

@compute @workgroup_size(256)
fn main(@builtin(local_invocation_id) local_id: vec3<u32>) {
	if local_id.x >= k {
		return;
	}

	let len = f32(atomicExchange(&clusters[local_id.x].len_centroid_sum, 0u));
	if len > 0 {
		clusters[local_id.x].color = vec3<f32>(
			f32(atomicExchange(&clusters[local_id.x].centroid_sum_r, 0u)) / len / 256.0,
			f32(atomicExchange(&clusters[local_id.x].centroid_sum_g, 0u)) / len / 256.0,
			f32(atomicExchange(&clusters[local_id.x].centroid_sum_b, 0u)) / len / 256.0,
		);
	}
	else if length(clusters[local_id.x].color) > 0.05 { // Not sure why, but it flashes a lot without this
		clusters[local_id.x].color = vec3<f32>(
			f32(atomicLoad(&furthest_point.color_r)) / 256.0,
			f32(atomicLoad(&furthest_point.color_g)) / 256.0,
			f32(atomicLoad(&furthest_point.color_b)) / 256.0,
		);
	}
	atomicStore(&furthest_point.distance, 0u);
}
