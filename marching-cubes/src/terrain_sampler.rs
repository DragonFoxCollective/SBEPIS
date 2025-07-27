use bevy::prelude::*;
use fastnoise_lite::FastNoiseLite;

pub trait TerrainSampler {
    fn sample_density(&self, position: Vec3) -> f32;
}

pub struct BoxTerrainSampler {
    pub center: Vec3,
    pub size: Vec3,
    pub rotation: Quat,
}

impl TerrainSampler for BoxTerrainSampler {
    fn sample_density(&self, position: Vec3) -> f32 {
        let position = position - self.center;
        let position = self.rotation * position;
        let distance = position.abs();
        let normalized_distance = distance / (self.size * 0.5);
        let max_distance = normalized_distance.max_element();
        1.0 - max_distance
    }
}

impl Default for BoxTerrainSampler {
    fn default() -> Self {
        Self {
            center: Vec3::splat(16.0),
            size: Vec3::splat(8.0),
            rotation: Quat::IDENTITY,
        }
    }
}

pub struct NoiseTerrainSampler(pub FastNoiseLite);

impl TerrainSampler for NoiseTerrainSampler {
    fn sample_density(&self, position: Vec3) -> f32 {
        self.0.get_noise_3d(position.x, position.y, position.z) * 0.5 + 0.5
    }
}

impl Default for NoiseTerrainSampler {
    fn default() -> Self {
        Self(FastNoiseLite::new())
    }
}
