use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_rapier3d::prelude::*;
use itertools::Itertools;

use crate::prelude::*;
use crate::util::{IterElements, TransformExt};

#[derive(AutoPlugin)]
#[auto_add_plugin(plugin = SbepisPlugin)]
#[auto_plugin(impl_plugin_trait)]
pub struct GravityPlugin;

#[auto_component(plugin = GravityPlugin, derive, reflect, register)]
#[require(Transform)]
pub struct GravityPoint {
    pub standard_radius: f32,
    pub acceleration_at_radius: f32,
    pub has_volume: bool,
}

#[auto_system(plugin = GravityPlugin, schedule = Update, config(
	before = calculate_gravity,
))]
fn gravity_point(
    mut rigidbodies: Query<(&Transform, &mut ComputedGravity), With<AffectedByGravity>>,
    gravity_fields: Query<(&GlobalTransform, &GravityPriority, &GravityPoint)>,
) {
    for (rigidbody_transform, mut computed_gravity) in rigidbodies.iter_mut() {
        for (gravity_transform, gravity_priority, gravity) in gravity_fields {
            let local_position =
                gravity_transform.inverse_transform_point(rigidbody_transform.translation);
            let acceleration = if gravity.has_volume
                && local_position.length() < gravity.standard_radius
            {
                local_position.length() / gravity.standard_radius
                    * gravity.acceleration_at_radius
                    * -local_position.normalize()
            } else {
                gravity.acceleration_at_radius * gravity.standard_radius * gravity.standard_radius
                    / local_position.length_squared()
                    * -local_position.normalize()
            };
            computed_gravity.elements.push(GravityElement {
                priority: gravity_priority.0,
                priority_factor: Vec3::ONE,
                acceleration,
            });
        }
    }
}

#[auto_component(plugin = GravityPlugin, derive, reflect, register)]
pub struct GlobalGravity {
    pub acceleration: Vec3,
}

#[auto_system(plugin = GravityPlugin, schedule = Update, config(
	before = calculate_gravity,
))]
fn global_gravity(
    mut rigidbodies: Query<&mut ComputedGravity, With<AffectedByGravity>>,
    gravity_fields: Query<(&GravityPriority, &GlobalGravity)>,
) {
    for mut computed_gravity in rigidbodies.iter_mut() {
        for (gravity_priority, gravity) in gravity_fields {
            computed_gravity.elements.push(GravityElement {
                priority: gravity_priority.0,
                priority_factor: Vec3::ONE,
                acceleration: gravity.acceleration,
            });
        }
    }
}

#[auto_component(plugin = GravityPlugin, derive, reflect, register)]
pub struct GravityPriority(pub u32);

#[auto_component(plugin = GravityPlugin, derive(Debug, Default), reflect, register)]
#[require(ComputedGravity, GravityFactor, RigidBody, Velocity)]
pub struct AffectedByGravity;

#[auto_component(plugin = GravityPlugin, derive(Debug), reflect, register)]
pub struct GravityFactor(pub f32);

impl Default for GravityFactor {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Reflect, Debug)]
struct GravityElement {
    priority: u32,
    priority_factor: Vec3,
    acceleration: Vec3,
}

#[auto_component(plugin = GravityPlugin, derive(Debug), reflect, register)]
pub struct ComputedGravity {
    pub acceleration: Vec3,
    pub up: Vec3,
    elements: Vec<GravityElement>,
}

impl Default for ComputedGravity {
    fn default() -> Self {
        Self {
            acceleration: Vec3::ZERO,
            up: Vec3::Y,
            elements: Vec::new(),
        }
    }
}

#[auto_system(plugin = GravityPlugin, schedule = Update)]
fn calculate_gravity(mut rigidbodies: Query<&mut ComputedGravity>) {
    for mut gravity in rigidbodies.iter_mut() {
        let element_groups: Vec<Vec<GravityElement>> = gravity
            .elements
            .drain(..)
            .sorted_by_cached_key(|el| el.priority)
            .chunk_by(|el| el.priority)
            .into_iter()
            .map(|(_, group)| group.collect())
            .collect();

        let acceleration =
            element_groups
                .into_iter()
                .fold(Vec3::ZERO, |lower_priority_acceleration, group| {
                    let unified_priority_factor = group
                        .iter()
                        .map(|el| el.priority_factor.iter_elements().product::<f32>())
                        .sum();
                    let acceleration = group
                        .iter()
                        .map(|el| el.acceleration * el.priority_factor)
                        .sum();
                    Vec3::lerp(
                        lower_priority_acceleration,
                        acceleration,
                        unified_priority_factor,
                    )
                });

        gravity.acceleration = acceleration;
        if let Some(dir) = acceleration.try_normalize() {
            gravity.up = -dir;
        } else if gravity.up == Vec3::ZERO {
            gravity.up = Vec3::Y;
        }
    }
}

#[auto_system(plugin = GravityPlugin, schedule = Update, config(
	after = calculate_gravity,
))]
fn apply_gravity(
    mut rigidbodies: Query<
        (&mut Velocity, &GravityFactor, &ComputedGravity, &RigidBody),
        With<AffectedByGravity>,
    >,
    time: Res<Time>,
) {
    for (mut velocity, gravity_factor, computed_gravity, rigidbody) in rigidbodies.iter_mut() {
        if *rigidbody == RigidBody::Dynamic {
            velocity.linvel += computed_gravity.acceleration * gravity_factor.0 * time.delta_secs();
        }
    }
}
