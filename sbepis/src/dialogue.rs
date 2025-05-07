use bevy::color::palettes::css;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::camera::PlayerCameraNode;
use crate::input::input_manager_bundle;
use crate::menus::*;

pub struct DialogueInfo {
    pub root: Entity,
    options: Entity,
}

pub fn spawn_dialogue<Input: Actionlike>(
    commands: &mut Commands,
    menu_stack: &mut MenuStack,
    text: String,
    bundle: impl Bundle,
    input_map: InputMap<Input>,
) -> DialogueInfo {
    let root = commands
        .spawn((
            Node {
                margin: UiRect::all(Val::Auto),
                width: Val::Percent(100.0),
                max_width: Val::Px(600.0),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(css::GRAY.into()),
            PlayerCameraNode,
            input_manager_bundle(input_map, false),
            Menu,
            MenuWithMouse,
            MenuWithInputManager,
            MenuDespawnsWhenClosed,
            bundle,
        ))
        .id();

    commands.spawn((
        Text(text),
        TextColor(Color::WHITE),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        Node {
            margin: UiRect::bottom(Val::Px(10.0)),
            ..default()
        },
        ChildOf(root),
    ));

    let options = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(10.0),
                ..default()
            },
            ChildOf(root),
        ))
        .id();

    menu_stack.push(root);

    DialogueInfo { root, options }
}

impl DialogueInfo {
    pub fn add_option(&mut self, commands: &mut Commands, text: String, bundle: impl Bundle) {
        commands
            .spawn((
                Button,
                Node {
                    padding: UiRect::all(Val::Px(10.0)),
                    flex_grow: 1.0,
                    ..default()
                },
                BackgroundColor(css::DARK_GRAY.into()),
                bundle,
                ChildOf(self.options),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text(text),
                    TextColor(Color::WHITE),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                ));
            });
    }
}
