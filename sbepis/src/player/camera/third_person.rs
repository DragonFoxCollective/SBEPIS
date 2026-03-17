use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;
use bevy_auto_plugin::prelude::*;
use bevy_pretty_nice_input::prelude::{Action, Pressed};
use return_ok::ok_or_continue;

use crate::player::Player;
use crate::player::camera::{CameraOfPlayer, PlayerCameraPlugin, PlayerOfCamera};

#[auto_component(plugin = PlayerCameraPlugin, derive, reflect, register)]
#[require(Transform)]
pub struct HiddenInFirstPerson;

#[derive(EntityEvent)]
struct UpdateHiddenInFirstPerson(Entity);

#[auto_observer(plugin = PlayerCameraPlugin)]
fn hide_first_person_add(
    ready: On<SceneInstanceReady>,
    players: Query<(), With<Player>>,
    mut commands: Commands,
) {
    if players.get(ready.entity).is_err() {
        return;
    }
    commands.trigger(UpdateHiddenInFirstPerson(ready.entity));
}

#[auto_observer(plugin = PlayerCameraPlugin)]
fn hide_first_person(
    hide: On<UpdateHiddenInFirstPerson>,
    children: Query<&Children>,
    mut transforms: Query<&mut Transform, With<HiddenInFirstPerson>>,
    players: Query<&PlayerOfCamera>,
    cameras: Query<&PlayerCameraPositionType>,
) -> Result {
    let scale = match *cameras.get(**players.get(hide.0)?)? {
        PlayerCameraPositionType::FirstPerson => Vec3::ZERO,
        _ => Vec3::ONE,
    };
    for child in [hide.0]
        .into_iter()
        .chain(children.iter_descendants(hide.0))
    {
        let mut transform = ok_or_continue!(transforms.get_mut(child));
        transform.scale = scale;
    }
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

#[auto_component(plugin = PlayerCameraPlugin, derive(Deref), reflect, register)]
#[relationship_target(relationship = FirstPersonOfCamera, linked_spawn)]
pub struct CameraOfFirstPerson(Entity);

#[auto_component(plugin = PlayerCameraPlugin, derive(Deref), reflect, register)]
#[relationship(relationship_target = CameraOfFirstPerson)]
pub struct FirstPersonOfCamera(pub Entity);

#[auto_component(plugin = PlayerCameraPlugin, derive(Deref), reflect, register)]
#[relationship_target(relationship = ThirdPersonOfCamera, linked_spawn)]
pub struct CameraOfThirdPerson(Entity);

#[auto_component(plugin = PlayerCameraPlugin, derive(Deref), reflect, register)]
#[relationship(relationship_target = CameraOfThirdPerson)]
pub struct ThirdPersonOfCamera(pub Entity);

#[auto_system(plugin = PlayerCameraPlugin, schedule = Update)]
fn camera_changed(
    cameras: Query<
        (
            Entity,
            &CameraOfFirstPerson,
            &CameraOfThirdPerson,
            &PlayerCameraPositionType,
            &CameraOfPlayer,
        ),
        Changed<PlayerCameraPositionType>,
    >,
    mut commands: Commands,
) -> Result {
    for (camera, first_person, third_person, position_type, player) in cameras {
        debug!("Changing to {:?}", position_type);
        let new_parent = match position_type {
            PlayerCameraPositionType::FirstPerson => **first_person,
            PlayerCameraPositionType::ThirdPerson => **third_person,
        };
        commands.entity(camera).insert(ChildOf(new_parent));
        commands.trigger(UpdateHiddenInFirstPerson(**player));
    }
    Ok(())
}

#[derive(Action)]
pub struct SwapCameraPosition;

#[auto_observer(plugin = PlayerCameraPlugin)]
fn swap_position(
    swap: On<Pressed<SwapCameraPosition>>,
    mut cameras: Query<(&mut PlayerCameraPositionType, &CameraOfPlayer)>,
    mut commands: Commands,
) -> Result {
    let (mut position_type, player) = cameras.get_mut(swap.input)?;
    position_type.become_next();
    commands.trigger(UpdateHiddenInFirstPerson(**player));
    Ok(())
}
