use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy_butler::*;
use bevy_pretty_nice_input::{Action, Binding1D, input};
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
                    input!(PlayC4Down, [Binding1D::Key(KeyCode::KeyZ)]),
                    input!(PlayCS4Down, [Binding1D::Key(KeyCode::KeyS)]),
                    input!(PlayD4Down, [Binding1D::Key(KeyCode::KeyX)]),
                    input!(PlayDS4Down, [Binding1D::Key(KeyCode::KeyD)]),
                    input!(PlayE4Down, [Binding1D::Key(KeyCode::KeyC)]),
                    input!(PlayF4Down, [Binding1D::Key(KeyCode::KeyV)]),
                    input!(PlayFS4Down, [Binding1D::Key(KeyCode::KeyG)]),
                    input!(PlayG4Down, [Binding1D::Key(KeyCode::KeyB)]),
                    input!(PlayGS4Down, [Binding1D::Key(KeyCode::KeyH)]),
                    input!(PlayA4Down, [Binding1D::Key(KeyCode::KeyN)]),
                    input!(PlayAS4Down, [Binding1D::Key(KeyCode::KeyJ)]),
                    input!(PlayB4Down, [Binding1D::Key(KeyCode::KeyM)]),
                ),
                (
                    input!(PlayC5Down, [Binding1D::Key(KeyCode::Comma)]),
                    input!(PlayCS5Down, [Binding1D::Key(KeyCode::KeyL)]),
                    input!(PlayD5Down, [Binding1D::Key(KeyCode::Period)]),
                    input!(PlayDS5Down, [Binding1D::Key(KeyCode::Semicolon)]),
                    input!(PlayE5Down, [Binding1D::Key(KeyCode::Slash)]),
                ),
                (
                    input!(PlayC5Up, [Binding1D::Key(KeyCode::KeyQ)]),
                    input!(PlayCS5Up, [Binding1D::Key(KeyCode::Digit2)]),
                    input!(PlayD5Up, [Binding1D::Key(KeyCode::KeyW)]),
                    input!(PlayDS5Up, [Binding1D::Key(KeyCode::Digit3)]),
                    input!(PlayE5Up, [Binding1D::Key(KeyCode::KeyE)]),
                    input!(PlayF5Up, [Binding1D::Key(KeyCode::KeyR)]),
                    input!(PlayFS5Up, [Binding1D::Key(KeyCode::Digit5)]),
                    input!(PlayG5Up, [Binding1D::Key(KeyCode::KeyT)]),
                    input!(PlayGS5Up, [Binding1D::Key(KeyCode::Digit6)]),
                    input!(PlayA5Up, [Binding1D::Key(KeyCode::KeyY)]),
                    input!(PlayAS5Up, [Binding1D::Key(KeyCode::Digit7)]),
                    input!(PlayB5Up, [Binding1D::Key(KeyCode::KeyU)]),
                ),
                (
                    input!(PlayC6Up, [Binding1D::Key(KeyCode::KeyI)]),
                    input!(PlayCS6Up, [Binding1D::Key(KeyCode::Digit9)]),
                    input!(PlayD6Up, [Binding1D::Key(KeyCode::KeyO)]),
                    input!(PlayDS6Up, [Binding1D::Key(KeyCode::Digit0)]),
                    input!(PlayE6Up, [Binding1D::Key(KeyCode::KeyP)]),
                ),
                input!(CloseStaff, [Binding1D::Key(KeyCode::Backquote)]),
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
