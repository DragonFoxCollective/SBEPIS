use bevy::prelude::*;
use bevy_butler::*;
use bevy_rapier3d::prelude::*;
use screen::*;

use crate::menus::{MenuManipulationSet, OpenMenuBinding};
use crate::player_controller::PlayerAction;
use crate::player_controller::camera_controls::InteractedWithSet;

mod screen;

#[butler_plugin]
pub struct InventoryPlugin;

#[derive(Component, Default)]
pub struct Inventory {
	pub items: Vec<Entity>,
}

#[derive(Component)]
pub struct Item {
	pub icon: Handle<Image>,
}

#[derive(Event)]
#[event(plugin = InventoryPlugin)]
pub struct ItemPickedUp(pub Entity);
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ItemPickedUpSet;

type InteractedWithItemSet = InteractedWithSet<Item>;

#[system(
	plugin = InventoryPlugin, schedule = Update,
	generics = Item,
	in_set = InteractedWithItemSet::default(),
)]
use crate::prelude::interact_with;

#[system(
	plugin = InventoryPlugin, schedule = Update,
	after = InteractedWithItemSet::default(),
	in_set = ItemPickedUpSet,
	in_set = InventoryChangedSet,
)]
fn pick_up_items(
	mut ev_interact: EventReader<InteractedWith<Item>>,
	mut commands: Commands,
	mut player: Query<(Entity, &mut Inventory)>,
	mut ev_picked_up: EventWriter<ItemPickedUp>,
	mut ev_inventory_changed: EventWriter<InventoryChanged>,
) {
	for ev in ev_interact.read() {
		let (inventory_entity, mut inventory) = player.single_mut();
		inventory.items.push(ev.0);
		commands
			.entity(ev.0)
			.remove::<RigidBody>()
			.insert(Visibility::Hidden)
			.insert(ColliderDisabled);
		ev_picked_up.send(ItemPickedUp(ev.0));
		ev_inventory_changed.send(InventoryChanged {
			_inventory: inventory_entity,
		});
	}
}

pub struct OpenInventoryBinding;
impl OpenMenuBinding for OpenInventoryBinding {
	type Action = PlayerAction;
	type Menu = InventoryScreen;
	fn action() -> Self::Action {
		PlayerAction::OpenInventory
	}
}

#[system(
	plugin = InventoryPlugin, schedule = Update,
	generics = OpenInventoryBinding,
	in_set = MenuManipulationSet,
)]
use crate::menus::show_menu_on_action;

#[derive(Event)]
#[event(plugin = InventoryPlugin)]
pub struct InventoryChanged {
	pub _inventory: Entity,
}
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct InventoryChangedSet;

#[event(plugin = InventoryPlugin, generics = Item)]
use crate::player_controller::camera_controls::InteractedWith;
