use bevy::prelude::*;
use bevy::render::mesh::CapsuleUvProfile;
use bevy_butler::*;
use bevy_rapier3d::geometry::Collider;

use crate::entity::spawner::{
	EntitySpawned, EntitySpawnedSet, SpawnerActivated, SpawnerActivatedSet,
};
use crate::entity::{Healing, RandomInput, RotateTowardMovement, SpawnHealthBar};
use crate::gridbox_material;
use crate::main_bundles::Mob;
use crate::npcs::NpcPlugin;
use crate::questing::{QuestGiver, SpawnQuestMarker};

use super::name_tags::SpawnNameTag;

#[derive(Component)]
pub struct Consort;

#[derive(Component)]
pub struct ConsortSpawner;

#[system(
	plugin = NpcPlugin, schedule = Update,
	after = SpawnerActivatedSet,
	in_set = EntitySpawnedSet,
)]
fn spawn_consort(
	mut ev_spawner: EventReader<SpawnerActivated>,
	mut ev_spawned: EventWriter<EntitySpawned>,
	spawners: Query<(), With<ConsortSpawner>>,
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	asset_server: Res<AssetServer>,
) {
	for ev in ev_spawner.read() {
		if spawners.get(ev.spawner).is_err() {
			continue;
		}

		commands
			.entity(ev.entity)
			.insert((
				Name::new("Consort"),
				Transform::from_translation(ev.position),
				Mob,
				SpawnHealthBar,
				RandomInput::default(),
				Healing(0.2),
				RotateTowardMovement,
				Consort,
				QuestGiver::default(),
				SpawnQuestMarker,
				SpawnNameTag,
			))
			.with_child((
				Transform::from_translation(Vec3::Y * 0.5),
				Mesh3d(
					meshes.add(
						Capsule3d::new(0.25, 0.5)
							.mesh()
							.rings(1)
							.latitudes(8)
							.longitudes(16)
							.uv_profile(CapsuleUvProfile::Fixed),
					),
				),
				MeshMaterial3d(gridbox_material("magenta", &mut materials, &asset_server)),
				Collider::capsule_y(0.25, 0.25),
			));
		ev_spawned.send(EntitySpawned { _entity: ev.entity });
	}
}
