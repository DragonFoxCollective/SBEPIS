use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use soundyrust::Note;

use crate::commands::PlayerCommandsPlugin;
use crate::commands::notes::{ClearNotes, PlayNote};
use crate::commands::staff::{
    F5_LINE_TOP, QUARTER_NOTE_HEIGHT, QUARTER_NOTE_LEFT_SPACING, QUARTER_NOTE_LEFT_START,
    QUARTER_NOTE_TOP_OFFSET, QUARTER_NOTE_WEIRD_SPACING_OFFSET, STAFF_HEIGHT,
};
use crate::util::MapRangeBetween;

#[auto_component(plugin = PlayerCommandsPlugin, derive(Default), reflect, register)]
pub struct NoteNodeHolder {
    note_entities: Vec<Entity>,
}

impl NoteNodeHolder {
    pub fn next_note_left(&mut self) -> f32 {
        QUARTER_NOTE_LEFT_START
            + (self.note_entities.len() as f32 + 1.0) * QUARTER_NOTE_LEFT_SPACING
    }

    pub fn note_top(&self, note: &Note) -> f32 {
        (note.position() as f32).map_range_between(
            (Note::E4.position() as f32)..(Note::F5.position() as f32),
            (F5_LINE_TOP + STAFF_HEIGHT - QUARTER_NOTE_WEIRD_SPACING_OFFSET)..F5_LINE_TOP,
        ) - QUARTER_NOTE_TOP_OFFSET
    }
}

#[auto_observer(plugin = PlayerCommandsPlugin)]
fn add_note_to_holder(
    play_note: On<PlayNote>,
    mut commands: Commands,
    mut note_holder: Query<(&mut NoteNodeHolder, Entity)>,
    asset_server: Res<AssetServer>,
) -> Result {
    let (mut note_holder, note_holder_entity) = note_holder.single_mut()?;

    let note = play_note.note;

    debug!(
        "{} {} {}",
        note,
        note.position(),
        note_holder.note_top(&note)
    );

    let note_entity = commands
        .spawn((
            ImageNode::new(asset_server.load("quarter_note.png")),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(note_holder.next_note_left()),
                top: Val::Px(note_holder.note_top(&note)),
                height: Val::Px(QUARTER_NOTE_HEIGHT),
                ..default()
            },
            ChildOf(note_holder_entity),
        ))
        .id();

    note_holder.note_entities.push(note_entity);

    Ok(())
}

#[auto_observer(plugin = PlayerCommandsPlugin)]
fn clear_holder_notes(
    _: On<ClearNotes>,
    mut commands: Commands,
    mut note_holder: Query<&mut NoteNodeHolder>,
) -> Result {
    let mut note_holder = note_holder.single_mut()?;
    for note_entity in note_holder.note_entities.iter_mut() {
        commands.entity(*note_entity).despawn();
    }
    note_holder.note_entities.clear();
    Ok(())
}
