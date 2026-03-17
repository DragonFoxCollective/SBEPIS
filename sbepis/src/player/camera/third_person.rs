use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_pretty_nice_input::prelude::{Action, Pressed};

use crate::player::camera::PlayerCameraPlugin;

#[auto_component(plugin = PlayerCameraPlugin, derive, reflect, register)]
#[require(Transform)]
pub struct HiddenInFirstPerson;

#[auto_observer(plugin = PlayerCameraPlugin)]
fn hide_first_person(
    add: On<Add, HiddenInFirstPerson>,
    mut transforms: Query<&mut Transform>,
) -> Result {
    let mut transform = transforms.get_mut(add.entity)?;
    transform.scale = Vec3::ZERO;
    Ok(())
}

#[auto_component(plugin = PlayerCameraPlugin, derive(Debug), reflect, register)]
pub enum PlayerCameraPositionType {
    FirstPerson,
    ThirdPerson,
}

impl PlayerCameraPositionType {
    pub fn become_next(&mut self) {
        *self = match *self {
            PlayerCameraPositionType::FirstPerson => PlayerCameraPositionType::ThirdPerson,
            PlayerCameraPositionType::ThirdPerson => PlayerCameraPositionType::FirstPerson,
        };
    }
}

#[auto_component(plugin = PlayerCameraPlugin, derive, reflect, register)]
pub struct PlayerCameraPositionEntities {
    pub first_person: Entity,
    pub third_person: Entity,
}

impl PlayerCameraPositionEntities {
    pub fn get(&self, position_type: &PlayerCameraPositionType) -> Entity {
        match position_type {
            PlayerCameraPositionType::FirstPerson => self.first_person,
            PlayerCameraPositionType::ThirdPerson => self.third_person,
        }
    }
}

#[auto_system(plugin = PlayerCameraPlugin, schedule = Update)]
fn camera_changed(
    player_cameras: Query<
        (
            Entity,
            &PlayerCameraPositionEntities,
            &PlayerCameraPositionType,
        ),
        Changed<PlayerCameraPositionType>,
    >,
    mut commands: Commands,
) -> Result {
    for (camera, position_entities, position_type) in player_cameras {
        debug!("Changing to {:?}", position_type);
        commands
            .entity(camera)
            .insert(ChildOf(position_entities.get(position_type)));
    }
    Ok(())
}

#[derive(Action)]
pub struct SwapCameraPosition;

#[auto_observer(plugin = PlayerCameraPlugin)]
fn swap_position(
    swap: On<Pressed<SwapCameraPosition>>,
    mut cameras: Query<&mut PlayerCameraPositionType>,
) -> Result {
    cameras.get_mut(swap.input)?.become_next();
    Ok(())
}
