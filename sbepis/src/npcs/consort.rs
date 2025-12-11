use bevy::mesh::CapsuleUvProfile;
use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_rapier3d::geometry::Collider;

use crate::entity::spawner::{ActivateSpawner, Spawn};
use crate::entity::{Healing, RandomInput, RotateTowardMovement, SpawnHealthBar};
use crate::gridbox_material;
use crate::main_bundles::Mob;
use crate::npcs::NpcPlugin;
use crate::questing::{QuestGiver, SpawnQuestMarker};

use super::name_tags::SpawnNameTag;

#[auto_component(plugin = NpcPlugin, derive, reflect, register)]
pub struct Consort;

#[auto_component(plugin = NpcPlugin, derive, reflect, register)]
pub struct ConsortSpawner;

#[auto_observer(plugin = NpcPlugin)]
fn spawn_consort(
    spawn: On<ActivateSpawner>,
    spawners: Query<(), With<ConsortSpawner>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    if spawners.get(spawn.spawner).is_err() {
        return;
    }

    commands
        .entity(spawn.spawned_entity)
        .insert((
            Name::new("Consort"),
            Transform::from_translation(spawn.position),
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
    commands.trigger(Spawn {
        entity: spawn.spawned_entity,
    });
}
