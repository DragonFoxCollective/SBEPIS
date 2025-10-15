use bevy::prelude::*;
use bevy_butler::*;
use bevy_rapier3d::prelude::*;
use screen::*;

use crate::player_controller::OpenInventory;
use crate::prelude::*;

mod screen;

#[butler_plugin]
#[add_plugin(to_plugin = SbepisPlugin)]
pub struct InventoryPlugin;

#[derive(Component, Default)]
pub struct Inventory {
    pub items: Vec<Entity>,
}

#[derive(Component)]
pub struct Item {
    pub icon: Handle<Image>,
}

#[derive(EntityEvent)]
pub struct PickUpItem {
    pub entity: Entity,
}

#[add_observer(plugin = InventoryPlugin, generics = <Item>)]
use crate::prelude::interact_with;

#[add_observer(plugin = InventoryPlugin)]
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

#[add_observer(plugin = InventoryPlugin, generics = <OpenInventory, InventoryScreen>)]
use bevy_pretty_nice_menus::show_menu_on_action;

#[derive(EntityEvent)]
pub struct ChangeInventory {
    pub entity: Entity,
}
