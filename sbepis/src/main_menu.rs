use bevy::audio::PlaybackMode;
use bevy::prelude::*;
use bevy_butler::*;
use serde::Deserialize;

use crate::camera::PlayerCamera;
use crate::prelude::*;

#[butler_plugin]
#[add_plugin(to_plugin = SbepisPlugin)]
struct MainMenuPlugin;

#[insert_state(plugin = MainMenuPlugin)]
#[derive(States, Debug, Default, Clone, PartialEq, Eq, Hash)]
#[states(scoped_entities)]
pub enum GameState {
    #[default]
    MainMenu,
    InGame,
}

#[add_sub_state(plugin = MainMenuPlugin)]
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
#[source(GameState = GameState::MainMenu)]
#[states(scoped_entities)]
pub enum MenuState {
    #[default]
    Home,
    Credits,
}

#[add_system(plugin = MainMenuPlugin, schedule = Startup)]
fn setup_global(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(DenySound(asset_server.load("deny.mp3")));
}

#[add_system(plugin = MainMenuPlugin, schedule = OnEnter(GameState::MainMenu))]
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Name::new("cuuuuuube"),
        Mesh3d(meshes.add(Cuboid::new(4.0, 4.0, 4.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.7, 0.6),
            reflectance: 0.02,
            unlit: false,
            ..default()
        })),
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
        StateScoped(GameState::MainMenu),
    ));

    commands.spawn((
        Name::new("Light"),
        PointLight::default(),
        Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
        StateScoped(GameState::MainMenu),
    ));

    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        Transform::from_translation(Vec3::new(0.0, 0.0, 15.0)).looking_at(Vec3::ZERO, Vec3::Y),
        StateScoped(GameState::MainMenu),
        PlayerCamera,
    ));

    commands.spawn((
        Name::new("BGM"),
        StateScoped(GameState::MainMenu),
        AudioPlayer::new(asset_server.load("crystalanthemums remix remix remix remix.mp3")),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            ..default()
        },
    ));
}

#[add_system(plugin = MainMenuPlugin, schedule = OnEnter(MenuState::Home))]
fn setup_home(
    mut commands: Commands,
    title_font: Option<Res<TitleFont>>,
    asset_server: Res<AssetServer>,
) -> Result {
    let font_size = 32.0;

    let title_font = match title_font {
        Some(font) => font.0.clone(),
        None => {
            let font = asset_server.load("Motenacity.ttf");
            commands.insert_resource(TitleFont(font.clone()));
            font
        }
    };

    let menu_root = commands
        .spawn((
            Name::new("Menu root"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            PlayerCameraNode,
            StateScoped(MenuState::Home),
        ))
        .id();

    let content = commands
        .spawn((
            Node {
                height: Val::Percent(100.0),
                aspect_ratio: Some(2.0 / 3.0),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            },
            ChildOf(menu_root),
        ))
        .id();

    commands.spawn((
        Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_grow: 3.0,
            ..default()
        },
        ChildOf(content),
        children![(
            Text::new("SBEPIS"),
            TextFont {
                font: title_font.clone(),
                font_size: 160.0,
                ..default()
            },
            TextLayout {
                justify: JustifyText::Center,
                ..default()
            },
            TextColor(Color::from(Srgba::hex("03a9f4")?)),
            TextShadow {
                offset: Vec2::new(2.0, 2.0),
                color: Color::from(Srgba::hex("000000")?),
            },
        )],
    ));
    commands
        .spawn((
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_grow: 1.0,
                ..default()
            },
            ChildOf(content),
            Button,
            children![(
                Text::new("Germinate Session"),
                TextFont {
                    font_size,
                    ..default()
                },
                TextLayout {
                    justify: JustifyText::Center,
                    ..default()
                },
            )],
        ))
        .observe(
            |_: Trigger<Pointer<Click>>, mut next_state: ResMut<NextState<GameState>>| {
                next_state.set(GameState::InGame);
            },
        );
    commands
        .spawn((
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_grow: 1.0,
                ..default()
            },
            ChildOf(content),
            Button,
            UselessButton,
            children![(
                Text::new("Join Session"),
                TextFont {
                    font_size,
                    ..default()
                },
                TextLayout {
                    justify: JustifyText::Center,
                    ..default()
                },
            )],
        ))
        .observe(
            |_: Trigger<Pointer<Click>>, mut commands: Commands, deny_sound: Res<DenySound>| {
                commands.spawn((
                    Name::new("Deny Sound"),
                    AudioPlayer::new(deny_sound.0.clone()),
                ));
            },
        );
    commands
        .spawn((
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_grow: 1.0,
                ..default()
            },
            ChildOf(content),
            Button,
            UselessButton,
            children![(
                Text::new("Advancement Database"),
                TextFont {
                    font_size,
                    ..default()
                },
                TextLayout {
                    justify: JustifyText::Center,
                    ..default()
                },
            )],
        ))
        .observe(
            |_: Trigger<Pointer<Click>>, mut commands: Commands, deny_sound: Res<DenySound>| {
                commands.spawn((
                    Name::new("Deny Sound"),
                    AudioPlayer::new(deny_sound.0.clone()),
                ));
            },
        );
    commands
        .spawn((
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_grow: 1.0,
                ..default()
            },
            ChildOf(content),
            Button,
            UselessButton,
            children![(
                Text::new("Settings"),
                TextFont {
                    font_size,
                    ..default()
                },
                TextLayout {
                    justify: JustifyText::Center,
                    ..default()
                },
            )],
        ))
        .observe(
            |_: Trigger<Pointer<Click>>, mut commands: Commands, deny_sound: Res<DenySound>| {
                commands.spawn((
                    Name::new("Deny Sound"),
                    AudioPlayer::new(deny_sound.0.clone()),
                ));
            },
        );
    commands
        .spawn((
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_grow: 1.0,
                ..default()
            },
            ChildOf(content),
            Button,
            children![(
                Text::new("About System"),
                TextFont {
                    font_size,
                    ..default()
                },
                TextLayout {
                    justify: JustifyText::Center,
                    ..default()
                },
            )],
        ))
        .observe(
            |_: Trigger<Pointer<Click>>, mut state: ResMut<NextState<MenuState>>| {
                state.set(MenuState::Credits);
            },
        );
    commands
        .spawn((
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_grow: 1.0,
                ..default()
            },
            ChildOf(content),
            Button,
            children![(
                Text::new("End Connection"),
                TextFont {
                    font_size,
                    ..default()
                },
                TextLayout {
                    justify: JustifyText::Center,
                    ..default()
                },
            )],
        ))
        .observe(
            |_: Trigger<Pointer<Click>>, mut ev_exit: EventWriter<AppExit>| {
                ev_exit.write(AppExit::Success);
            },
        );

    Ok(())
}

