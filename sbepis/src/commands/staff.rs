use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_pretty_nice_input::prelude::*;
use bevy_pretty_nice_menus::{MenuDespawnsWhenClosed, MenuStack, MenuWithInput, MenuWithoutMouse};

use crate::commands::PlayerCommandsPlugin;
use crate::commands::note_holder::NoteNodeHolder;
use crate::commands::notes::*;
use crate::player::OpenStaff;
use crate::prelude::*;

#[auto_component(plugin = PlayerCommandsPlugin, derive, reflect, register)]
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

#[auto_observer(plugin = PlayerCommandsPlugin)]
fn spawn_staff(
    _staff: On<JustPressed<OpenStaff>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut menu_stack: ResMut<MenuStack>,
) {
    // Background
    let menu = commands
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
            BackgroundColor(css::BEIGE.into()),
            CommandStaff,
            PlayerCameraNode,
            (
                (
                    input!(PlayC4Down, Axis1D[binding1d::key(KeyCode::KeyZ)]),
                    input!(PlayCS4Down, Axis1D[binding1d::key(KeyCode::KeyS)]),
                    input!(PlayD4Down, Axis1D[binding1d::key(KeyCode::KeyX)]),
                    input!(PlayDS4Down, Axis1D[binding1d::key(KeyCode::KeyD)]),
                    input!(PlayE4Down, Axis1D[binding1d::key(KeyCode::KeyC)]),
                    input!(PlayF4Down, Axis1D[binding1d::key(KeyCode::KeyV)]),
                    input!(PlayFS4Down, Axis1D[binding1d::key(KeyCode::KeyG)]),
                    input!(PlayG4Down, Axis1D[binding1d::key(KeyCode::KeyB)]),
                    input!(PlayGS4Down, Axis1D[binding1d::key(KeyCode::KeyH)]),
                    input!(PlayA4Down, Axis1D[binding1d::key(KeyCode::KeyN)]),
                    input!(PlayAS4Down, Axis1D[binding1d::key(KeyCode::KeyJ)]),
                    input!(PlayB4Down, Axis1D[binding1d::key(KeyCode::KeyM)]),
                ),
                (
                    input!(PlayC5Down, Axis1D[binding1d::key(KeyCode::Comma)]),
                    input!(PlayCS5Down, Axis1D[binding1d::key(KeyCode::KeyL)]),
                    input!(PlayD5Down, Axis1D[binding1d::key(KeyCode::Period)]),
                    input!(PlayDS5Down, Axis1D[binding1d::key(KeyCode::Semicolon)]),
                    input!(PlayE5Down, Axis1D[binding1d::key(KeyCode::Slash)]),
                ),
                (
                    input!(PlayC5Up, Axis1D[binding1d::key(KeyCode::KeyQ)]),
                    input!(PlayCS5Up, Axis1D[binding1d::key(KeyCode::Digit2)]),
                    input!(PlayD5Up, Axis1D[binding1d::key(KeyCode::KeyW)]),
                    input!(PlayDS5Up, Axis1D[binding1d::key(KeyCode::Digit3)]),
                    input!(PlayE5Up, Axis1D[binding1d::key(KeyCode::KeyE)]),
                    input!(PlayF5Up, Axis1D[binding1d::key(KeyCode::KeyR)]),
                    input!(PlayFS5Up, Axis1D[binding1d::key(KeyCode::Digit5)]),
                    input!(PlayG5Up, Axis1D[binding1d::key(KeyCode::KeyT)]),
                    input!(PlayGS5Up, Axis1D[binding1d::key(KeyCode::Digit6)]),
                    input!(PlayA5Up, Axis1D[binding1d::key(KeyCode::KeyY)]),
                    input!(PlayAS5Up, Axis1D[binding1d::key(KeyCode::Digit7)]),
                    input!(PlayB5Up, Axis1D[binding1d::key(KeyCode::KeyU)]),
                ),
                (
                    input!(PlayC6Up, Axis1D[binding1d::key(KeyCode::KeyI)]),
                    input!(PlayCS6Up, Axis1D[binding1d::key(KeyCode::Digit9)]),
                    input!(PlayD6Up, Axis1D[binding1d::key(KeyCode::KeyO)]),
                    input!(PlayDS6Up, Axis1D[binding1d::key(KeyCode::Digit0)]),
                    input!(PlayE6Up, Axis1D[binding1d::key(KeyCode::KeyP)]),
                ),
                input!(CloseStaff, Axis1D[binding1d::key(KeyCode::Backquote)]),
            ),
            MenuWithInput,
            MenuWithoutMouse,
            MenuDespawnsWhenClosed,
        ))
        .with_children(|parent| {
            // Clef
            parent.spawn((
                Name::new("Clef"),
                ImageNode::new(asset_server.load("unlicensed/treble_clef.png")),
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
        })
        .id();
    menu_stack.push(menu);
}

#[derive(Action)]
pub struct CloseStaff;
