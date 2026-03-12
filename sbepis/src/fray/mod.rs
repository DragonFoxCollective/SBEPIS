use std::f32::consts::TAU;
use std::time::Duration;

use bevy::audio::Volume;
use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_pretty_nice_menus::close_menu_on_event;
use return_ok::ok_or_return_ok;
use soundyrust::*;
use tracks::{FrayTracks, Track};

use crate::fray::tracks::{SwitchTrack, TrackSwitcher};
use crate::npcs::imp::Imp;
use crate::player::weapons::Hit;
use crate::prelude::*;
use crate::util::MapRangeBetween;

mod tracks;

#[derive(AutoPlugin)]
#[auto_add_plugin(plugin = SbepisPlugin)]
pub struct FrayPlugin;

#[auto_plugin(plugin = FrayPlugin)]
fn build(app: &mut App) {
    app.add_plugins(soundyrust::SoundyPlugin);
    app.add_observer(interact_with::<TrackSwitcher>);
    app.add_observer(close_menu_on_event::<SwitchTrack>);
}

#[auto_system(plugin = FrayPlugin, schedule = Startup)]
fn load_background_music(mut commands: Commands, mut assets: ResMut<Assets<MidiAudio>>) {
    let mut midi = MidiAudio::from_bytes(include_bytes!("../../assets/hl4mgm.sf2"));
    let backing_track = midi.add_track(
        MidiAudioTrack::from_bytes(include_bytes!("../../assets/fray backing.mid"), 4.0 / 4.0)
            .with_channel_patch(0, 0, 3)
            .with_channel_patch(1, 128, 0)
            .with_channel_patch(2, 0, 0),
    );
    let four_four = midi.add_track(
        MidiAudioTrack::from_bytes(include_bytes!("../../assets/fray 4⁄4 lead.mid"), 4.0 / 4.0)
            .with_channel_patch(0, 0, 1)
            .stopped(),
    );
    let six_eight = midi.add_track(
        MidiAudioTrack::from_bytes(include_bytes!("../../assets/fray 6⁄8 lead.mid"), 6.0 / 8.0)
            .with_channel_patch(0, 0, 1)
            .stopped(),
    );

    midi.queue(
        four_four,
        MidiQueueEvent {
            event: MidiQueueEventType::Queue(Box::new(MidiQueueEvent {
                event: MidiQueueEventType::Stop,
                timing: MidiQueueTiming::Bar,
                looping: MidiQueueLooping::Once,
            })),
            timing: MidiQueueTiming::Bar,
            looping: MidiQueueLooping::Loop,
        },
    );
    midi.queue(
        six_eight,
        MidiQueueEvent {
            event: MidiQueueEventType::Queue(Box::new(MidiQueueEvent {
                event: MidiQueueEventType::Stop,
                timing: MidiQueueTiming::Bar,
                looping: MidiQueueLooping::Once,
            })),
            timing: MidiQueueTiming::Bar,
            looping: MidiQueueLooping::Loop,
        },
    );

    commands.insert_resource(FrayTracks {
        midi: assets.add(midi),
        player: Track::FourFour,
        imp: Track::SixEight,
        backing_track,
        four_four,
        six_eight,
    });
}

#[auto_system(plugin = FrayPlugin, schedule = OnEnter(GameState::InGame))]
fn spawn_background_music(mut commands: Commands, tracks: Res<FrayTracks>) {
    commands.spawn((
        AudioPlayer(tracks.midi.clone()),
        PlaybackSettings::LOOP
            .with_volume(Volume::Linear(0.2))
            .paused(),
        Name::new("Background Music"),
        FrayMusic::new(tracks.backing_track),
        DespawnOnExit(GameState::InGame),
    ));

    commands.spawn((
        Name::new("Beat Counter"),
        BeatCounter::default(),
        Text("".to_owned()),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            left: Val::Px(5.0),
            ..default()
        },
        PlayerCameraNode,
        DespawnOnExit(GameState::InGame),
    ));
}

#[auto_component(plugin = FrayPlugin, derive, reflect, register)]
pub struct FrayMusic {
    beat: f64,
    beats_per_bar: f64,
    beats_per_second: f64,
    delay: Option<Duration>,
    backing_track: MidiAudioTrackHandle,
}

impl FrayMusic {
    fn new(backing_track: MidiAudioTrackHandle) -> Self {
        Self {
            beat: 0.0,
            beats_per_bar: 1.0,
            beats_per_second: 0.0,
            delay: Some(Duration::from_secs_f32(1.0)),
            backing_track,
        }
    }

