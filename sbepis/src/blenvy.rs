use std::time::Duration;

use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_butler::*;
use bevy_rapier3d::prelude::*;
use return_ok::{ok_or_continue, some_or_continue};

use crate::entity::GelViscosity;
use crate::entity::spawner::Spawner;
use crate::gravity::{AffectedByGravity, GravityPoint, GravityPriority};
use crate::npcs::consort::ConsortSpawner;
use crate::npcs::imp::ImpSpawner;

#[butler_plugin(build(add_plugins(::blenvy::BlenvyPlugin::default())))]
pub struct BlenvyPlugin;

#[derive(Component, Reflect)]
#[reflect(Component)]
#[register_type(plugin = BlenvyPlugin)]
pub struct MeshColliderBlundle;

#[system(
	plugin = BlenvyPlugin, schedule = PreUpdate,
)]
fn create_mesh_collider(
	scenes: Query<Entity, With<MeshColliderBlundle>>,
	children: Query<&Children>,
	meshes: Query<&Mesh3d>,
	mesh_assets: Res<Assets<Mesh>>,
	mut commands: Commands,
) {
	for scene in scenes.iter() {
		let mut num_colliders = 0;

		for child in children.iter_descendants(scene) {
			let mesh = ok_or_continue!(meshes.get(child));
			let mesh = some_or_continue!(mesh_assets.get(&mesh.0));
			let collider = Collider::from_bevy_mesh(mesh, &ComputedColliderShape::default())
				.expect("Couldn't make a mesh collider");
			commands.entity(child).insert(collider);
			num_colliders += 1;
		}

		if num_colliders > 0 {
			commands.entity(scene).remove::<MeshColliderBlundle>();
		}
	}
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[register_type(plugin = BlenvyPlugin)]
pub struct PlanetBlundle {
	pub radius: f32,
	pub gravity: f32,
}

#[system(
	plugin = BlenvyPlugin, schedule = PreUpdate,
)]
fn create_planet(scenes: Query<(Entity, &PlanetBlundle)>, mut commands: Commands) {
	for (scene, planet) in scenes.iter() {
		commands.entity(scene).remove::<PlanetBlundle>().insert((
			RigidBody::Fixed,
			GravityPoint {
				standard_radius: planet.radius,
				acceleration_at_radius: planet.gravity,
			},
			GravityPriority(0),
		));
	}
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[register_type(plugin = BlenvyPlugin)]
pub struct BoxBlundle;

#[system(
	plugin = BlenvyPlugin, schedule = PreUpdate,
)]
fn create_box(scenes: Query<Entity, With<BoxBlundle>>, mut commands: Commands) {
	for scene in scenes.iter() {
		commands.entity(scene).remove::<BoxBlundle>().insert((
			AffectedByGravity::default(),
			Velocity {
				linvel: Vec3::ZERO,
				angvel: Vec3::new(2.5, 3.4, 1.6),
			},
			GelViscosity {
				value: 1.0,
				max: 1.0,
			},
		));
	}
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[register_type(plugin = BlenvyPlugin)]
pub enum SpawnerBlundle {
	Imp,
	Consort,
}

#[system(
	plugin = BlenvyPlugin, schedule = PreUpdate,
)]
fn create_spawner(scenes: Query<(Entity, &SpawnerBlundle)>, mut commands: Commands) {
	for (scene, spawner) in scenes.iter() {
		let mut spawner_commands = commands.entity(scene);

		spawner_commands
			.remove::<SpawnerBlundle>()
			.insert((Spawner {
				max_amount: 5,
				spawn_delay: Duration::from_secs_f32(5.),
				spawn_timer: Duration::ZERO,
				entities: HashSet::new(),
			},));

		match spawner {
			SpawnerBlundle::Imp => {
				spawner_commands.insert(ImpSpawner);
			}
			SpawnerBlundle::Consort => {
				spawner_commands.insert(ConsortSpawner);
			}
		}
	}
}
