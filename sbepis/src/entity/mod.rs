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

#[derive(Message)]
#[add_message(plugin = EntityPlugin)]
pub struct Kill(pub Entity);
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityKilledSet;

#[add_system(
	plugin = EntityPlugin, schedule = Update,
	after = EntityKilledSet,
)]
fn kill_entities(mut kill: MessageReader<Kill>, mut commands: Commands) {
    for ev in kill.read() {
        commands.entity(ev.0).despawn();
    }
}
