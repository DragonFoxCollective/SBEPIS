use bevy::prelude::*;
use bevy_butler::*;
use bevy_rapier3d::prelude::*;
use itertools::Itertools;

use crate::util::{IterElements, TransformEx};

#[butler_plugin]
pub struct GravityPlugin;

#[derive(Component, Reflect)]
#[register_type(plugin = GravityPlugin)]
pub struct GravityPriority(pub u32);

pub trait GravitationalField {
	/// How much this acceleration affects an object, but also how much this priority should override lower priorities.
	fn get_priority_factor_at(&self, local_position: Vec3) -> Vec3;
	fn get_acceleration_at(&self, local_position: Vec3) -> Vec3;
}

#[derive(Component, Reflect)]
#[register_type(plugin = GravityPlugin)]
pub struct GravityPoint {
	pub standard_radius: f32,
	pub acceleration_at_radius: f32,
}

impl GravitationalField for GravityPoint {
	/// Points affect *all* objects, so they will always override lower priorities.
	fn get_priority_factor_at(&self, _local_position: Vec3) -> Vec3 {
		Vec3::ONE
	}

	fn get_acceleration_at(&self, local_position: Vec3) -> Vec3 {
		let mass = self.acceleration_at_radius * self.standard_radius * self.standard_radius;
		mass / -local_position.length_squared() * local_position.normalize()
	}
}

#[derive(Component)]
#[require(RigidBody, Velocity)]
pub struct AffectedByGravity {
	pub factor: f32,
	pub acceleration: Vec3,
	pub up: Vec3,
}

impl Default for AffectedByGravity {
	fn default() -> Self {
		Self {
			factor: 1.0,
			acceleration: Vec3::ZERO,
			up: Vec3::Y,
		}
	}
}

#[system(
	plugin = GravityPlugin, schedule = Update,
)]
fn calculate_gravity(
	mut rigidbodies: Query<(&Transform, &mut AffectedByGravity)>,
	gravity_fields: Query<(&GlobalTransform, &GravityPriority, &GravityPoint)>,
) {
	let field_groups: Vec<Vec<(&GlobalTransform, &GravityPriority, &GravityPoint)>> =
		gravity_fields
			.into_iter()
			.sorted_by_cached_key(|(_, priority, _)| priority.0)
			.chunk_by(|(_, priority, _)| priority.0)
			.into_iter()
			.map(|(_, group)| group.collect())
			.collect();

	for (transform, mut gravity) in rigidbodies.iter_mut() {
		let acceleration =
			field_groups
				.iter()
				.fold(Vec3::ZERO, |lower_priority_acceleration, group| {
					let local_positions: Vec<Vec3> = group
						.iter()
						.map(|(global_transform, _, _)| {
							global_transform.inverse_transform_point(transform.translation)
						})
						.collect();
					let priority_factors: Vec<f32> = group
						.iter()
						.zip(&local_positions)
						.map(|((_, _, field), local_position)| {
							field
								.get_priority_factor_at(*local_position)
								.iter_elements()
								.product()
						})
						.collect();
					let accelerations: Vec<Vec3> = group
						.iter()
						.zip(&local_positions)
						.map(|((transform, _, field), local_position)| {
							transform.transform_vector3(field.get_acceleration_at(*local_position))
						})
						.collect();
					let accelerations: Vec<Vec3> = accelerations
						.into_iter()
						.zip(&priority_factors)
						.map(|(acceleration, priority_factor)| acceleration * *priority_factor)
						.collect();
					Vec3::lerp(
						lower_priority_acceleration,
						accelerations.iter().sum(),
						priority_factors.iter().sum(),
					)
				});

		gravity.acceleration = acceleration;
		gravity.up = -acceleration.normalize_or(Vec3::Y);
	}
}

#[system(
	plugin = GravityPlugin, schedule = Update,
	after = calculate_gravity,
)]
fn apply_gravity(mut rigidbodies: Query<(&mut Velocity, &AffectedByGravity)>, time: Res<Time>) {
	for (mut velocity, gravity) in rigidbodies.iter_mut() {
		velocity.linvel += gravity.acceleration * gravity.factor * time.delta_secs();
	}
}
