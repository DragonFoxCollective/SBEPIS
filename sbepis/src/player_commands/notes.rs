use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_pretty_nice_input::prelude::*;
use soundyrust::Note;

use crate::player_commands::PlayerCommandsPlugin;
use crate::player_commands::commands::SendCommand;
use crate::player_commands::staff::CloseStaff;

#[auto_event(plugin = PlayerCommandsPlugin, target(global), derive, reflect, register)]
pub struct PlayNote {
    pub note: Note,
}

#[auto_event(plugin = PlayerCommandsPlugin, target(global), derive, reflect, register)]
pub struct ClearNotes;

macro_rules! note_action {
    ($note:ident, $id:ident) => {
        paste::paste! {
            #[derive(Action)]
            pub struct [<Play $note $id>];

            #[allow(non_snake_case)]
            #[auto_observer(plugin = PlayerCommandsPlugin)]
            fn [<map_to_enum_ $note _ $id>](_: On<JustPressed<[<Play $note $id>]>>, mut commands: Commands) {
                commands.trigger(PlayNote { note: Note::$note });
            }
        }
    };
}

note_action!(C4, Down);
note_action!(CS4, Down);
note_action!(D4, Down);
note_action!(DS4, Down);
note_action!(E4, Down);
note_action!(F4, Down);
note_action!(FS4, Down);
note_action!(G4, Down);
note_action!(GS4, Down);
note_action!(A4, Down);
note_action!(AS4, Down);
note_action!(B4, Down);
note_action!(C5, Down);
note_action!(CS5, Down);
note_action!(D5, Down);
note_action!(DS5, Down);
note_action!(E5, Down);

note_action!(C5, Up);
note_action!(CS5, Up);
note_action!(D5, Up);
note_action!(DS5, Up);
note_action!(E5, Up);
note_action!(F5, Up);
note_action!(FS5, Up);
note_action!(G5, Up);
note_action!(GS5, Up);
note_action!(A5, Up);
note_action!(AS5, Up);
note_action!(B5, Up);
note_action!(C6, Up);
note_action!(CS6, Up);
note_action!(D6, Up);
note_action!(DS6, Up);
note_action!(E6, Up);

#[auto_observer(plugin = PlayerCommandsPlugin)]
fn spawn_note_audio(
    play_note: On<PlayNote>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        AudioPlayer::new(asset_server.load("flute.wav")),
        PlaybackSettings::DESPAWN.with_speed(play_note.note.frequency / Note::C4.frequency),
    ));
}

#[auto_observer(plugin = PlayerCommandsPlugin)]
fn clear_notes_on_close(_: On<JustPressed<CloseStaff>>, mut commands: Commands) {
    commands.trigger(ClearNotes);
}

#[auto_observer(plugin = PlayerCommandsPlugin)]
fn clear_notes_after_command(_: On<SendCommand>, mut commands: Commands) {
    commands.trigger(ClearNotes);
}
