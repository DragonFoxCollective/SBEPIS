use std::time::Duration;

use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;

use crate::entity::{EntityPlugin, Kill};

#[auto_component(plugin = EntityPlugin, derive, reflect, register)]
pub struct Spawner {
    pub max_amount: usize,
    pub spawn_delay: Duration,
    pub spawn_timer: Duration,
    pub entities: HashSet<Entity>,
}

#[auto_event(plugin = EntityPlugin, target(entity), derive, reflect, register)]
pub struct ActivateSpawner {
    #[event_target]
    pub spawner: Entity,
    pub spawned_entity: Entity,
    pub position: Vec3,
}

#[auto_event(plugin = EntityPlugin, target(entity), derive, reflect, register)]
pub struct Spawn {
    pub entity: Entity,
}

#[auto_system(plugin = EntityPlugin, schedule = Update)]
fn spawn_entities(
    mut spawners: Query<(Entity, &mut Spawner, &GlobalTransform)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (spawner_entity, mut spawner, transform) in spawners.iter_mut() {
        spawner.spawn_timer += time.delta();

        if spawner.spawn_timer >= spawner.spawn_delay && spawner.entities.len() < spawner.max_amount
        {
            let spawned_entity = commands.spawn_empty().id();
            spawner.entities.insert(spawned_entity);
            spawner.spawn_timer = Duration::ZERO;
            commands.trigger(ActivateSpawner {
                spawned_entity,
                spawner: spawner_entity,
                position: transform.translation(),
            });
        }
    }
}

#[auto_observer(plugin = EntityPlugin)]
fn remove_entity(kill: On<Kill>, mut spawners: Query<&mut Spawner>) {
    for mut spawner in spawners.iter_mut() {
        spawner.entities.remove(&kill.victim);
    }
}
