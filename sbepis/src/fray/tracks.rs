use bevy::prelude::*;
use bevy_butler::*;
use leafwing_input_manager::prelude::*;
use soundyrust::{MidiAudio, MidiAudioTrackHandle};

use crate::dialogue::spawn_dialogue;
use crate::fray::FrayPlugin;
use crate::input::{ActionButtonEvent, InputManagerReference};
use crate::menus::MenuStack;
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

#[add_system(
	plugin = FrayPlugin, schedule = Update,
	generics = <TrackSwitcher>,
)]
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
        InputMap::<TrackSwitcherAction>::default(),
    );
    dialogue.add_option(
        &mut commands,
        "4/4".to_owned(),
        TrackSwitcherFourFour {
            dialogue: dialogue.root,
        },
    );
    dialogue.add_option(
        &mut commands,
        "6/8".to_owned(),
        TrackSwitcherSixEight {
            dialogue: dialogue.root,
        },
    );
}

#[add_system(
	plugin = FrayPlugin, schedule = Update,
	generics = <TrackSwitcherFourFour>,
)]
#[add_system(
	plugin = FrayPlugin, schedule = Update,
	generics = <TrackSwitcherSixEight>,
)]
use crate::input::fire_action_button_events;

#[add_observer(plugin = FrayPlugin)]
fn switch_track(switch_track: On<SwitchTrack>, mut fray_tracks: ResMut<FrayTracks>) {
    fray_tracks.set_player_track(switch_track.track);
}

#[derive(Component)]
pub struct TrackSwitcherFourFour {
    pub dialogue: Entity,
}
impl InputManagerReference for TrackSwitcherFourFour {
    fn input_manager(&self) -> Entity {
        self.dialogue
    }
}
impl ActionButtonEvent for TrackSwitcherFourFour {
    type Action = TrackSwitcherAction;
    type Button = Self;
    type Event = SwitchTrack;

    fn make_event_system() -> impl IntoSystem<In<Entity>, Self::Event, ()> {
        IntoSystem::into_system(|In(dialogue): In<Entity>| SwitchTrack {
            track: Track::FourFour,
            dialogue,
        })
    }

    fn action() -> Self::Action {
        TrackSwitcherAction::FourFour
    }
}

#[derive(Component)]
pub struct TrackSwitcherSixEight {
    pub dialogue: Entity,
}
impl InputManagerReference for TrackSwitcherSixEight {
    fn input_manager(&self) -> Entity {
        self.dialogue
    }
}
impl ActionButtonEvent for TrackSwitcherSixEight {
    type Action = TrackSwitcherAction;
    type Button = Self;
    type Event = SwitchTrack;

    fn make_event_system() -> impl IntoSystem<In<Entity>, Self::Event, ()> {
        IntoSystem::into_system(|In(dialogue): In<Entity>| SwitchTrack {
            track: Track::SixEight,
            dialogue,
        })
    }

    fn action() -> Self::Action {
        TrackSwitcherAction::SixEight
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Reflect, Debug)]
pub enum TrackSwitcherAction {
    FourFour,
    SixEight,
}
impl Actionlike for TrackSwitcherAction {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            TrackSwitcherAction::FourFour => InputControlKind::Button,
            TrackSwitcherAction::SixEight => InputControlKind::Button,
        }
    }
}

#[derive(Clone, Copy)]
pub enum Track {
    FourFour,
    SixEight,
}

#[add_observer(plugin = FrayPlugin, generics = <SwitchTrack>)]
use crate::menus::close_menu_on_event;

#[derive(Event)]
pub struct SwitchTrack {
    pub track: Track,
    pub dialogue: Entity,
}
impl InputManagerReference for SwitchTrack {
    fn input_manager(&self) -> Entity {
        self.dialogue
    }
}
