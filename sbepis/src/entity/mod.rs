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

#[derive(EntityEvent)]
pub struct Kill {
    #[event_target]
    pub victim: Entity,
}

#[add_observer(plugin = EntityPlugin)]
fn kill_entities(kill: On<Kill>, mut commands: Commands) {
    commands.entity(kill.victim).despawn();
}
