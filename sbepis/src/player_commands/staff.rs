use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy_butler::*;
use leafwing_input_manager::Actionlike;
use leafwing_input_manager::prelude::InputMap;

use crate::camera::PlayerCameraNode;
use crate::input::input_manager_bundle;
use crate::menus::{
    CloseMenuBinding, Menu, MenuHidesWhenClosed, MenuManipulationSet, MenuWithInputManager,
    MenuWithoutMouse, OpenMenuBinding,
};
use crate::player_commands::PlayerCommandsPlugin;
use crate::player_commands::note_holder::NoteNodeHolder;
use crate::player_commands::notes::PlayNoteAction;
use crate::player_controller::PlayerAction;

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
            input_manager_bundle(
                InputMap::default()
                    .with(PlayNoteAction::C4, KeyCode::KeyZ)
                    .with(PlayNoteAction::CS4, KeyCode::KeyS)
                    .with(PlayNoteAction::D4, KeyCode::KeyX)
                    .with(PlayNoteAction::DS4, KeyCode::KeyD)
                    .with(PlayNoteAction::E4, KeyCode::KeyC)
                    .with(PlayNoteAction::F4, KeyCode::KeyV)
                    .with(PlayNoteAction::FS4, KeyCode::KeyG)
                    .with(PlayNoteAction::G4, KeyCode::KeyB)
                    .with(PlayNoteAction::GS4, KeyCode::KeyH)
                    .with(PlayNoteAction::A4, KeyCode::KeyN)
                    .with(PlayNoteAction::AS4, KeyCode::KeyJ)
                    .with(PlayNoteAction::B4, KeyCode::KeyM)
                    .with(PlayNoteAction::C5, KeyCode::Comma)
                    .with(PlayNoteAction::CS5, KeyCode::KeyL)
                    .with(PlayNoteAction::D5, KeyCode::Period)
                    .with(PlayNoteAction::DS5, KeyCode::Semicolon)
                    .with(PlayNoteAction::E5, KeyCode::Slash)
                    .with(PlayNoteAction::C5, KeyCode::KeyQ)
                    .with(PlayNoteAction::CS5, KeyCode::Digit2)
                    .with(PlayNoteAction::D5, KeyCode::KeyW)
                    .with(PlayNoteAction::DS5, KeyCode::Digit3)
                    .with(PlayNoteAction::E5, KeyCode::KeyE)
                    .with(PlayNoteAction::F5, KeyCode::KeyR)
                    .with(PlayNoteAction::FS5, KeyCode::Digit5)
                    .with(PlayNoteAction::G5, KeyCode::KeyT)
                    .with(PlayNoteAction::GS5, KeyCode::Digit6)
                    .with(PlayNoteAction::A5, KeyCode::KeyY)
                    .with(PlayNoteAction::AS5, KeyCode::Digit7)
                    .with(PlayNoteAction::B5, KeyCode::KeyU)
                    .with(PlayNoteAction::C6, KeyCode::KeyI)
                    .with(PlayNoteAction::CS6, KeyCode::Digit9)
                    .with(PlayNoteAction::D6, KeyCode::KeyO)
                    .with(PlayNoteAction::DS6, KeyCode::Digit0)
                    .with(PlayNoteAction::E6, KeyCode::KeyP),
                false,
            ),
            input_manager_bundle(
                InputMap::default().with(CloseStaffAction, KeyCode::Backquote),
                false,
            ),
            Menu,
            MenuWithInputManager,
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

pub struct OpenStaffBinding;
impl OpenMenuBinding for OpenStaffBinding {
    type Action = PlayerAction;
    type Menu = CommandStaff;
    fn action() -> Self::Action {
        PlayerAction::OpenStaff
    }
}

#[add_system(
	plugin = PlayerCommandsPlugin, schedule = Update,
	generics = <OpenStaffBinding>,
	in_set = MenuManipulationSet,
)]
use crate::menus::show_menu_on_action;

#[derive(Actionlike, Clone, Copy, Eq, PartialEq, Hash, Reflect, Debug)]
pub struct CloseStaffAction;

pub struct CloseStaffBinding;
impl CloseMenuBinding for CloseStaffBinding {
    type Action = CloseStaffAction;
    fn action() -> Self::Action {
        CloseStaffAction
    }
}

#[add_system(
	plugin = PlayerCommandsPlugin, schedule = Update,
	generics = <CloseStaffBinding>,
	in_set = MenuManipulationSet,
)]
use crate::menus::close_menu_on_action;
