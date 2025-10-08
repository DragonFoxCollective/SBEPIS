use std::time::Duration;

use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy_butler::*;

use crate::entity::{EntityPlugin, Kill};

#[derive(Component)]
pub struct Spawner {
    pub max_amount: usize,
    pub spawn_delay: Duration,
    pub spawn_timer: Duration,
    pub entities: HashSet<Entity>,
}

#[derive(Message)]
#[add_message(plugin = EntityPlugin)]
pub struct ActivateSpawner {
    pub entity: Entity,
    pub spawner: Entity,
    pub position: Vec3,
}
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpawnerActivatedSet;

#[derive(Message)]
#[add_message(plugin = EntityPlugin)]
pub struct Spawn {
    pub _entity: Entity,
}
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpawnSystems;

#[add_system(
	plugin = EntityPlugin, schedule = Update,
	in_set = SpawnerActivatedSet,
)]
fn spawn_entities(
    mut spawners: Query<(Entity, &mut Spawner, &GlobalTransform)>,
    time: Res<Time>,
    mut activate_spawner: MessageWriter<ActivateSpawner>,
    mut commands: Commands,
) {
    for (spawner_entity, mut spawner, transform) in spawners.iter_mut() {
        spawner.spawn_timer += time.delta();

        if spawner.spawn_timer >= spawner.spawn_delay && spawner.entities.len() < spawner.max_amount
        {
            let entity = commands.spawn_empty().id();
            spawner.entities.insert(entity);
            spawner.spawn_timer = Duration::ZERO;
            activate_spawner.write(ActivateSpawner {
                entity,
                spawner: spawner_entity,
                position: transform.translation(),
            });
        }
    }
}

#[add_observer(plugin = EntityPlugin)]
fn remove_entity(kill: On<Kill>, mut spawners: Query<&mut Spawner>) {
    for mut spawner in spawners.iter_mut() {
        spawner.entities.remove(&kill.victim);
    }
}
