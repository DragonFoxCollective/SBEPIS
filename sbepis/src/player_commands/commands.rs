use bevy::prelude::*;
use bevy_butler::*;
use return_ok::some_or_return;
use soundyrust::Note;

use crate::player_commands::{ClearNotes, NotePlayedSet, NotesClearedSet, PlayerCommandsPlugin};

use crate::player_commands::notes::PlayNote;

#[add_system(
	plugin = PlayerCommandsPlugin, schedule = Update,
	generics = <PingCommandMessage>,
	in_set = CommandSentSet,
	run_if = on_message::<PlayNote>,
)]
#[add_system(
	plugin = PlayerCommandsPlugin, schedule = Update,
	generics = <KillCommandMessage>,
	in_set = CommandSentSet,
	run_if = on_message::<PlayNote>,
)]
fn check_note_patterns<T: Message + NotePatternMessage>(
    note_holder: Res<NotePatternPlayer>,
    mut command: MessageWriter<T>,
    mut send_command: MessageWriter<SendCommand>,
) {
    let event = T::compare_notes(note_holder.current_pattern.as_slice());
    let event = some_or_return!(event);
    command.write(event);
    send_command.write(SendCommand);
}

#[derive(Resource, Default)]
#[insert_resource(plugin = PlayerCommandsPlugin)]
pub struct NotePatternPlayer {
    pub current_pattern: Vec<Note>,
}

#[derive(Message)]
#[add_message(plugin = PlayerCommandsPlugin)]
pub struct SendCommand;
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommandSentSet;

#[add_system(
	plugin = PlayerCommandsPlugin, schedule = Update,
	after = NotePlayedSet,
	before = CommandSentSet,
)]
fn add_note_to_player(
    mut player: ResMut<NotePatternPlayer>,
    mut play_note: MessageReader<PlayNote>,
) {
    for ev in play_note.read() {
        player.current_pattern.push(ev.note);
    }
}

#[add_system(
	plugin = PlayerCommandsPlugin, schedule = Update,
	after = NotesClearedSet,
	run_if = on_message::<ClearNotes>,
)]
fn clear_player_notes(mut player: ResMut<NotePatternPlayer>) {
    player.current_pattern.clear();
}

pub trait NotePatternMessage {
    fn compare_notes(notes: &[Note]) -> Option<Self>
    where
        Self: Sized;
}

pub trait NoteSequence {
    fn eat(self, notes: &[Note]) -> Option<Self>
    where
        Self: Sized;
}

impl NoteSequence for &[Note] {
    fn eat(self, notes: &[Note]) -> Option<Self> {
        if self.starts_with(notes) {
            Some(&self[notes.len()..])
        } else {
            None
        }
    }
}

pub trait NoteSequenceTyped<T> {
    fn eat_type(self) -> Option<(T, Self)>
    where
        Self: Sized;
}

impl NoteSequenceTyped<bool> for &[Note] {
    fn eat_type(self) -> Option<(bool, Self)>
    where
        Self: Sized,
    {
        if self.starts_with(&[Note::A4]) {
            Some((true, &self[1..]))
        } else if self.starts_with(&[Note::C5]) {
            Some((false, &self[1..]))
        } else {
            None
        }
    }
}

#[derive(Message)]
#[add_message(plugin = PlayerCommandsPlugin)]
pub struct PingCommandMessage;

impl PingCommandMessage {
    const PATTERN: &'static [Note] = &[Note::C4, Note::D4, Note::E4];
}

impl NotePatternMessage for PingCommandMessage {
    fn compare_notes(notes: &[Note]) -> Option<Self>
    where
        Self: Sized,
    {
        let _notes = notes.eat(PingCommandMessage::PATTERN)?;
        Some(PingCommandMessage)
    }
}

#[add_system(
	plugin = PlayerCommandsPlugin, schedule = Update,
	after = CommandSentSet,
)]
fn ping(
    mut ping: MessageReader<PingCommandMessage>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for _ in ping.read() {
        commands.spawn((
            AudioPlayer::new(asset_server.load("pester_notif.mp3")),
            PlaybackSettings::DESPAWN,
        ));
    }
}

#[derive(Message)]
#[add_message(plugin = PlayerCommandsPlugin)]
pub struct KillCommandMessage(pub bool);

impl KillCommandMessage {
    const PATTERN: &'static [Note] = &[Note::D4, Note::D4, Note::D5];
}

impl NotePatternMessage for KillCommandMessage {
    fn compare_notes(notes: &[Note]) -> Option<Self>
    where
        Self: Sized,
    {
        let notes = notes.eat(KillCommandMessage::PATTERN)?;
        let (actually_kill, _notes) = notes.eat_type()?;
        Some(KillCommandMessage(actually_kill))
    }
}

#[add_system(
	plugin = PlayerCommandsPlugin, schedule = Update,
	after = CommandSentSet,
)]
fn kill(mut kill: MessageReader<KillCommandMessage>, mut exit: MessageWriter<AppExit>) {
    for ev in kill.read() {
        debug!("Tried to kill {}", ev.0);
        if ev.0 {
            exit.write(AppExit::Success);
        }
    }
}
