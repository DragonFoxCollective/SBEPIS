use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_pretty_nice_input::prelude::*;
use bevy_pretty_nice_menus::{CloseMenuAction, MenuHidesWhenClosed, MenuWithInput, MenuWithMouse};

use crate::prelude::*;
use crate::questing::{AcceptQuest, EndQuest, QuestId, QuestingPlugin, Quests};
use crate::util::MapRangeBetween;

#[auto_component(plugin = QuestingPlugin, derive, reflect, register)]
pub struct QuestScreen;

#[auto_component(plugin = QuestingPlugin, derive, reflect, register)]
pub struct QuestScreenNodeList;

#[auto_component(plugin = QuestingPlugin, derive, reflect, register)]
pub struct QuestScreenNodeDisplay(Option<Entity>);

#[auto_component(plugin = QuestingPlugin, derive, reflect, register)]
pub struct QuestScreenNode {
    pub quest_id: QuestId,
    pub display: Entity,
    pub progress_text: Entity,
    pub progress_bar: Entity,
}

#[auto_system(plugin = QuestingPlugin, schedule = Startup)]
fn spawn_quest_screen(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(bevy::color::palettes::css::GRAY.with_alpha(0.5).into()),
            Visibility::Hidden,
            input!(CloseMenuAction, Axis1D[binding1d::key(KeyCode::KeyJ)]),
            PlayerCameraNode,
            MenuWithMouse,
            MenuWithInput,
            MenuHidesWhenClosed,
            QuestScreen,
        ))
        .insert(Name::new("Quest Screen"))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                QuestScreenNodeList,
            ));
            parent.spawn((
                Node {
                    width: Val::Px(2.0),
                    ..default()
                },
                BackgroundColor(css::WHITE.into()),
            ));
            parent.spawn((
                Node {
                    flex_grow: 4.0,
                    ..default()
                },
                QuestScreenNodeDisplay(None),
            ));
        });
}

#[auto_observer(plugin = QuestingPlugin)]
fn add_quest_nodes(
    accept_quest: On<AcceptQuest>,
    mut commands: Commands,
    quests: Res<Quests>,
    quest_screen_node_list: Query<Entity, With<QuestScreenNodeList>>,
    quest_screen_node_display: Query<Entity, With<QuestScreenNodeDisplay>>,
) -> Result {
    let quest_screen_node_list = quest_screen_node_list.single()?;
    let quest_screen_node_display = quest_screen_node_display.single()?;

    let quest_id = accept_quest.quest_id;
    let quest = quests.0.get(&quest_id).ok_or("Unknown quest")?;

    let mut progress_text: Option<Entity> = None;
    let mut progress_bar: Option<Entity> = None;

    let display = commands
        .spawn((
            Node {
                display: bevy::ui::Display::None,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ChildOf(quest_screen_node_display),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text(quest.description.clone()),
                TextColor(Color::WHITE),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
            ));
            progress_text = Some(
                parent
                    .spawn((
                        Text(format!(
                            "{}/{}",
                            quest.quest_type.progress(),
                            quest.quest_type.max_progress()
                        )),
                        TextColor(Color::WHITE),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                    ))
                    .id(),
            );
            parent
                .spawn((
                    Node {
                        height: Val::Px(30.0),
                        width: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(css::DARK_GRAY.into()),
                ))
                .with_children(|parent| {
                    progress_bar = Some(
                        parent
                            .spawn((
                                Node {
                                    width: Val::Percent(0.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                BackgroundColor(css::LIGHT_GRAY.into()),
                            ))
                            .id(),
                    );
                });
        })
        .id();

    commands
        .spawn((
            Button,
            Node {
                padding: UiRect::all(Val::Px(10.0)),
                width: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(css::GRAY.into()),
            QuestScreenNode {
                quest_id,
                display,
                progress_text: progress_text.unwrap(),
                progress_bar: progress_bar.unwrap(),
            },
            ChildOf(quest_screen_node_list),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text(quest.name.clone()),
                TextColor(Color::WHITE),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
            ));
        });

    Ok(())
}

#[auto_observer(plugin = QuestingPlugin)]
fn remove_quest_nodes(
    end: On<EndQuest>,
    mut commands: Commands,
    quest_nodes: Query<(Entity, &QuestScreenNode)>,
) {
    for (quest_node_entity, quest_node) in quest_nodes
        .iter()
        .filter(|(_, node)| node.quest_id == end.quest_id)
    {
        commands.entity(quest_node_entity).despawn();
        commands.entity(quest_node.display).despawn();
    }
}

#[auto_system(plugin = QuestingPlugin, schedule = Update)]
fn change_displayed_node(
    quest_nodes: Query<(&QuestScreenNode, &Interaction), Changed<Interaction>>,
    mut quest_node_displays: Query<&mut Node>,
    mut quest_screen_node_display: Query<&mut QuestScreenNodeDisplay>,
) -> Result {
    let mut quest_screen_node_display = quest_screen_node_display.single_mut()?;

    for (quest_node, &interaction) in quest_nodes.iter() {
        if interaction == Interaction::Pressed {
            if let Some(mut style) = quest_screen_node_display
                .0
                .and_then(|e| quest_node_displays.get_mut(e).ok())
            {
                style.display = bevy::ui::Display::None;
            }

            if let Ok(mut style) = quest_node_displays.get_mut(quest_node.display) {
                style.display = bevy::ui::Display::DEFAULT;
                quest_screen_node_display.0 = Some(quest_node.display);
            }
        }
    }

    Ok(())
}

#[auto_system(plugin = QuestingPlugin, schedule = Update, config(
	after = QuestProgressUpdatedSet,
))]
fn update_quest_node_progress(
    quests: Res<Quests>,
    mut quest_nodes: Query<&QuestScreenNode>,
    mut progress_texts: Query<&mut Text>,
    mut progress_bars: Query<&mut Node>,
) -> Result {
    if !quests.is_changed() {
        return Ok(());
    }

    for quest_node in quest_nodes.iter_mut() {
        let quest = quests.0.get(&quest_node.quest_id).ok_or("Unknown quest")?;
        let mut progress_text = progress_texts.get_mut(quest_node.progress_text).unwrap();
        let mut progress_bar = progress_bars.get_mut(quest_node.progress_bar).unwrap();

        progress_text.0 = format!(
            "{}/{}",
            quest.quest_type.progress(),
            quest.quest_type.max_progress()
        );
        progress_bar.width = Val::Percent(
            (quest.quest_type.progress() as f32)
                .map_range_between(quest.quest_type.progress_range(), 0.0..100.0),
        );
    }

    Ok(())
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuestProgressUpdatedSet;