#[add_system(plugin = MainMenuPlugin, schedule = OnEnter(MenuState::Credits))]
fn setup_credits(
    mut commands: Commands,
    main_menu_names: Res<MainMenuNames>,
    supporters: Res<Assets<Supporters>>,
    developers: Res<Assets<Developers>>,
) -> Result {
    let (supporters, developers) = match (
        supporters.get(&main_menu_names.supporters),
        developers.get(&main_menu_names.developers),
    ) {
        (Some(supporters), Some(developers)) => (supporters, developers),
        _ => {
            commands.spawn(Text::new("Couldn't load credits :("));
            return Ok(());
        }
    };

    let mechanics_names = developers
        .names
        .iter()
        .filter(|s| s.area == DeveloperArea::Mechanics)
        .map(|s| s.name.clone())
        .collect::<Vec<_>>();
    let programming_names = developers
        .names
        .iter()
        .filter(|s| s.area == DeveloperArea::Programming)
        .map(|s| s.name.clone())
        .collect::<Vec<_>>();
    let art_names = developers
        .names
        .iter()
        .filter(|s| s.area == DeveloperArea::Art)
        .map(|s| s.name.clone())
        .collect::<Vec<_>>();
    let music_names = developers
        .names
        .iter()
        .filter(|s| s.area == DeveloperArea::Music)
        .map(|s| s.name.clone())
        .collect::<Vec<_>>();
    let documentation_names = developers
        .names
        .iter()
        .filter(|s| s.area == DeveloperArea::Documentation)
        .map(|s| s.name.clone())
        .collect::<Vec<_>>();
    let contributor_names = developers
        .names
        .iter()
        .filter(|s| s.area == DeveloperArea::Contributor)
        .map(|s| s.name.clone())
        .collect::<Vec<_>>();

    let past_names = supporters
        .names
        .iter()
        .filter(|s| s.tier == SupporterTier::Past)
        .map(|s| s.name.clone())
        .collect::<Vec<_>>();
    let pgo_names = supporters
        .names
        .iter()
        .filter(|s| s.tier == SupporterTier::Pgo)
        .map(|s| s.name.clone())
        .collect::<Vec<_>>();
    let captcha_names = supporters
        .names
        .iter()
        .filter(|s| s.tier == SupporterTier::Captcha)
        .map(|s| s.name.clone())
        .collect::<Vec<_>>();
    let alchemiter_names = supporters
        .names
        .iter()
        .filter(|s| s.tier == SupporterTier::Alchemiter)
        .map(|s| s.name.clone())
        .collect::<Vec<_>>();
    let denizen_names = supporters
        .names
        .iter()
        .filter(|s| s.tier == SupporterTier::Denizen)
        .map(|s| s.name.clone())
        .collect::<Vec<_>>();
    let master_names = supporters
        .names
        .iter()
        .filter(|s| s.tier == SupporterTier::Master)
        .map(|s| s.name.clone())
        .collect::<Vec<_>>();

    let header_font_size = 32.0;
    let font_size = 24.0;

    let menu_root = commands
        .spawn((
            Name::new("Menu root"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(10.0),
                ..default()
            },
            PlayerCameraNode,
            StateScoped(MenuState::Credits),
        ))
        .id();

    let content_1 = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            ChildOf(menu_root),
        ))
        .id();
    let content_2 = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            ChildOf(menu_root),
        ))
        .id();
    let content_3 = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            ChildOf(menu_root),
        ))
        .id();

    commands.spawn((
        Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        ChildOf(content_2),
        children![(
            Text::new("Credits"),
            TextFont {
                font_size: 64.0,
                ..default()
            },
            TextLayout {
                justify: JustifyText::Center,
                ..default()
            },
        )],
    ));
    commands.spawn((
        Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        ChildOf(content_1),
        children![(
            Text::new("SBEPIS by Dragon & Fox Collective"),
            TextFont {
                font_size: header_font_size,
                ..default()
            },
            TextLayout {
                justify: JustifyText::Center,
                ..default()
            },
        )],
    ));
    commands.spawn((
        Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        ChildOf(content_3),
        children![(
            Text::new("Based on Homestuck by Andrew Hussie"),
            TextFont {
                font_size: header_font_size,
                ..default()
            },
            TextLayout {
                justify: JustifyText::Center,
                ..default()
            },
        )],
    ));

    let mechanics_root = commands
        .spawn((
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ChildOf(content_1),
            children![(
                Text::new("Mechanics"),
                TextFont {
                    font_size: header_font_size,
                    ..default()
                },
                TextLayout {
                    justify: JustifyText::Center,
                    ..default()
                },
                TextColor(Color::from(Srgba::hex("ff2106")?)),
            )],
        ))
        .id();
    for name in mechanics_names {
        commands.spawn((
            Text::new(name),
            TextFont {
                font_size,
                ..default()
            },
            TextLayout {
                justify: JustifyText::Center,
                ..default()
            },
            ChildOf(mechanics_root),
        ));
    }

    let programming_root = commands
        .spawn((
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ChildOf(content_1),
            children![(
                Text::new("Programming"),
                TextFont {
                    font_size: header_font_size,
                    ..default()
                },
                TextLayout {
                    justify: JustifyText::Center,
                    ..default()
                },
                TextColor(Color::from(Srgba::hex("20401f")?)),
            )],
        ))
        .id();
    for name in programming_names {
        commands.spawn((
            Text::new(name),
            TextFont {
                font_size,
                ..default()
            },
            TextLayout {
                justify: JustifyText::Center,
                ..default()
            },
            ChildOf(programming_root),
        ));
    }

    let art_root = commands
        .spawn((
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ChildOf(content_1),
            children![(
                Text::new("Art"),
                TextFont {
                    font_size: header_font_size,
                    ..default()
                },
                TextLayout {
                    justify: JustifyText::Center,
                    ..default()
                },
                TextColor(Color::from(Srgba::hex("2df901")?)),
            )],
        ))
        .id();
    for name in art_names {
        commands.spawn((
            Text::new(name),
            TextFont {
                font_size,
                ..default()
            },
            TextLayout {
                justify: JustifyText::Center,
                ..default()
            },
            ChildOf(art_root),
        ));
    }

    let music_root = commands
        .spawn((
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ChildOf(content_1),
            children![(
                Text::new("Music"),
                TextFont {
                    font_size: header_font_size,
                    ..default()
                },
                TextLayout {
                    justify: JustifyText::Center,
                    ..default()
                },
                TextColor(Color::from(Srgba::hex("bd1864")?)),
            )],
        ))
        .id();
    for name in music_names {
        commands.spawn((
            Text::new(name),
            TextFont {
                font_size,
                ..default()
            },
            TextLayout {
                justify: JustifyText::Center,
                ..default()
            },
            ChildOf(music_root),
        ));
    }

    let documentation_root = commands
        .spawn((
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ChildOf(content_2),
            children![(
                Text::new("Documentation"),
                TextFont {
                    font_size: header_font_size,
                    ..default()
                },
                TextLayout {
                    justify: JustifyText::Center,
                    ..default()
                },
                TextColor(Color::from(Srgba::hex("fff547")?)),
            )],
        ))
        .id();
    for name in documentation_names {
        commands.spawn((
            Text::new(name),
            TextFont {
                font_size,
                ..default()
            },
            TextLayout {
                justify: JustifyText::Center,
                ..default()
            },
            ChildOf(documentation_root),
        ));
    }

    let contributor_root = commands
        .spawn((
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ChildOf(content_2),
            children![(
                Text::new("Additional Contributions"),
                TextFont {
                    font_size: header_font_size,
                    ..default()
                },
                TextLayout {
                    justify: JustifyText::Center,
                    ..default()
                },
                TextColor(Color::from(Srgba::hex("033476")?)),
            )],
        ))
        .id();
    for name in contributor_names {
        commands.spawn((
            Text::new(name),
            TextFont {
                font_size,
                ..default()
            },
            TextLayout {
                justify: JustifyText::Center,
                ..default()
            },
            ChildOf(contributor_root),
        ));
    }

    let supporters_pane = commands
        .spawn((
            Node {
                height: Val::Percent(100.0),
                overflow: Overflow::scroll(),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            },
            ChildOf(content_3),
        ))
        .id();

    if !master_names.is_empty() {
        let master_root = commands
            .spawn((
                Node {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ChildOf(supporters_pane),
                children![(
                    Text::new("Master Tier Supporters"),
                    TextFont {
                        font_size: 32.0,
                        ..default()
                    },
                    TextLayout {
                        justify: JustifyText::Center,
                        ..default()
                    },
                    TextColor(Color::from(Srgba::hex("ff0000")?)),
                )],
            ))
            .id();
        for (i, name) in master_names.into_iter().enumerate() {
            commands.spawn((
                Text::new(name),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextLayout {
                    justify: JustifyText::Center,
                    ..default()
                },
                ChildOf(master_root),
                TextColor(Color::from(Srgba::hex(if i % 2 == 0 {
                    "00ff00"
                } else {
                    "ff0000"
                })?)),
            ));
        }
    }

    if !denizen_names.is_empty() {
        let denizen_root = commands
            .spawn((
                Node {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ChildOf(supporters_pane),
                children![(
                    Text::new("Denizen Tier Supporters"),
                    TextFont {
                        font_size: 32.0,
                        ..default()
                    },
                    TextLayout {
                        justify: JustifyText::Center,
                        ..default()
                    },
                    TextColor(Color::from(Srgba::hex("efbf04")?)),
                )],
            ))
            .id();
        for name in denizen_names {
            commands.spawn((
                Text::new(name),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextLayout {
                    justify: JustifyText::Center,
                    ..default()
                },
                ChildOf(denizen_root),
            ));
        }
    }

    if !alchemiter_names.is_empty() {
        let alchemiter_root = commands
            .spawn((
                Node {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ChildOf(supporters_pane),
                children![(
                    Text::new("Alchemiter Tier Supporters"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextLayout {
                        justify: JustifyText::Center,
                        ..default()
                    },
                    TextColor(Color::from(Srgba::hex("03a9f4")?)),
                )],
            ))
            .id();
        for name in alchemiter_names {
            commands.spawn((
                Text::new(name),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextLayout {
                    justify: JustifyText::Center,
                    ..default()
                },
                ChildOf(alchemiter_root),
            ));
        }
    }

    if !captcha_names.is_empty() {
        let captcha_root = commands
            .spawn((
                Node {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ChildOf(supporters_pane),
                children![(
                    Text::new("Captcha Tier Supporters"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextLayout {
                        justify: JustifyText::Center,
                        ..default()
                    },
                    TextColor(Color::from(Srgba::hex("ff067c")?)),
                )],
            ))
            .id();
        for name in captcha_names {
            commands.spawn((
                Text::new(name),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextLayout {
                    justify: JustifyText::Center,
                    ..default()
                },
                ChildOf(captcha_root),
            ));
        }
    }

    if !pgo_names.is_empty() {
        let pgo_root = commands
            .spawn((
                Node {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ChildOf(supporters_pane),
                children![(
                    Text::new("PGO Tier Supporters"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextLayout {
                        justify: JustifyText::Center,
                        ..default()
                    },
                    TextColor(Color::from(Srgba::hex("4bec13")?)),
                )],
            ))
            .id();
        for name in pgo_names {
            commands.spawn((
                Text::new(name),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextLayout {
                    justify: JustifyText::Center,
                    ..default()
                },
                ChildOf(pgo_root),
            ));
        }
    }

    if !past_names.is_empty() {
        let past_root = commands
            .spawn((
                Node {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ChildOf(supporters_pane),
                children![(
                    Text::new("Past Supporters"),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextLayout {
                        justify: JustifyText::Center,
                        ..default()
                    },
                    TextColor(Color::from(Srgba::hex("aaaaaa")?)),
                )],
            ))
            .id();
        for name in past_names {
            commands.spawn((
                Text::new(name),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextLayout {
                    justify: JustifyText::Center,
                    ..default()
                },
                ChildOf(past_root),
            ));
        }
    }

    commands.spawn((
        Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        ChildOf(content_2),
        children![
            (
                Text::new("Wizard"),
                TextFont {
                    font_size: header_font_size,
                    ..default()
                },
                TextLayout {
                    justify: JustifyText::Center,
                    ..default()
                },
                TextColor(Color::from(Srgba::hex("46fbc4")?)),
            ),
            (
                Text::new("Kagrul"),
                TextFont {
                    font_size,
                    ..default()
                },
                TextLayout {
                    justify: JustifyText::Center,
                    ..default()
                },
            ),
        ],
    ));

    commands.spawn((
        Node {
            flex_grow: 1.0,
            ..default()
        },
        ChildOf(content_3),
    ));
    commands.spawn((
        Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        ChildOf(content_3),
        children![(
            Text::new("If your name is missing from any of these lists, please let us know!"),
            TextFont {
                font_size,
                ..default()
            },
            TextLayout {
                justify: JustifyText::Center,
                ..default()
            },
        ),],
    ));

    commands.spawn((
        Node {
            flex_grow: 1.0,
            ..default()
        },
        ChildOf(content_2),
    ));
    commands
        .spawn((
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ChildOf(content_2),
            Button,
            children![(
                Text::new("Back"),
                TextFont {
                    font_size,
                    ..default()
                },
                TextLayout {
                    justify: JustifyText::Center,
                    ..default()
                },
            )],
        ))
        .observe(
            |_: Trigger<Pointer<Click>>, mut state: ResMut<NextState<MenuState>>| {
                state.set(MenuState::Home);
            },
        );

    Ok(())
}

#[add_system(plugin = MainMenuPlugin, schedule = Update)]
fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Has<UselessButton>),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, is_useless) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = if is_useless {
                    Color::srgb(0.75, 0.35, 0.35).into()
                } else {
                    Color::srgb(0.35, 0.75, 0.35).into()
                };
            }
            Interaction::Hovered => {
                *color = Color::srgb(0.25, 0.25, 0.25).into();
            }
            Interaction::None => {
                *color = Color::srgb(0.15, 0.15, 0.15).into();
            }
        }
    }
}

#[derive(Resource)]
pub struct DenySound(pub Handle<AudioSource>);

#[derive(Component)]
struct UselessButton;

#[derive(Asset, Clone, Deserialize, TypePath)]
pub struct Supporters {
    pub names: Vec<Supporter>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Supporter {
    pub name: String,
    pub tier: SupporterTier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum SupporterTier {
    Past,
    Pgo,
    Captcha,
    Alchemiter,
    Denizen,
    Master,
}

#[derive(Resource)]
struct MainMenuNames {
    pub supporters: Handle<Supporters>,
    pub developers: Handle<Developers>,
}

#[add_system(plugin = MainMenuPlugin, schedule = Startup)]
fn load_names(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(MainMenuNames {
        supporters: asset_server.load("supporters.supporters.ron"),
        developers: asset_server.load("developers.developers.ron"),
    });
}

#[add_plugin(to_plugin = MainMenuPlugin, generics = <Supporters>, init = RonAssetPlugin::<Supporters>::new(&["supporters.ron"]))]
#[add_plugin(to_plugin = MainMenuPlugin, generics = <Developers>, init = RonAssetPlugin::<Developers>::new(&["developers.ron"]))]
use bevy_common_assets::ron::RonAssetPlugin;

#[derive(Asset, Clone, Deserialize, TypePath)]
pub struct Developers {
    pub names: Vec<Developer>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Developer {
    pub name: String,
    pub area: DeveloperArea,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum DeveloperArea {
    Mechanics,
    Programming,
    Art,
    Music,
    Documentation,
    Contributor,
}

#[derive(Resource)]
pub struct TitleFont(pub Handle<Font>);
