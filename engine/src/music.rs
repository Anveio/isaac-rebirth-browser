//! Original music channels and controller; device completion is explicit input.
use serde::{Deserialize, Serialize};

pub struct MusicTrack {
    pub id: u32,
    pub path: &'static str,
    pub intro: Option<&'static str>,
    pub layer: Option<&'static str>,
    pub layer_intro: Option<&'static str>,
    pub looping: bool,
}
include!(concat!(env!("OUT_DIR"), "/music_catalog.rs"));

fn track(id: u32) -> Option<&'static MusicTrack> {
    MUSIC_TRACKS.iter().find(|track| track.id == id)
}
fn authored_path(path: &str) -> bool {
    MUSIC_TRACKS.iter().any(|track| {
        [
            Some(track.path),
            track.intro,
            track.layer,
            track.layer_intro,
        ]
        .contains(&Some(path))
    })
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MusicChannel {
    pub generation: Option<u64>,
    pub backend_path: Option<String>,
    pub queued_path: Option<String>,
    pub queued_looping: bool,
    pub wrapper_looping: bool,
    pub backend_looping: bool,
    pub backend_playing: bool,
    pub playing: bool,
    pub updates_since_prepare: i32,
    pub volume: f32,
    pub target_volume: f32,
    pub fade_rate: f32,
    pub gain: f32,
    pub pitch: f32,
}
impl MusicChannel {
    fn empty(rate: f32) -> Self {
        Self {
            generation: None,
            backend_path: None,
            queued_path: None,
            queued_looping: false,
            wrapper_looping: false,
            backend_looping: false,
            backend_playing: false,
            playing: false,
            updates_since_prepare: 0,
            volume: 1.0,
            target_volume: 1.0,
            fade_rate: rate,
            gain: 1.0,
            pitch: 1.0,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum MusicEvent {
    Start {
        channel: u8,
        generation: u64,
        path: String,
        looping: bool,
        queued_path: Option<String>,
        queued_looping: bool,
        gain: f32,
        pitch: f32,
    },
    Stop {
        channel: u8,
        generation: u64,
    },
    Gain {
        channel: u8,
        generation: u64,
        gain: f32,
    },
    Pitch {
        channel: u8,
        generation: u64,
        pitch: f32,
    },
    Loop {
        channel: u8,
        generation: u64,
        looping: bool,
    },
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum BackendEvent {
    Ended,
    Loop,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum MusicRequest {
    Crossfade { id: u32 },
    Room { id: u32 },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(
    rename_all = "camelCase",
    deny_unknown_fields,
    try_from = "SavedMusicState"
)]
pub struct MusicState {
    pub volume: f32,
    pub music_enabled: bool,
    pub active_channel: u8,
    pub current_id: u32,
    pub queued_id: u32,
    pub layer_enabled: bool,
    pub pitch: f32,
    pub target_pitch: f32,
    pub channels: [MusicChannel; 3],
    pub next_generation: u64,
    pub events: Vec<MusicEvent>,
    pub pending: Vec<MusicRequest>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SavedMusicState {
    volume: f32,
    music_enabled: bool,
    active_channel: u8,
    current_id: u32,
    queued_id: u32,
    layer_enabled: bool,
    pitch: f32,
    target_pitch: f32,
    channels: [MusicChannel; 3],
    next_generation: u64,
    events: Vec<MusicEvent>,
    pending: Vec<MusicRequest>,
}
impl TryFrom<SavedMusicState> for MusicState {
    type Error = String;
    fn try_from(saved: SavedMusicState) -> Result<Self, Self::Error> {
        if !saved.volume.is_finite()
            || !(0.0..=1.0).contains(&saved.volume)
            || saved.music_enabled != (saved.volume > 0.0)
            || saved.active_channel > 1
            || (saved.current_id != 0 && track(saved.current_id).is_none())
            || (saved.queued_id != 0 && track(saved.queued_id).is_none())
            || !saved.pitch.is_finite()
            || !saved.target_pitch.is_finite()
            || saved.pitch <= 0.0
            || saved.target_pitch <= 0.0
        {
            return Err("music continuation has invalid controller fields".into());
        }
        let mut generations = std::collections::BTreeSet::new();
        for (index, channel) in saved.channels.iter().enumerate() {
            if channel.fade_rate != if index == 2 { 0.02_f32 } else { 0.08_f32 }
                || !channel.volume.is_finite()
                || !channel.target_volume.is_finite()
                || !channel.gain.is_finite()
                || !channel.pitch.is_finite()
                || channel.pitch <= 0.0
                || channel
                    .backend_path
                    .as_deref()
                    .is_some_and(|path| !authored_path(path))
                || channel
                    .queued_path
                    .as_deref()
                    .is_some_and(|path| !authored_path(path))
                || channel.generation.is_some() != channel.backend_path.is_some()
                || channel.generation.is_some_and(|generation| {
                    generation >= saved.next_generation || !generations.insert(generation)
                })
                || (channel.generation.is_none()
                    && (channel.playing
                        || channel.backend_playing
                        || channel.queued_path.is_some()))
            {
                return Err("music continuation has invalid authored channels".into());
            }
        }
        for request in &saved.pending {
            let id = match request {
                MusicRequest::Crossfade { id } | MusicRequest::Room { id } => *id,
            };
            if track(id).is_none() {
                return Err("music continuation has an unconfigured pending ID".into());
            }
        }
        for event in &saved.events {
            let (channel, generation) = match event {
                MusicEvent::Start {
                    channel,
                    generation,
                    path,
                    queued_path,
                    gain,
                    pitch,
                    ..
                } => {
                    if !authored_path(path)
                        || queued_path
                            .as_deref()
                            .is_some_and(|path| !authored_path(path))
                        || !gain.is_finite()
                        || !pitch.is_finite()
                        || *pitch <= 0.0
                    {
                        return Err("music continuation has invalid stream preparation".into());
                    }
                    (*channel, *generation)
                }
                MusicEvent::Stop {
                    channel,
                    generation,
                } => (*channel, *generation),
                MusicEvent::Loop {
                    channel,
                    generation,
                    ..
                } => (*channel, *generation),
                MusicEvent::Gain {
                    channel,
                    generation,
                    gain,
                } => {
                    if !gain.is_finite() {
                        return Err("music continuation has invalid gain".into());
                    }
                    (*channel, *generation)
                }
                MusicEvent::Pitch {
                    channel,
                    generation,
                    pitch,
                } => {
                    if !pitch.is_finite() || *pitch <= 0.0 {
                        return Err("music continuation has invalid pitch".into());
                    }
                    (*channel, *generation)
                }
            };
            if channel > 2 || generation >= saved.next_generation {
                return Err("music continuation has invalid output generation".into());
            }
        }
        Ok(Self {
            volume: saved.volume,
            music_enabled: saved.music_enabled,
            active_channel: saved.active_channel,
            current_id: saved.current_id,
            queued_id: saved.queued_id,
            layer_enabled: saved.layer_enabled,
            pitch: saved.pitch,
            target_pitch: saved.target_pitch,
            channels: saved.channels,
            next_generation: saved.next_generation,
            events: saved.events,
            pending: saved.pending,
        })
    }
}

impl Default for MusicState {
    fn default() -> Self {
        let mut state = Self {
            volume: 1.0,
            music_enabled: true,
            active_channel: 0,
            current_id: 0,
            queued_id: 0,
            layer_enabled: false,
            pitch: 1.0,
            target_pitch: 1.0,
            channels: [
                MusicChannel::empty(0.08),
                MusicChannel::empty(0.08),
                MusicChannel::empty(0.02),
            ],
            next_generation: 0,
            events: Vec::new(),
            pending: Vec::new(),
        };
        // Browser entry is first Title. The earlier native intro video/voiceover
        // is outside this entry point; no invented seek or tail is played.
        state.crossfade(61);
        state.advance();
        state
    }
}

impl MusicState {
    pub fn begin_command(&mut self) {
        self.events.clear();
    }
    /// GameStop54F41C ->5B31B0 resets controller pitch and active/layer wrappers.
    pub fn reset_pitch(&mut self) {
        self.pitch = 1.0;
        self.target_pitch = 1.0;
        for index in [usize::from(self.active_channel), 2] {
            self.channels[index].pitch = 1.0;
            if let Some(generation) = self.channels[index].generation {
                self.events.push(MusicEvent::Pitch { channel: index as u8, generation, pitch: 1.0 });
            }
        }
    }
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
        self.music_enabled = volume > 0.0;
        let channel = &mut self.channels[usize::from(self.active_channel)];
        channel.volume = volume;
        channel.target_volume = volume;
    }
    fn unload(&mut self, index: usize) {
        let channel = &mut self.channels[index];
        if let Some(generation) = channel.generation.take() {
            self.events.push(MusicEvent::Stop {
                channel: index as u8,
                generation,
            });
        }
        channel.backend_path = None;
        channel.queued_path = None;
        channel.backend_playing = false;
    }
    fn stop(&mut self, index: usize) {
        if !self.channels[index].playing {
            return;
        }
        self.unload(index);
        self.channels[index].playing = false;
        self.channels[index].updates_since_prepare = 0;
    }
    fn prepare(
        &mut self,
        index: usize,
        path: &str,
        looping: bool,
        queued: Option<&str>,
        queued_looping: bool,
        volume: f32,
    ) -> MusicEvent {
        self.unload(index);
        let generation = self.next_generation;
        self.next_generation = self
            .next_generation
            .checked_add(1)
            .expect("music generation exhausted");
        let gain = if self.music_enabled { volume } else { 0.0 };
        let channel = &mut self.channels[index];
        channel.generation = Some(generation);
        channel.backend_path = Some(path.into());
        channel.queued_path = queued.map(str::to_owned);
        channel.queued_looping = queued_looping;
        channel.wrapper_looping = looping;
        channel.backend_looping = looping;
        channel.backend_playing = true;
        channel.playing = true;
        channel.updates_since_prepare = 0;
        channel.volume = volume;
        channel.target_volume = volume;
        channel.gain = gain;
        channel.pitch = self.pitch;
        MusicEvent::Start {
            channel: index as u8,
            generation,
            path: path.into(),
            looping,
            queued_path: queued.map(str::to_owned),
            queued_looping,
            gain,
            pitch: self.pitch,
        }
    }
    fn prepare_track(&mut self, configured: &MusicTrack, volume: f32) {
        let main = usize::from(self.active_channel);
        let main_start = self.prepare(
            main,
            configured.intro.unwrap_or(configured.path),
            configured.intro.is_none() && configured.looping,
            configured.intro.map(|_| configured.path),
            configured.looping,
            volume,
        );
        self.stop(2);
        if let Some(layer) = configured.layer {
            let layer_start = self.prepare(
                2,
                configured.layer_intro.unwrap_or(layer),
                configured.layer_intro.is_none(),
                configured.layer_intro.map(|_| layer),
                true,
                0.0,
            );
            self.events.push(layer_start);
        }
        self.events.push(main_start);
    }
    /// Source5b2c20 ->5b2a30 stops both main channels and plays at configured volume.
    pub fn play(&mut self, id: u32) {
        let Some(configured) = track(id) else {
            return;
        };
        self.stop(0);
        self.stop(1);
        self.current_id = id;
        self.queued_id = 0;
        self.prepare_track(configured, self.volume);
    }
    /// Source5b3190 stores the next track; real backend completion admits it.
    pub fn queue(&mut self, id: u32) {
        if track(id).is_some() {
            self.queued_id = id;
        }
    }
    pub fn crossfade(&mut self, id: u32) {
        let Some(configured) = track(id) else {
            return;
        };
        if self.current_id == id {
            self.channels[usize::from(self.active_channel)].target_volume = self.volume;
            return;
        }
        let old = usize::from(self.active_channel);
        self.channels[old].target_volume = 0.0;
        self.channels[old].wrapper_looping = false;
        self.channels[old].backend_looping = false;
        if let Some(generation) = self.channels[old].generation {
            self.events.push(MusicEvent::Loop {
                channel: old as u8,
                generation,
                looping: false,
            });
        }
        self.active_channel = (self.active_channel + 1) % 2;
        self.current_id = id;
        self.queued_id = 0;
        self.prepare_track(configured, 0.0);
        self.channels[usize::from(self.active_channel)].target_volume = self.volume;
    }
    fn play_fade_in(&mut self, id: u32) {
        let Some(configured) = track(id) else {
            return;
        };
        self.stop(0);
        self.stop(1);
        self.current_id = id;
        self.queued_id = 0;
        self.prepare_track(configured, 0.0);
        self.channels[usize::from(self.active_channel)].target_volume = self.volume;
    }
    pub fn room(&mut self, id: u32) {
        if track(id).is_none() {
            return;
        }
        if self.current_id == 95 {
            self.queued_id = id;
        } else {
            self.crossfade(id);
        }
    }
    pub fn flush_pending(&mut self) {
        for request in std::mem::take(&mut self.pending) {
            match request {
                MusicRequest::Crossfade { id } => self.crossfade(id),
                MusicRequest::Room { id } => self.room(id),
            }
        }
    }
    pub fn backend(&mut self, index: u8, generation: u64, event: BackendEvent) {
        let Some(channel) = self.channels.get_mut(usize::from(index)) else {
            return;
        };
        if channel.generation != Some(generation) {
            return;
        }
        match event {
            BackendEvent::Ended => channel.backend_playing = false,
            BackendEvent::Loop => {
                if let Some(path) = channel.queued_path.take() {
                    channel.backend_path = Some(path);
                    channel.backend_looping = channel.queued_looping;
                }
            }
        }
    }
    fn advance_channel(&mut self, index: usize, stop_at_zero: bool) {
        let channel = &mut self.channels[index];
        if channel.updates_since_prepare > 10 {
            channel.playing = channel.backend_playing;
        }
        if !channel.playing {
            self.unload(index);
            return;
        }
        channel.updates_since_prepare = channel.updates_since_prepare.wrapping_add(1);
        let delta = channel.target_volume - channel.volume;
        let snapped = delta.abs() <= 0.001_f32 && channel.target_volume != channel.volume;
        if delta.abs() > 0.001_f32 {
            channel.volume += delta * channel.fade_rate;
        } else if snapped {
            channel.volume = channel.target_volume;
        }
        let gain = if self.music_enabled {
            channel.volume
        } else {
            0.0
        };
        if delta.abs() > 0.001_f32 || snapped || gain != channel.gain {
            channel.gain = gain;
            self.events.push(MusicEvent::Gain {
                channel: index as u8,
                generation: channel.generation.expect("playing channel has backend"),
                gain,
            });
        }
        // Equal-zero/matching-gain takes the native early return. Only a
        // non-equal volume snapped to zero enters the old-channel Stop branch.
        if snapped && channel.volume == 0.0 && stop_at_zero {
            self.stop(index);
        }
    }
    pub fn advance(&mut self) {
        self.advance_channel(0, self.active_channel != 0);
        self.advance_channel(1, self.active_channel != 1);
        if self.channels[2].playing {
            self.channels[2].target_volume = if self.layer_enabled { self.volume } else { 0.0 };
        }
        self.advance_channel(2, false);
        let delta = self.target_pitch - self.pitch;
        if delta.abs() > 0.001_f32 {
            self.pitch += delta * 0.1_f32;
            for (index, channel) in self.channels.iter_mut().enumerate() {
                channel.pitch = self.pitch;
                if let Some(generation) = channel.generation {
                    self.events.push(MusicEvent::Pitch {
                        channel: index as u8,
                        generation,
                        pitch: self.pitch,
                    });
                }
            }
        }
        if !self.channels[usize::from(self.active_channel)].playing && self.queued_id != 0 {
            self.play_fade_in(self.queued_id);
        }
    }
}

// Level stage and alternate flag select music. A room's authored asset-stage
// can be zero for the special starting template and is not the Level stage.
pub fn stage_track(stage: u32, alternate: bool) -> Option<u32> {
    Some(match stage {
        1 | 2 => {
            if alternate {
                4
            } else {
                1
            }
        }
        3 | 4 => {
            if alternate {
                5
            } else {
                2
            }
        }
        5 | 6 => {
            if alternate {
                6
            } else {
                3
            }
        }
        7 | 8 => 7,
        9 => {
            if alternate {
                10
            } else {
                11
            }
        }
        11 => {
            if alternate {
                13
            } else {
                12
            }
        }
        _ => return None,
    })
}