    pub fn tick(&mut self, delta: Duration, midi_audio: &MidiAudio) {
        self.beats_per_second = midi_audio.beats_per_second(&self.backing_track).unwrap() / 2.0;
        self.beats_per_bar = midi_audio.beats_per_bar(&self.backing_track).unwrap();
        self.beat += self.time_to_bpm_beat(delta);
        self.beat %= self.beats_per_bar;
    }

    pub fn subbeats(&self, divisions: u32) -> u32 {
        (self.beat * divisions as f64).floor() as u32
    }

    pub fn beat_progress(&self) -> f32 {
        self.beat.fract() as f32
    }

    pub fn modify_fray_damage(&self, damage: f32) -> f32 {
        let modifier = self.single_beat_modifier(1.0)
            + self.single_beat_modifier(2.0)
            + self.single_beat_modifier(4.0)
            + self.single_beat_modifier(8.0);
        damage * modifier
    }

    fn single_beat_modifier(&self, factor: f32) -> f32 {
        (self.beat_progress() * factor)
            .map_range_between(0.0..1.0, 0.0..TAU)
            .cos()
            .map_range_between(-1.0..1.0, 0.0..1.0)
            / factor
    }

    pub fn time_to_bpm_beat(&self, time: Duration) -> f64 {
        time.as_secs_f64() * self.beats_per_second
    }

    pub fn speed(&self) -> f32 {
        self.beats_per_second as f32
    }
}

#[auto_component(plugin = FrayPlugin, derive(Default), reflect, register)]
pub struct BeatCounter {
    pub beat: u32,
}

#[auto_system(plugin = FrayPlugin, schedule = Update)]
fn tick_fray_music(
    #[cfg(feature = "metronome")] mut commands: Commands,
    #[cfg(feature = "metronome")] asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut fray_musics: Query<(&mut FrayMusic, &AudioSink, &AudioPlayer<MidiAudio>)>,
    mut beat_counters: Query<(&mut BeatCounter, &mut Text)>,
    mut assets: ResMut<Assets<MidiAudio>>,
) -> Result {
    let (mut beat_counter, mut beat_counter_text) = ok_or_return_ok!(beat_counters.single_mut());
    for (mut fray_music, audio_sink, midi_audio) in fray_musics.iter_mut() {
        if let Some(delay) = fray_music.delay {
            fray_music.delay = delay.checked_sub(time.delta());

            #[cfg(feature = "metronome")]
            if fray_music.delay.is_none() {
                commands.spawn((
                    Name::new("Beat"),
                    AudioPlayer::new(asset_server.load("unlicensed/metronome.mp3")),
                    PlaybackSettings::DESPAWN,
                ));
            }

            continue;
        }

        let midi_audio = assets
            .get_mut(&midi_audio.0)
            .ok_or("Midi audio not found")?;
        audio_sink.play(); // this should really be phased out or smth
        fray_music.tick(time.delta(), midi_audio);
        let beat = fray_music.subbeats(1);
        let beat_progress = fray_music.beat_progress();

        beat_counter_text.0 = format!("{beat} {beat_progress:.2}");

        #[cfg(feature = "metronome")]
        if beat_counter.beat != beat {
            commands.spawn((
                Name::new("Beat"),
                AudioPlayer::new(asset_server.load("unlicensed/metronome.mp3")),
                PlaybackSettings::DESPAWN.with_speed(if beat % 4 == 0 { 1.0 } else { 0.5 }),
            ));
        }

        beat_counter.beat = beat;
    }

    Ok(())
}

#[auto_observer(plugin = FrayPlugin)]
fn queue_tracks_on_hit(
    hit: On<Hit>,
    imps: Query<(), With<Imp>>,
    players: Query<(), With<Player>>,
    audio_players: Query<&AudioPlayer<MidiAudio>>,
    mut assets: ResMut<Assets<MidiAudio>>,
    fray_tracks: Res<FrayTracks>,
) -> Result {
    let audio = assets
        .get_mut(&ok_or_return_ok!(audio_players.single()).0)
        .ok_or("Midi audio not found")?;

    if imps.get(hit.perpetrator).is_ok() {
        audio.queue(
            fray_tracks.imp_track(),
            MidiQueueEvent {
                event: MidiQueueEventType::Play,
                timing: MidiQueueTiming::Bar,
                looping: MidiQueueLooping::Once,
            },
        );
    }
    if players.get(hit.perpetrator).is_ok() {
        audio.queue(
            fray_tracks.player_track(),
            MidiQueueEvent {
                event: MidiQueueEventType::Play,
                timing: MidiQueueTiming::Bar,
                looping: MidiQueueLooping::Once,
            },
        );
    }

    Ok(())
}
