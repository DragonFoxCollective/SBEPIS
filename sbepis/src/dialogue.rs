use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy_pretty_nice_input::prelude::*;
use bevy_pretty_nice_menus::{MenuDespawnsWhenClosed, MenuStack, MenuWithInput, MenuWithMouse};

use crate::camera::PlayerCameraNode;

pub struct DialogueInfo {
    pub root: Entity,
    options: Entity,
}

pub fn spawn_dialogue(
    commands: &mut Commands,
    menu_stack: &mut MenuStack,
    text: String,
    bundle: impl Bundle,
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
            MenuWithMouse,
            MenuWithInput,
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
    pub fn add_option<'a, 'b>(
        &mut self,
        commands: &'b mut Commands,
        text: String,
        bundle: impl Bundle,
        event: impl Event<Trigger<'a>: Default> + Clone,
    ) -> EntityCommands<'b> {
        let mut commands = commands.spawn((
            Button,
            Node {
                padding: UiRect::all(Val::Px(10.0)),
                flex_grow: 1.0,
                ..default()
            },
            BackgroundColor(css::DARK_GRAY.into()),
            bundle,
            ChildOf(self.options),
        ));
        let event_2 = event.clone();
        commands
            .with_children(|parent| {
                parent.spawn((
                    Text(text),
                    TextColor(Color::WHITE),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                ));
            })
            .observe(
                move |_: On<JustPressed<PickDialogueOption>>, mut commands: Commands| {
                    commands.trigger(event_2.clone());
                },
            )
            .observe(move |_: On<Pointer<Click>>, mut commands: Commands| {
                commands.trigger(event.clone());
            });
        commands
    }
}

#[derive(Action)]
pub struct PickDialogueOption;
