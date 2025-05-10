use bevy::prelude::*;
use bevy_butler::*;
use leafwing_input_manager::prelude::*;
use soundyrust::Note;

use crate::input::{MapsToEvent, button_just_pressed};
use crate::player_commands::{CloseStaffAction, CommandSent, CommandSentSet, PlayerCommandsPlugin};

#[derive(Event)]
#[add_event(plugin = PlayerCommandsPlugin)]
pub struct NotePlayed {
    pub note: Note,
}
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct NotePlayedSet;

// #[add_system(
// 	plugin = PlayerCommandsPlugin, schedule = Update,
// 	generics = <PlayNoteAction, NotePlayed>,
// 	in_set = NotePlayedSet,
// )]
use crate::input::map_action_to_event;

#[derive(Event)]
#[add_event(plugin = PlayerCommandsPlugin)]
pub struct NotesCleared;
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct NotesClearedSet;

#[derive(Actionlike, Clone, Copy, Eq, PartialEq, Hash, Reflect, Debug)]
pub enum PlayNoteAction {
    C0,
    CS0,
    D0,
    DS0,
    E0,
    F0,
    FS0,
    G0,
    GS0,
    A0,
    AS0,
    B0,
    C1,
    CS1,
    D1,
    DS1,
    E1,
    F1,
    FS1,
    G1,
    GS1,
    A1,
    AS1,
    B1,
    C2,
    CS2,
    D2,
    DS2,
    E2,
    F2,
    FS2,
    G2,
    GS2,
    A2,
    AS2,
    B2,
    C3,
    CS3,
    D3,
    DS3,
    E3,
    F3,
    FS3,
    G3,
    GS3,
    A3,
    AS3,
    B3,
    C4,
    CS4,
    D4,
    DS4,
    E4,
    F4,
    FS4,
    G4,
    GS4,
    A4,
    AS4,
    B4,
    C5,
    CS5,
    D5,
    DS5,
    E5,
    F5,
    FS5,
    G5,
    GS5,
    A5,
    AS5,
    B5,
    C6,
    CS6,
    D6,
    DS6,
    E6,
    F6,
    FS6,
    G6,
    GS6,
    A6,
    AS6,
    B6,
    C7,
    CS7,
    D7,
    DS7,
    E7,
    F7,
    FS7,
    G7,
    GS7,
    A7,
    AS7,
    B7,
    C8,
    CS8,
    D8,
    DS8,
    E8,
    F8,
    FS8,
    G8,
    GS8,
    A8,
    AS8,
    B8,
}

