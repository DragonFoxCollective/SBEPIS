use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::SbepisPlugin;
use crate::entity::{GelViscosity, GravityOrientation, Movement};
use crate::gravity::AffectedByGravity;

#[auto_component(plugin = SbepisPlugin, derive, reflect, register)]
#[require(
	Name::new("Box"),
	Collider::cuboid(0.5, 0.5, 0.5),
	AffectedByGravity,
	Velocity {
		linvel: Vec3::ZERO,
		angvel: Vec3::new(2.5, 3.4, 1.6),
	},
	GelViscosity {
		value: 1.0,
		max: 1.0,
	},
)]
pub struct Box;

#[auto_component(plugin = SbepisPlugin, derive, reflect, register)]
#[require(
	Name::new("Mob"),
	AffectedByGravity,
	GravityOrientation,
	Movement,
	LockedAxes::ROTATION_LOCKED,
	GelViscosity {
		value: 3.0,
		max: 6.0,
	},
	Visibility,
)]
pub struct Mob;
