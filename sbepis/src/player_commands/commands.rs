use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use soundyrust::Note;

use crate::player_commands::PlayerCommandsPlugin;
use crate::player_commands::notes::{ClearNotes, PlayNote};

#[auto_resource(plugin = PlayerCommandsPlugin, derive(Default), reflect, register, init)]
pub struct NotePatternPlayer {
    pub current_pattern: Vec<Note>,
}

#[auto_event(plugin = PlayerCommandsPlugin, target(global), derive, reflect, register)]
pub struct ChangeNotePattern {
    pub notes: Vec<Note>,
}

#[auto_event(plugin = PlayerCommandsPlugin, target(global), derive, reflect, register)]
pub struct SendCommand;

#[auto_observer(plugin = PlayerCommandsPlugin)]
fn add_note_to_player_and_check(
    play_note: On<PlayNote>,
    mut player: ResMut<NotePatternPlayer>,
    mut commands: Commands,
) {
    player.current_pattern.push(play_note.note);
    commands.trigger(ChangeNotePattern {
        notes: player.current_pattern.clone(),
    });
}

#[auto_observer(plugin = PlayerCommandsPlugin)]
fn clear_player_notes(_: On<ClearNotes>, mut player: ResMut<NotePatternPlayer>) {
    player.current_pattern.clear();
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

#[auto_observer(plugin = PlayerCommandsPlugin)]
fn ping(pattern: On<ChangeNotePattern>, mut commands: Commands, asset_server: Res<AssetServer>) {
    if let Some(()) = (|| {
        let _notes = pattern.notes.eat(&[Note::D4, Note::D4, Note::D5])?;
        Some(())
    })() {
        commands.spawn((
            AudioPlayer::new(asset_server.load("pester_notif.mp3")),
            PlaybackSettings::DESPAWN,
        ));

        commands.trigger(SendCommand);
    }
}

#[auto_observer(plugin = PlayerCommandsPlugin)]
fn kill(pattern: On<ChangeNotePattern>, mut commands: Commands, mut exit: MessageWriter<AppExit>) {
    if let Some(actually_kill) = (|| {
        let notes = pattern.notes.eat(&[Note::D4, Note::D4, Note::D5])?;
        let (actually_kill, _notes) = notes.eat_type()?;
        Some(actually_kill)
    })() {
        debug!("Tried to kill {actually_kill}");
        if actually_kill {
            exit.write(AppExit::Success);
        }

        commands.trigger(SendCommand);
    }
}
