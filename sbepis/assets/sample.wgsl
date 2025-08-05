@group(0) @binding(0)
var<uniform> chunk_position: vec3<i32>;

@group(0) @binding(1)
var<uniform> num_voxels_per_axis: u32;

@group(0) @binding(2)
var<uniform> num_samples_per_axis: u32;

@group(0) @binding(3)
var<uniform> chunk_size: f32;

@group(0) @binding(4)
var<storage, read_write> densities: array<f32>;

fn coord_to_world(coord: vec3<u32>) -> vec3<f32> {
	return (vec3<f32>(chunk_position) + (vec3<f32>(coord) - vec3<f32>(1.0)) / f32(num_voxels_per_axis)) * chunk_size;
}

fn density_index(coord: vec3<u32>) -> u32 {
	return coord.x * num_samples_per_axis * num_samples_per_axis + coord.y * num_samples_per_axis + coord.z;
}

@compute @workgroup_size(8, 8, 8)
fn main(
	@builtin(global_invocation_id) coord: vec3<u32>
) {
	if coord.x >= num_samples_per_axis || coord.y >= num_samples_per_axis || coord.z >= num_samples_per_axis {
		return;
	}

	densities[density_index(coord)] = sample_noise(coord_to_world(coord) * 0.1);
}

fn sample_noise(coord: vec3<f32>) -> f32 {
	// Simplex noise stolen from fastnoise crate

	var seed = 0i;

	var i = fast_round(coord.x);
	var j = fast_round(coord.y);
	var k = fast_round(coord.z);
	var x0 = coord.x - f32(i);
	var y0 = coord.y - f32(j);
	var z0 = coord.z - f32(k);

	var x_n_sign = i32(-1. - x0) | 1;
	var y_n_sign = i32(-1. - y0) | 1;
	var z_n_sign = i32(-1. - z0) | 1;

	var ax0 = f32(x_n_sign) * -x0;
	var ay0 = f32(y_n_sign) * -y0;
	var az0 = f32(z_n_sign) * -z0;

	i = i * PRIME_X;
	j = j * PRIME_Y;
	k = k * PRIME_Z;

	var value = 0.;
	var a = (0.6 - x0 * x0) - (y0 * y0 + z0 * z0);

	var l = 0;
	loop {
		if a > 0. {
			value += (a * a) * (a * a) * grad_coord_3d(seed, i, j, k, x0, y0, z0);
		}

		if ax0 >= ay0 && ax0 >= az0 {
			let b = a + ax0 + ax0;
			if b > 1. {
				let b = b - 1.;
				value += (b * b)
					* (b * b)
					* grad_coord_3d(
						seed,
						i - x_n_sign * PRIME_X,
						j,
						k,
						x0 + f32(x_n_sign),
						y0,
						z0,
					);
			}
		} else if ay0 > ax0 && ay0 >= az0 {
			let b = a + ay0 + ay0;
			if b > 1. {
				let b = b - 1.;
				value += (b * b)
					* (b * b)
					* grad_coord_3d(
						seed,
						i,
						j - y_n_sign * PRIME_Y,
						k,
						x0,
						y0 + f32(y_n_sign),
						z0,
					);
			}
		} else {
			let b = a + az0 + az0;
			if b > 1. {
				let b = b - 1.;
				value += (b * b)
					* (b * b)
					* grad_coord_3d(
						seed,
						i,
						j,
						k - z_n_sign * PRIME_Z,
						x0,
						y0,
						z0 + f32(z_n_sign),
					);
			}
		}

		if l == 1 {
			break;
		}

		ax0 = 0.5 - ax0;
		ay0 = 0.5 - ay0;
		az0 = 0.5 - az0;

		x0 = f32(x_n_sign) * ax0;
		y0 = f32(y_n_sign) * ay0;
		z0 = f32(z_n_sign) * az0;

		a = a + (0.75 - ax0) - (ay0 + az0);

		i = i + ((x_n_sign >> 1) & PRIME_X);
		j = j + ((y_n_sign >> 1) & PRIME_Y);
		k = k + ((z_n_sign >> 1) & PRIME_Z);

		x_n_sign = -x_n_sign;
		y_n_sign = -y_n_sign;
		z_n_sign = -z_n_sign;

		seed = ~seed;

		l += 1;
	}

	return value * 32.69428253173828125;
}

