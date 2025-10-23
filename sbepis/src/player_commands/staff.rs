use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy_butler::*;
use bevy_pretty_nice_input::{Action, binding1d, input};
use bevy_pretty_nice_menus::{Menu, MenuHidesWhenClosed, MenuWithInput, MenuWithoutMouse};

use crate::camera::PlayerCameraNode;
use crate::player_commands::PlayerCommandsPlugin;
use crate::player_commands::note_holder::NoteNodeHolder;
use crate::player_commands::notes::*;
use crate::player_controller::OpenStaff;

#[derive(Component)]
pub struct CommandStaff;

// This should be enough information to map all notes
pub const F5_LINE_TOP: f32 = 15.0;
pub const STAFF_HEIGHT: f32 = 60.0;
pub const CLEF_HEIGHT: f32 = 80.0;
pub const LINE_HEIGHT: f32 = 2.0;

pub const QUARTER_NOTE_TOP_OFFSET: f32 = 41.0;
pub const QUARTER_NOTE_HEIGHT: f32 = 55.0;
pub const QUARTER_NOTE_LEFT_START: f32 = 40.0;
pub const QUARTER_NOTE_LEFT_SPACING: f32 = 20.0;

// Does top + height not actually equal bottom???
pub const QUARTER_NOTE_WEIRD_SPACING_OFFSET: f32 = 18.0;

#[add_system(
	plugin = PlayerCommandsPlugin, schedule = Startup,
)]
fn spawn_staff(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Background
    commands
        .spawn((
            Name::new("Staff"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(100.0),
                flex_direction: FlexDirection::Row,
                margin: UiRect::all(Val::Px(10.0)),
                padding: UiRect::axes(Val::Px(100.0), Val::Px(10.0)),
                ..default()
            },
            Visibility::Hidden,
            BackgroundColor(css::BEIGE.into()),
            CommandStaff,
            PlayerCameraNode,
            (
                (
                    input!(PlayC4Down, [binding1d::key(KeyCode::KeyZ)]),
                    input!(PlayCS4Down, [binding1d::key(KeyCode::KeyS)]),
                    input!(PlayD4Down, [binding1d::key(KeyCode::KeyX)]),
                    input!(PlayDS4Down, [binding1d::key(KeyCode::KeyD)]),
                    input!(PlayE4Down, [binding1d::key(KeyCode::KeyC)]),
                    input!(PlayF4Down, [binding1d::key(KeyCode::KeyV)]),
                    input!(PlayFS4Down, [binding1d::key(KeyCode::KeyG)]),
                    input!(PlayG4Down, [binding1d::key(KeyCode::KeyB)]),
                    input!(PlayGS4Down, [binding1d::key(KeyCode::KeyH)]),
                    input!(PlayA4Down, [binding1d::key(KeyCode::KeyN)]),
                    input!(PlayAS4Down, [binding1d::key(KeyCode::KeyJ)]),
                    input!(PlayB4Down, [binding1d::key(KeyCode::KeyM)]),
                ),
                (
                    input!(PlayC5Down, [binding1d::key(KeyCode::Comma)]),
                    input!(PlayCS5Down, [binding1d::key(KeyCode::KeyL)]),
                    input!(PlayD5Down, [binding1d::key(KeyCode::Period)]),
                    input!(PlayDS5Down, [binding1d::key(KeyCode::Semicolon)]),
                    input!(PlayE5Down, [binding1d::key(KeyCode::Slash)]),
                ),
                (
                    input!(PlayC5Up, [binding1d::key(KeyCode::KeyQ)]),
                    input!(PlayCS5Up, [binding1d::key(KeyCode::Digit2)]),
                    input!(PlayD5Up, [binding1d::key(KeyCode::KeyW)]),
                    input!(PlayDS5Up, [binding1d::key(KeyCode::Digit3)]),
                    input!(PlayE5Up, [binding1d::key(KeyCode::KeyE)]),
                    input!(PlayF5Up, [binding1d::key(KeyCode::KeyR)]),
                    input!(PlayFS5Up, [binding1d::key(KeyCode::Digit5)]),
                    input!(PlayG5Up, [binding1d::key(KeyCode::KeyT)]),
                    input!(PlayGS5Up, [binding1d::key(KeyCode::Digit6)]),
                    input!(PlayA5Up, [binding1d::key(KeyCode::KeyY)]),
                    input!(PlayAS5Up, [binding1d::key(KeyCode::Digit7)]),
                    input!(PlayB5Up, [binding1d::key(KeyCode::KeyU)]),
                ),
                (
                    input!(PlayC6Up, [binding1d::key(KeyCode::KeyI)]),
                    input!(PlayCS6Up, [binding1d::key(KeyCode::Digit9)]),
                    input!(PlayD6Up, [binding1d::key(KeyCode::KeyO)]),
                    input!(PlayDS6Up, [binding1d::key(KeyCode::Digit0)]),
                    input!(PlayE6Up, [binding1d::key(KeyCode::KeyP)]),
                ),
                input!(CloseStaff, [binding1d::key(KeyCode::Backquote)]),
            ),
            Menu,
            MenuWithInput,
            MenuWithoutMouse,
            MenuHidesWhenClosed,
        ))
        .with_children(|parent| {
            // Clef
            parent.spawn((
                Name::new("Clef"),
                ImageNode::new(asset_server.load("treble_clef.png")),
                Node {
                    position_type: PositionType::Absolute,
                    height: Val::Px(CLEF_HEIGHT),
                    ..default()
                },
            ));

            // Staff lines
            parent
                .spawn((
                    Name::new("Staff lines"),
                    Node {
                        flex_direction: FlexDirection::Column,
                        flex_grow: 1.0,
                        padding: UiRect::top(Val::Px(F5_LINE_TOP)),
                        height: Val::Px(STAFF_HEIGHT),
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    NoteNodeHolder::default(),
                ))
                .with_children(|parent| {
                    for i in 0..5 {
                        parent.spawn((
                            Name::new(format!("Line {i}")),
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(LINE_HEIGHT),
                                ..default()
                            },
                            BackgroundColor(Color::BLACK),
                        ));
                    }
                });
        });
}

#[add_observer(plugin = PlayerCommandsPlugin, generics = <OpenStaff, CommandStaff>)]
use bevy_pretty_nice_menus::show_menu_on_action;

#[derive(Action)]
pub struct CloseStaff;

#[add_observer(plugin = PlayerCommandsPlugin, generics = <CloseStaff>)]
use bevy_pretty_nice_menus::close_menu_on_action;
