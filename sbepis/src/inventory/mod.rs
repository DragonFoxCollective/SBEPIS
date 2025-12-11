use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_pretty_nice_menus::show_menu_on_action;
use bevy_rapier3d::prelude::*;
use screen::*;

use crate::player_controller::OpenInventory;
use crate::prelude::*;

mod screen;

#[derive(AutoPlugin)]
#[auto_add_plugin(plugin = SbepisPlugin)]
pub struct InventoryPlugin;

#[auto_plugin(plugin = InventoryPlugin)]
fn build(app: &mut App) {
    app.add_observer(interact_with::<Item>);
    app.add_observer(show_menu_on_action::<OpenInventory, InventoryScreen>);
}

#[auto_component(plugin = InventoryPlugin, derive(Default), reflect, register)]
pub struct Inventory {
    pub items: Vec<Entity>,
}

#[auto_component(plugin = InventoryPlugin, derive, reflect, register)]
pub struct Item {
    pub icon: Handle<Image>,
}

#[auto_event(plugin = InventoryPlugin, target(entity), derive, reflect, register)]
pub struct PickUpItem {
    pub entity: Entity,
}

#[auto_observer(plugin = InventoryPlugin)]
fn pick_up_items(
    interact: On<InteractWith<Item>>,
    mut commands: Commands,
    mut player: Query<(Entity, &mut Inventory)>,
) -> Result {
    let (inventory_entity, mut inventory) = player.single_mut()?;
    inventory.items.push(interact.entity);
    commands
        .entity(interact.entity)
        .remove::<RigidBody>()
        .insert(Visibility::Hidden)
        .insert(ColliderDisabled);
    commands.trigger(PickUpItem {
        entity: interact.entity,
    });
    commands.trigger(ChangeInventory {
        entity: inventory_entity,
    });

    Ok(())
}

#[auto_event(plugin = InventoryPlugin, target(entity), derive, reflect, register)]
pub struct ChangeInventory {
    pub entity: Entity,
}
