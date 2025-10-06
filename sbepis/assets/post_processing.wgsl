#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_pbr::mesh_view_bindings::view
#import bevy_pbr::view_transformations::depth_ndc_to_view_z
#import bevy_render::view::View

struct PostProcessSettings {
    intensity: f32,
	radius: f32,
}

// @group(0) @binding(0) is `mesh_view_bindings::view`.

@group(0) @binding(1) var screen_texture: texture_2d<f32>;
@group(0) @binding(2) var screen_texture_sampler: sampler;

// The depth texture for the main view.
@group(0) @binding(3) var depth_texture: texture_depth_2d;

@group(0) @binding(4) var blur_texture: texture_2d<f32>;

@group(0) @binding(5) var<uniform> settings: PostProcessSettings;

// Performs a single direction of the separable Gaussian blur kernel.
//
// * `frag_coord` is the screen-space pixel coordinate of the fragment (i.e. the
//   `position` input to the fragment).
//
// * `coc` is the diameter (not the radius) of the circle of confusion for this
//   fragment.
//
// * `frag_offset` is the vector, in screen-space units, from one sample to the
//   next. For a horizontal blur this will be `vec2(1.0, 0.0)`; for a vertical
//   blur this will be `vec2(0.0, 1.0)`.
//
// Returns the resulting color of the fragment.
fn gaussian_blur(frag_coord: vec4<f32>, coc: f32, frag_offset: vec2<f32>) -> f32 {
    // Usually σ (the standard deviation) is half the radius, and the radius is
    // half the CoC. So we multiply by 0.25.
    let sigma = coc * 0.25;

    // 1.5σ is a good, somewhat aggressive default for support—the number of
    // texels on each side of the center that we process.
    let support = i32(ceil(sigma * 1.5));
    let uv = frag_coord.xy / vec2<f32>(textureDimensions(screen_texture));
    let offset = frag_offset / vec2<f32>(textureDimensions(screen_texture));

    // The probability density function of the Gaussian blur is (up to constant factors) `exp(-1 / 2σ² *
    // x²). We precalculate the constant factor here to avoid having to
    // calculate it in the inner loop.
    let exp_factor = -1.0 / (2.0 * sigma * sigma);

    // Accumulate samples on both sides of the current texel. Go two at a time,
    // taking advantage of bilinear filtering.
    var sum = textureSampleLevel(blur_texture, screen_texture_sampler, uv, 0.0).r;
    var weight_sum = 1.0;
    for (var i = 1; i <= support; i += 2) {
        // This is a well-known trick to reduce the number of needed texture
        // samples by a factor of two. We seek to accumulate two adjacent
        // samples c₀ and c₁ with weights w₀ and w₁ respectively, with a single
        // texture sample at a carefully chosen location. Observe that:
        //
        //     k ⋅ lerp(c₀, c₁, t) = w₀⋅c₀ + w₁⋅c₁
        //
        //                              w₁
        //     if k = w₀ + w₁ and t = ───────
        //                            w₀ + w₁
        //
        // Therefore, if we sample at a distance of t = w₁ / (w₀ + w₁) texels in
        // between the two texel centers and scale by k = w₀ + w₁ afterward, we
        // effectively evaluate w₀⋅c₀ + w₁⋅c₁ with a single texture lookup.
        let w0 = exp(exp_factor * f32(i) * f32(i));
        let w1 = exp(exp_factor * f32(i + 1) * f32(i + 1));
        let uv_offset = offset * (f32(i) + w1 / (w0 + w1));
        let weight = w0 + w1;

        sum += (
            textureSampleLevel(blur_texture, screen_texture_sampler, uv + uv_offset, 0.0).r +
            textureSampleLevel(blur_texture, screen_texture_sampler, uv - uv_offset, 0.0).r
        ) * weight;
        weight_sum += weight * 2.0;
    }

    return sum / weight_sum;
}

// this is so stupid
@fragment
fn depth_blit(in: FullscreenVertexOutput) -> @location(0) f32 {
    // Sample the depth.
    let frag_coord = vec2<i32>(floor(in.position.xy));
    let raw_depth = textureLoad(depth_texture, frag_coord, 0);
    let depth = -depth_ndc_to_view_z(raw_depth);
	return depth;
}

// Calculates the horizontal component of the separable Gaussian blur.
@fragment
fn blur_horizontal(in: FullscreenVertexOutput) -> @location(0) f32 {
    return gaussian_blur(in.position, settings.radius * 2, vec2(1.0, 0.0));
}

// Calculates the vertical component of the separable Gaussian blur.
@fragment
fn blur_vertical(in: FullscreenVertexOutput) -> @location(0) f32 {
    return gaussian_blur(in.position, settings.radius * 2, vec2(0.0, 1.0));
}

@fragment
fn main(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    // Sample the depth.
    let frag_coord = vec2<i32>(floor(in.position.xy));
    let raw_depth = textureLoad(depth_texture, frag_coord, 0);
    let depth = -depth_ndc_to_view_z(raw_depth);

	let blur_depth = textureSampleLevel(blur_texture, screen_texture_sampler, in.uv, 0.0).r;

	let outside_mask = depth - blur_depth > 0.01;

    return vec4<f32>(
		select(textureSample(screen_texture, screen_texture_sampler, in.uv).rgb, vec3<f32>(), outside_mask),
        1.0
    );
}
