use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;

use crate::prelude::*;

pub use self::health::{GelViscosity, Healing, SpawnHealthBar};
pub use self::movement::{Movement, RandomInput, RotateTowardMovement, TargetPlayer};
pub use self::orientation::GravityOrientation;

pub mod health;
pub mod movement;
pub mod orientation;
pub mod spawner;

#[derive(AutoPlugin)]
#[auto_add_plugin(plugin = SbepisPlugin)]
#[auto_plugin(impl_plugin_trait)]
pub struct EntityPlugin;

#[auto_event(plugin = EntityPlugin, target(entity), derive, reflect, register)]
pub struct Kill {
    #[event_target]
    pub victim: Entity,
}

#[auto_observer(plugin = EntityPlugin)]
fn kill_entities(kill: On<Kill>, mut commands: Commands) {
    commands.entity(kill.victim).despawn();
}