impl PlayNoteAction {
    pub fn note(&self) -> Note {
        match self {
            PlayNoteAction::C0 => Note::C0,
            PlayNoteAction::CS0 => Note::CS0,
            PlayNoteAction::D0 => Note::D0,
            PlayNoteAction::DS0 => Note::DS0,
            PlayNoteAction::E0 => Note::E0,
            PlayNoteAction::F0 => Note::F0,
            PlayNoteAction::FS0 => Note::FS0,
            PlayNoteAction::G0 => Note::G0,
            PlayNoteAction::GS0 => Note::GS0,
            PlayNoteAction::A0 => Note::A0,
            PlayNoteAction::AS0 => Note::AS0,
            PlayNoteAction::B0 => Note::B0,
            PlayNoteAction::C1 => Note::C1,
            PlayNoteAction::CS1 => Note::CS1,
            PlayNoteAction::D1 => Note::D1,
            PlayNoteAction::DS1 => Note::DS1,
            PlayNoteAction::E1 => Note::E1,
            PlayNoteAction::F1 => Note::F1,
            PlayNoteAction::FS1 => Note::FS1,
            PlayNoteAction::G1 => Note::G1,
            PlayNoteAction::GS1 => Note::GS1,
            PlayNoteAction::A1 => Note::A1,
            PlayNoteAction::AS1 => Note::AS1,
            PlayNoteAction::B1 => Note::B1,
            PlayNoteAction::C2 => Note::C2,
            PlayNoteAction::CS2 => Note::CS2,
            PlayNoteAction::D2 => Note::D2,
            PlayNoteAction::DS2 => Note::DS2,
            PlayNoteAction::E2 => Note::E2,
            PlayNoteAction::F2 => Note::F2,
            PlayNoteAction::FS2 => Note::FS2,
            PlayNoteAction::G2 => Note::G2,
            PlayNoteAction::GS2 => Note::GS2,
            PlayNoteAction::A2 => Note::A2,
            PlayNoteAction::AS2 => Note::AS2,
            PlayNoteAction::B2 => Note::B2,
            PlayNoteAction::C3 => Note::C3,
            PlayNoteAction::CS3 => Note::CS3,
            PlayNoteAction::D3 => Note::D3,
            PlayNoteAction::DS3 => Note::DS3,
            PlayNoteAction::E3 => Note::E3,
            PlayNoteAction::F3 => Note::F3,
            PlayNoteAction::FS3 => Note::FS3,
            PlayNoteAction::G3 => Note::G3,
            PlayNoteAction::GS3 => Note::GS3,
            PlayNoteAction::A3 => Note::A3,
            PlayNoteAction::AS3 => Note::AS3,
            PlayNoteAction::B3 => Note::B3,
            PlayNoteAction::C4 => Note::C4,
            PlayNoteAction::CS4 => Note::CS4,
            PlayNoteAction::D4 => Note::D4,
            PlayNoteAction::DS4 => Note::DS4,
            PlayNoteAction::E4 => Note::E4,
            PlayNoteAction::F4 => Note::F4,
            PlayNoteAction::FS4 => Note::FS4,
            PlayNoteAction::G4 => Note::G4,
            PlayNoteAction::GS4 => Note::GS4,
            PlayNoteAction::A4 => Note::A4,
            PlayNoteAction::AS4 => Note::AS4,
            PlayNoteAction::B4 => Note::B4,
            PlayNoteAction::C5 => Note::C5,
            PlayNoteAction::CS5 => Note::CS5,
            PlayNoteAction::D5 => Note::D5,
            PlayNoteAction::DS5 => Note::DS5,
            PlayNoteAction::E5 => Note::E5,
            PlayNoteAction::F5 => Note::F5,
            PlayNoteAction::FS5 => Note::FS5,
            PlayNoteAction::G5 => Note::G5,
            PlayNoteAction::GS5 => Note::GS5,
            PlayNoteAction::A5 => Note::A5,
            PlayNoteAction::AS5 => Note::AS5,
            PlayNoteAction::B5 => Note::B5,
            PlayNoteAction::C6 => Note::C6,
            PlayNoteAction::CS6 => Note::CS6,
            PlayNoteAction::D6 => Note::D6,
            PlayNoteAction::DS6 => Note::DS6,
            PlayNoteAction::E6 => Note::E6,
            PlayNoteAction::F6 => Note::F6,
            PlayNoteAction::FS6 => Note::FS6,
            PlayNoteAction::G6 => Note::G6,
            PlayNoteAction::GS6 => Note::GS6,
            PlayNoteAction::A6 => Note::A6,
            PlayNoteAction::AS6 => Note::AS6,
            PlayNoteAction::B6 => Note::B6,
            PlayNoteAction::C7 => Note::C7,
            PlayNoteAction::CS7 => Note::CS7,
            PlayNoteAction::D7 => Note::D7,
            PlayNoteAction::DS7 => Note::DS7,
            PlayNoteAction::E7 => Note::E7,
            PlayNoteAction::F7 => Note::F7,
            PlayNoteAction::FS7 => Note::FS7,
            PlayNoteAction::G7 => Note::G7,
            PlayNoteAction::GS7 => Note::GS7,
            PlayNoteAction::A7 => Note::A7,
            PlayNoteAction::AS7 => Note::AS7,
            PlayNoteAction::B7 => Note::B7,
            PlayNoteAction::C8 => Note::C8,
            PlayNoteAction::CS8 => Note::CS8,
            PlayNoteAction::D8 => Note::D8,
            PlayNoteAction::DS8 => Note::DS8,
            PlayNoteAction::E8 => Note::E8,
            PlayNoteAction::F8 => Note::F8,
            PlayNoteAction::FS8 => Note::FS8,
            PlayNoteAction::G8 => Note::G8,
            PlayNoteAction::GS8 => Note::GS8,
            PlayNoteAction::A8 => Note::A8,
            PlayNoteAction::AS8 => Note::AS8,
            PlayNoteAction::B8 => Note::B8,
        }
    }
}
impl MapsToEvent<NotePlayed> for PlayNoteAction {
    fn make_event(&self) -> NotePlayed {
        NotePlayed { note: self.note() }
    }
}

#[add_system(
	plugin = PlayerCommandsPlugin, schedule = Update,
	after = NotePlayedSet,
)]
fn spawn_note_audio(
    mut commands: Commands,
    mut ev_note_played: EventReader<NotePlayed>,
    asset_server: Res<AssetServer>,
) {
    for ev in ev_note_played.read() {
        let note = ev.note;

        commands.spawn((
            AudioPlayer::new(asset_server.load("flute.wav")),
            PlaybackSettings::DESPAWN.with_speed(note.frequency / Note::C4.frequency),
        ));
    }
}

#[add_system(
	plugin = PlayerCommandsPlugin, schedule = Update,
	after = CommandSentSet,
	in_set = NotesClearedSet,
	run_if = on_event::<CommandSent>,
)]
#[add_system(
	plugin = PlayerCommandsPlugin, schedule = Update,
	in_set = NotesClearedSet,
	run_if = button_just_pressed(CloseStaffAction),
)]
fn clear_notes(mut ev_clear_notes: EventWriter<NotesCleared>) {
    ev_clear_notes.write(NotesCleared);
}
