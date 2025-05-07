use bevy::prelude::*;
use bevy_butler::*;
use leafwing_input_manager::prelude::*;
use soundyrust::MidiAudioTrackHandle;

use crate::dialogue::spawn_dialogue;
use crate::fray::FrayPlugin;
use crate::input::{ActionButtonEvent, InputManagerReference};
use crate::menus::{MenuManipulationSet, MenuStack};
use crate::prelude::InteractedWithSet;

#[derive(Component, Reflect)]
#[reflect(Component)]
#[register_type(plugin = FrayPlugin)]
pub struct TrackSwitcher;

#[add_event(plugin = FrayPlugin, generics = <TrackSwitcher>)]
use crate::prelude::InteractedWith;

#[derive(Resource)]
pub struct FrayTracks {
    pub player: Track,
    pub imp: Track,
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
	in_set = InteractedWithTrackSwitcherSet::default(),
)]
use crate::player_controller::camera_controls::interact_with;

type InteractedWithTrackSwitcherSet = InteractedWithSet<TrackSwitcher>;

#[add_system(
	plugin = FrayPlugin, schedule = Update,
	after = InteractedWithTrackSwitcherSet::default(),
	in_set = MenuManipulationSet,
)]
fn open_track_switch_dialogue(
    mut ev_interact: EventReader<InteractedWith<TrackSwitcher>>,
    mut commands: Commands,
    mut menu_stack: ResMut<MenuStack>,
) {
    for _ev in ev_interact.read() {
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
}

#[add_system(
	plugin = FrayPlugin, schedule = Update,
	generics = <TrackSwitcherFourFour>,
	in_set = TrackSwitchedSet,
)]
#[add_system(
	plugin = FrayPlugin, schedule = Update,
	generics = <TrackSwitcherSixEight>,
	in_set = TrackSwitchedSet,
)]
use crate::input::fire_action_button_events;

#[add_system(
	plugin = FrayPlugin, schedule = Update,
	after = TrackSwitchedSet,
)]
fn switch_track(
    mut ev_track_switched: EventReader<TrackSwitched>,
    mut fray_tracks: ResMut<FrayTracks>,
) {
    for ev in ev_track_switched.read() {
        fray_tracks.set_player_track(ev.track);
    }
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
    type Event = TrackSwitched;

    fn make_event_system() -> impl IntoSystem<In<Entity>, Self::Event, ()> {
        IntoSystem::into_system(|In(dialogue): In<Entity>| TrackSwitched {
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
    type Event = TrackSwitched;

    fn make_event_system() -> impl IntoSystem<In<Entity>, Self::Event, ()> {
        IntoSystem::into_system(|In(dialogue): In<Entity>| TrackSwitched {
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

#[add_system(
	plugin = FrayPlugin, schedule = Update,
	generics = <TrackSwitched>,
	after = TrackSwitchedSet,
	in_set = MenuManipulationSet,
)]
use crate::menus::close_menu_on_event;

#[derive(Event, Clone, Copy)]
#[add_event(plugin = FrayPlugin)]
pub struct TrackSwitched {
    pub track: Track,
    pub dialogue: Entity,
}
impl InputManagerReference for TrackSwitched {
    fn input_manager(&self) -> Entity {
        self.dialogue
    }
}
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrackSwitchedSet;
