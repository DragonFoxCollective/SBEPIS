use bevy::prelude::*;
use bevy_butler::*;
use bevy_rapier3d::prelude::*;
use screen::*;

use crate::menus::{MenuManipulationSystems, OpenMenuBinding};
use crate::player_controller::PlayerAction;
use crate::player_controller::camera_controls::InteractedWithSet;
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

#[derive(Message)]
#[add_message(plugin = InventoryPlugin)]
pub struct PickUpItem(pub Entity);
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ItemPickedUpSet;

type InteractedWithItemSet = InteractedWithSet<Item>;

#[add_system(
	plugin = InventoryPlugin, schedule = Update,
	generics = <Item>,
	in_set = InteractedWithItemSet::default(),
)]
use crate::prelude::interact_with;

#[add_system(
	plugin = InventoryPlugin, schedule = Update,
	after = InteractedWithItemSet::default(),
	in_set = ItemPickedUpSet,
	in_set = InventoryChangedSet,
)]
fn pick_up_items(
    mut interact: MessageReader<InteractWith<Item>>,
    mut commands: Commands,
    mut player: Query<(Entity, &mut Inventory)>,
    mut pick_up: MessageWriter<PickUpItem>,
    mut change_inventory: MessageWriter<ChangeInventory>,
) -> Result {
    for ev in interact.read() {
        let (inventory_entity, mut inventory) = player.single_mut()?;
        inventory.items.push(ev.0);
        commands
            .entity(ev.0)
            .remove::<RigidBody>()
            .insert(Visibility::Hidden)
            .insert(ColliderDisabled);
        pick_up.write(PickUpItem(ev.0));
        change_inventory.write(ChangeInventory {
            _inventory: inventory_entity,
        });
    }
    Ok(())
}

pub struct OpenInventoryBinding;
impl OpenMenuBinding for OpenInventoryBinding {
    type Action = PlayerAction;
    type Menu = InventoryScreen;
    fn action() -> Self::Action {
        PlayerAction::OpenInventory
    }
}

#[add_system(
	plugin = InventoryPlugin, schedule = Update,
	generics = <OpenInventoryBinding>,
	in_set = MenuManipulationSystems,
)]
use crate::menus::show_menu_on_action;

#[derive(Message)]
#[add_message(plugin = InventoryPlugin)]
pub struct ChangeInventory {
    pub _inventory: Entity,
}
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct InventoryChangedSet;

#[add_message(plugin = InventoryPlugin, generics = <Item>)]
use crate::player_controller::camera_controls::InteractWith;
