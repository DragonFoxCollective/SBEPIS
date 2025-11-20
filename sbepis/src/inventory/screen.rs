use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy_butler::*;
use bevy_pretty_nice_input::{binding1d, input};
use bevy_pretty_nice_menus::{
    CloseMenuAction, Menu, MenuHidesWhenClosed, MenuWithInput, MenuWithMouse,
};

use crate::camera::PlayerCameraNode;
use crate::inventory::{InventoryPlugin, Item, PickUpItem};

#[derive(Component)]
pub struct InventoryScreen;

#[add_system(
	plugin = InventoryPlugin, schedule = Startup,
)]
fn spawn_inventory_screen(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                margin: UiRect::all(Val::Px(10.0)),
                row_gap: Val::Px(10.0),
                column_gap: Val::Px(10.0),
                flex_wrap: FlexWrap::Wrap,
                ..default()
            },
            BackgroundColor(css::GRAY.with_alpha(0.5).into()),
            Visibility::Hidden,
            input!(CloseMenuAction, Axis1D[binding1d::key(KeyCode::KeyV)]),
            PlayerCameraNode,
            Menu,
            MenuWithMouse,
            MenuWithInput,
            MenuHidesWhenClosed,
            InventoryScreen,
        ))
        .insert(Name::new("Inventory Screen"));
}

#[add_observer(plugin = InventoryPlugin)]
fn add_item_to_inventory_screen(
    pick_up: On<PickUpItem>,
    mut commands: Commands,
    items: Query<&Item>,
    inventory_screen: Query<Entity, With<InventoryScreen>>,
) -> Result {
    let inventory_screen = inventory_screen.single()?;

    let item = items.get(pick_up.entity)?;

    commands.spawn((
        ImageNode::new(item.icon.clone()),
        Node {
            width: Val::Px(100.0),
            height: Val::Px(100.0),
            ..default()
        },
        BackgroundColor(css::DARK_GRAY.into()),
        ChildOf(inventory_screen),
    ));

    Ok(())
}
