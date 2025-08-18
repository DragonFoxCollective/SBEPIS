use bevy::audio::PlaybackMode;
use bevy::prelude::*;
use bevy_butler::*;

use crate::camera::PlayerCamera;
use crate::prelude::*;

#[butler_plugin]
#[add_plugin(to_plugin = SbepisPlugin)]
struct MainMenuPlugin;

#[insert_state(plugin = MainMenuPlugin, init = GameState::MainMenu)]
#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
#[states(scoped_entities)]
pub enum GameState {
    MainMenu,
    InGame,
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

    commands.insert_resource(DenySound(asset_server.load("deny.wav")));

    let font_size = 32.0;

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
            StateScoped(GameState::MainMenu),
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
            flex_grow: 4.0,
            ..default()
        },
        ChildOf(content),
        children![(
            Text::new("SBEPIS"),
            TextFont {
                font_size: 64.0,
                ..default()
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
            UselessButton,
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
