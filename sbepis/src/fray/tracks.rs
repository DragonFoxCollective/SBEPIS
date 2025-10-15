use bevy::prelude::*;
use bevy_butler::*;
use bevy_pretty_nice_menus::MenuStack;
use soundyrust::{MidiAudio, MidiAudioTrackHandle};

use crate::dialogue::spawn_dialogue;
use crate::fray::FrayPlugin;
use crate::prelude::*;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct TrackSwitcher;

#[derive(Resource)]
pub struct FrayTracks {
    pub midi: Handle<MidiAudio>,
    pub player: Track,
    pub imp: Track,
    pub backing_track: MidiAudioTrackHandle,
    pub four_four: MidiAudioTrackHandle,
    pub six_eight: MidiAudioTrackHandle,
}
impl FrayTracks {
    pub fn player_track(&self) -> MidiAudioTrackHandle {
        self.track(self.player)
    }

    pub fn imp_track(&self) -> MidiAudioTrackHandle {
        self.track(self.imp)
    }

    fn track(&self, track: Track) -> MidiAudioTrackHandle {
        match track {
            Track::FourFour => self.four_four,
            Track::SixEight => self.six_eight,
        }
    }

    pub fn set_player_track(&mut self, track: Track) {
        self.player = track;
        self.imp = match track {
            Track::FourFour => Track::SixEight,
            Track::SixEight => Track::FourFour,
        };
    }
}

#[add_observer(plugin = FrayPlugin, generics = <TrackSwitcher>)]
use crate::player_controller::camera_controls::interact_with;

#[add_observer(plugin = FrayPlugin)]
fn open_track_switch_dialogue(
    _: On<InteractWith<TrackSwitcher>>,
    mut commands: Commands,
    mut menu_stack: ResMut<MenuStack>,
) {
    let mut dialogue = spawn_dialogue(
        &mut commands,
        &mut menu_stack,
        "Select a track for the player to use.\nThe imps will use the other one.".to_owned(),
        (),
    );
    dialogue.add_option(
        &mut commands,
        "4/4".to_owned(),
        (),
        SwitchTrack {
            track: Track::FourFour,
            dialogue: dialogue.root,
        },
    );
    dialogue.add_option(
        &mut commands,
        "6/8".to_owned(),
        (),
        SwitchTrack {
            track: Track::SixEight,
            dialogue: dialogue.root,
        },
    );
}

#[add_observer(plugin = FrayPlugin)]
fn switch_track(switch_track: On<SwitchTrack>, mut fray_tracks: ResMut<FrayTracks>) {
    fray_tracks.set_player_track(switch_track.track);
}

#[derive(Clone, Copy)]
pub enum Track {
    FourFour,
    SixEight,
}

#[add_observer(plugin = FrayPlugin, generics = <SwitchTrack>)]
use bevy_pretty_nice_menus::close_menu_on_event;

#[derive(EntityEvent, Clone)]
pub struct SwitchTrack {
    pub track: Track,
    #[event_target]
    pub dialogue: Entity,
}
