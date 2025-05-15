use bevy::prelude::*;
use bevy_butler::*;
use return_ok::some_or_return;
use soundyrust::Note;

use crate::player_commands::{NotePlayedSet, NotesCleared, NotesClearedSet, PlayerCommandsPlugin};

use crate::player_commands::notes::NotePlayed;

#[add_system(
	plugin = PlayerCommandsPlugin, schedule = Update,
	generics = <PingCommandEvent>,
	in_set = CommandSentSet,
	run_if = on_event::<NotePlayed>,
)]
#[add_system(
	plugin = PlayerCommandsPlugin, schedule = Update,
	generics = <KillCommandEvent>,
	in_set = CommandSentSet,
	run_if = on_event::<NotePlayed>,
)]
fn check_note_patterns<T: Event + NotePatternEvent>(
    note_holder: Res<NotePatternPlayer>,
    mut ev_command: EventWriter<T>,
    mut ev_command_sent: EventWriter<CommandSent>,
) {
    let event = T::compare_notes(note_holder.current_pattern.as_slice());
    let event = some_or_return!(event);
    ev_command.write(event);
    ev_command_sent.write(CommandSent);
}

#[derive(Resource, Default)]
#[insert_resource(plugin = PlayerCommandsPlugin)]
pub struct NotePatternPlayer {
    pub current_pattern: Vec<Note>,
}

#[derive(Event)]
#[add_event(plugin = PlayerCommandsPlugin)]
pub struct CommandSent;
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommandSentSet;

#[add_system(
	plugin = PlayerCommandsPlugin, schedule = Update,
	after = NotePlayedSet,
	before = CommandSentSet,
)]
fn add_note_to_player(
    mut player: ResMut<NotePatternPlayer>,
    mut ev_note_played: EventReader<NotePlayed>,
) {
    for ev in ev_note_played.read() {
        player.current_pattern.push(ev.note);
    }
}

#[add_system(
	plugin = PlayerCommandsPlugin, schedule = Update,
	after = NotesClearedSet,
	run_if = on_event::<NotesCleared>,
)]
fn clear_player_notes(mut player: ResMut<NotePatternPlayer>) {
    player.current_pattern.clear();
}

pub trait NotePatternEvent {
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

#[derive(Event)]
#[add_event(plugin = PlayerCommandsPlugin)]
pub struct PingCommandEvent;

impl PingCommandEvent {
    const PATTERN: &'static [Note] = &[Note::C4, Note::D4, Note::E4];
}

impl NotePatternEvent for PingCommandEvent {
    fn compare_notes(notes: &[Note]) -> Option<Self>
    where
        Self: Sized,
    {
        let _notes = notes.eat(PingCommandEvent::PATTERN)?;
        Some(PingCommandEvent)
    }
}

#[add_system(
	plugin = PlayerCommandsPlugin, schedule = Update,
	after = CommandSentSet,
)]
fn ping(
    mut ev_ping: EventReader<PingCommandEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for _ in ev_ping.read() {
        commands.spawn((
            AudioPlayer::new(asset_server.load("pester_notif.mp3")),
            PlaybackSettings::DESPAWN,
        ));
    }
}

#[derive(Event)]
#[add_event(plugin = PlayerCommandsPlugin)]
pub struct KillCommandEvent(pub bool);

impl KillCommandEvent {
    const PATTERN: &'static [Note] = &[Note::D4, Note::D4, Note::D5];
}

impl NotePatternEvent for KillCommandEvent {
    fn compare_notes(notes: &[Note]) -> Option<Self>
    where
        Self: Sized,
    {
        let notes = notes.eat(KillCommandEvent::PATTERN)?;
        let (actually_kill, _notes) = notes.eat_type()?;
        Some(KillCommandEvent(actually_kill))
    }
}

#[add_system(
	plugin = PlayerCommandsPlugin, schedule = Update,
	after = CommandSentSet,
)]
fn kill(mut ev_kill: EventReader<KillCommandEvent>, mut ev_quit: EventWriter<AppExit>) {
    for ev in ev_kill.read() {
        debug!("Tried to kill {}", ev.0);
        if ev.0 {
            ev_quit.write(AppExit::Success);
        }
    }
}
