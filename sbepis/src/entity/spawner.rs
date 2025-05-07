use std::time::Duration;

use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy_butler::*;

use crate::entity::{EntityKilled, EntityKilledSet, EntityPlugin};

#[derive(Component)]
pub struct Spawner {
    pub max_amount: usize,
    pub spawn_delay: Duration,
    pub spawn_timer: Duration,
    pub entities: HashSet<Entity>,
}

#[derive(Event)]
#[add_event(plugin = EntityPlugin)]
pub struct SpawnerActivated {
    pub entity: Entity,
    pub spawner: Entity,
    pub position: Vec3,
}
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpawnerActivatedSet;

#[derive(Event)]
#[add_event(plugin = EntityPlugin)]
pub struct EntitySpawned {
    pub _entity: Entity,
}
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntitySpawnedSet;

#[add_system(
	plugin = EntityPlugin, schedule = Update,
	in_set = SpawnerActivatedSet,
)]
fn spawn_entities(
    mut spawners: Query<(Entity, &mut Spawner, &GlobalTransform)>,
    time: Res<Time>,
    mut ev_spawned: EventWriter<SpawnerActivated>,
    mut commands: Commands,
) {
    for (spawner_entity, mut spawner, transform) in spawners.iter_mut() {
        spawner.spawn_timer += time.delta();

        if spawner.spawn_timer >= spawner.spawn_delay && spawner.entities.len() < spawner.max_amount
        {
            let entity = commands.spawn_empty().id();
            spawner.entities.insert(entity);
            spawner.spawn_timer = Duration::ZERO;
            ev_spawned.write(SpawnerActivated {
                entity,
                spawner: spawner_entity,
                position: transform.translation(),
            });
        }
    }
}

#[add_system(
	plugin = EntityPlugin, schedule = Update,
	in_set = EntityKilledSet,
)]
fn remove_entity(mut spawners: Query<&mut Spawner>, mut ev_killed: EventReader<EntityKilled>) {
    for killed in ev_killed.read() {
        for mut spawner in spawners.iter_mut() {
            spawner.entities.remove(&killed.0);
        }
    }
}