fn fast_round(f: f32) -> i32 {
	if f >= 0. {
		return i32(f + 0.5);
	} else {
		return i32(f - 0.5);
	}
}

// Primes for hashing
const PRIME_X: i32 = 501125321;
const PRIME_Y: i32 = 1136930381;
const PRIME_Z: i32 = 1720413743;

fn grad_coord_3d(
	seed: i32,
	x_primed: i32,
	y_primed: i32,
	z_primed: i32,
	xd: f32,
	yd: f32,
	zd: f32,
) -> f32 {
	var hash = hash_3d(seed, x_primed, y_primed, z_primed);
	hash = hash ^ (hash >> 15);
	hash = hash & (63 << 2);

	let xg = GRADIENTS_3D[hash];
	let yg = GRADIENTS_3D[(hash | 1)];
	let zg = GRADIENTS_3D[(hash | 2)];

	return xd * xg + yd * yg + zd * zg;
}

fn hash_3d(seed: i32, x_primed: i32, y_primed: i32, z_primed: i32) -> i32 {
	let hash = seed ^ x_primed ^ y_primed ^ z_primed;
	return hash * 0x27d4eb2d;
}

const GRADIENTS_3D: array<f32, 256> = array<f32, 256>(
	0., 1., 1., 0.,  0.,-1., 1., 0.,  0., 1.,-1., 0.,  0.,-1.,-1., 0.,
	1., 0., 1., 0., -1., 0., 1., 0.,  1., 0.,-1., 0., -1., 0.,-1., 0.,
	1., 1., 0., 0., -1., 1., 0., 0.,  1.,-1., 0., 0., -1.,-1., 0., 0.,
	0., 1., 1., 0.,  0.,-1., 1., 0.,  0., 1.,-1., 0.,  0.,-1.,-1., 0.,
	1., 0., 1., 0., -1., 0., 1., 0.,  1., 0.,-1., 0., -1., 0.,-1., 0.,
	1., 1., 0., 0., -1., 1., 0., 0.,  1.,-1., 0., 0., -1.,-1., 0., 0.,
	0., 1., 1., 0.,  0.,-1., 1., 0.,  0., 1.,-1., 0.,  0.,-1.,-1., 0.,
	1., 0., 1., 0., -1., 0., 1., 0.,  1., 0.,-1., 0., -1., 0.,-1., 0.,
	1., 1., 0., 0., -1., 1., 0., 0.,  1.,-1., 0., 0., -1.,-1., 0., 0.,
	0., 1., 1., 0.,  0.,-1., 1., 0.,  0., 1.,-1., 0.,  0.,-1.,-1., 0.,
	1., 0., 1., 0., -1., 0., 1., 0.,  1., 0.,-1., 0., -1., 0.,-1., 0.,
	1., 1., 0., 0., -1., 1., 0., 0.,  1.,-1., 0., 0., -1.,-1., 0., 0.,
	0., 1., 1., 0.,  0.,-1., 1., 0.,  0., 1.,-1., 0.,  0.,-1.,-1., 0.,
	1., 0., 1., 0., -1., 0., 1., 0.,  1., 0.,-1., 0., -1., 0.,-1., 0.,
	1., 1., 0., 0., -1., 1., 0., 0.,  1.,-1., 0., 0., -1.,-1., 0., 0.,
	1., 1., 0., 0.,  0.,-1., 1., 0., -1., 1., 0., 0.,  0.,-1.,-1., 0.,
);