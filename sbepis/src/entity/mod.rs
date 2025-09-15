use bevy::prelude::*;
use bevy_butler::*;

use crate::prelude::*;

pub use self::health::{GelViscosity, Healing, SpawnHealthBar};
pub use self::movement::{Movement, RandomInput, RotateTowardMovement, TargetPlayer};
pub use self::orientation::GravityOrientation;

pub mod health;
pub mod movement;
pub mod orientation;
pub mod spawner;

#[butler_plugin]
#[add_plugin(to_plugin = SbepisPlugin)]
pub struct EntityPlugin;

#[derive(Event)]
#[add_event(plugin = EntityPlugin)]
pub struct EntityKilled(pub Entity);
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityKilledSet;

#[add_system(
	plugin = EntityPlugin, schedule = Update,
	after = EntityKilledSet,
)]
fn kill_entities(mut ev_killed: EventReader<EntityKilled>, mut commands: Commands) {
    for ev in ev_killed.read() {
        commands.entity(ev.0).despawn();
    }
}
