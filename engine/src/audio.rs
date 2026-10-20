//! Source SFX admission, independent of browser autoplay and voice allocation.
use crate::mt::Mt19937;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

include!(concat!(env!("OUT_DIR"), "/audio_catalog.rs"));

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SoundRequest {
    pub id: u32,
    pub volume: f32,
    pub delay: i32,
    pub looping: bool,
    pub pitch: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SampleSettings {
    pub looping: bool,
    pub gain: f32,
    pub pitch: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SoundSlot {
    // Virgin native lastTick bytes are not an admitted-event timestamp.
    pub last_tick: Option<i32>,
    pub next_allowed: i32,
    pub last_volume: f32,
    pub loaded: bool,
    pub samples: Vec<SampleSettings>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SoundEvent {
    pub sequence: u64,
    pub id: u32,
    pub sample_index: u32,
    pub system_counter: u32,
    pub half_tick: i32,
    pub looping: bool,
    pub pitch: f32,
    pub gain: f32,
}

///5B97A0/5E1DB0 callback, ordered before the next admitted sound sequence.
///Loaded slots retain their request volume; actual playing ownership is backend.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VolumeCallback {
    pub sequence: u64,
    pub before_sound_sequence: u64,
    pub volume: f32,
    pub loaded_slot_volumes: BTreeMap<u32, f32>,
}

///5E1D00 queries actual backend playing/looping for every loaded sample.
///This ordered request carries loaded slots; backend voice life remains there.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StopLoopingCallback {
    pub sequence: u64,
    pub before_sound_sequence: u64,
    pub before_volume_sequence: u64,
    pub loaded_slots: Vec<u32>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(
    rename_all = "camelCase",
    deny_unknown_fields,
    try_from = "SavedAudioState"
)]
pub struct AudioState {
    pub sfx_volume: f32,
    pub slots: BTreeMap<u32, SoundSlot>,
    pub next_sequence: u64,
    // The current command's admissions, consumed once by browser playback.
    pub events: Vec<SoundEvent>,
    pub pending: Vec<SoundRequest>,
    pub next_volume_sequence: u64,
    pub volume_callbacks: Vec<VolumeCallback>,
    pub next_stop_sequence: u64,
    pub stop_looping_callbacks: Vec<StopLoopingCallback>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SavedAudioState {
    sfx_volume: f32,
    slots: BTreeMap<u32, SoundSlot>,
    next_sequence: u64,
    events: Vec<SoundEvent>,
    pending: Vec<SoundRequest>,
    #[serde(default)]
    next_volume_sequence: u64,
    #[serde(default)]
    volume_callbacks: Vec<VolumeCallback>,
    #[serde(default)]
    next_stop_sequence: u64,
    #[serde(default)]
    stop_looping_callbacks: Vec<StopLoopingCallback>,
}

impl TryFrom<SavedAudioState> for AudioState {
    type Error = String;

    fn try_from(saved: SavedAudioState) -> Result<Self, Self::Error> {
        if !saved.sfx_volume.is_finite()
            || !(0.0..=1.0).contains(&saved.sfx_volume)
            || saved.slots.len() != SOUND_COUNTS.iter().filter(|count| **count > 0).count()
        {
            return Err(
                "audio continuation requires a complete configured catalogue and valid SFX volume"
                    .into(),
            );
        }
        for (id, slot) in &saved.slots {
            let count = SOUND_COUNTS.get(*id as usize).copied().unwrap_or(0);
            if count == 0
                || usize::from(count) != slot.samples.len()
                || !slot.last_volume.is_finite()
                || slot
                    .samples
                    .iter()
                    .any(|sample| !sample.gain.is_finite() || !sample.pitch.is_finite())
            {
                return Err("audio continuation has invalid configured samples".into());
            }
        }
        let mut previous = None;
        for event in &saved.events {
            let count = SOUND_COUNTS.get(event.id as usize).copied().unwrap_or(0);
            if count == 0
                || event.sample_index >= u32::from(count)
                || event.sequence >= saved.next_sequence
                || previous.is_some_and(|sequence| event.sequence <= sequence)
                || event.half_tick != (event.system_counter as i32) >> 1
                || !event.gain.is_finite()
                || !event.pitch.is_finite()
            {
                return Err("audio continuation has invalid admitted events".into());
            }
            previous = Some(event.sequence);
        }
        if saved
            .pending
            .iter()
            .any(|request| !request.volume.is_finite() || !request.pitch.is_finite())
        {
            return Err("audio continuation has invalid pending requests".into());
        }
        let mut previous_callback = None;
        let mut previous_boundary = 0;
        for callback in &saved.volume_callbacks {
            if callback.sequence >= saved.next_volume_sequence
                || previous_callback.is_some_and(|sequence| callback.sequence <= sequence)
                || callback.before_sound_sequence < previous_boundary
                || callback.before_sound_sequence > saved.next_sequence
                || !callback.volume.is_finite()
                || !(0.0..=1.0).contains(&callback.volume)
                || callback
                    .loaded_slot_volumes
                    .iter()
                    .any(|(id, volume)| !saved.slots.contains_key(id) || !volume.is_finite())
            {
                return Err("audio continuation has invalid volume callbacks".into());
            }
            previous_callback = Some(callback.sequence);
            previous_boundary = callback.before_sound_sequence;
        }
        let mut previous_stop = None;
        let mut previous_stop_boundary = 0;
        let mut previous_volume_boundary = 0;
        for callback in &saved.stop_looping_callbacks {
            if callback.sequence >= saved.next_stop_sequence
                || previous_stop.is_some_and(|sequence| callback.sequence <= sequence)
                || callback.before_sound_sequence < previous_stop_boundary
                || callback.before_sound_sequence > saved.next_sequence
                || callback.before_volume_sequence < previous_volume_boundary
                || callback.before_volume_sequence > saved.next_volume_sequence
                || callback
                    .loaded_slots
                    .windows(2)
                    .any(|pair| pair[0] >= pair[1])
                || callback
                    .loaded_slots
                    .iter()
                    .any(|id| !saved.slots.contains_key(id))
            {
                return Err("audio continuation has invalid Stop-looping callbacks".into());
            }
            previous_stop = Some(callback.sequence);
            previous_stop_boundary = callback.before_sound_sequence;
            previous_volume_boundary = callback.before_volume_sequence;
        }
        Ok(Self {
            sfx_volume: saved.sfx_volume,
            slots: saved.slots,
            next_sequence: saved.next_sequence,
            events: saved.events,
            pending: saved.pending,
            next_volume_sequence: saved.next_volume_sequence,
            volume_callbacks: saved.volume_callbacks,
            next_stop_sequence: saved.next_stop_sequence,
            stop_looping_callbacks: saved.stop_looping_callbacks,
        })
    }
}

impl Default for AudioState {
    fn default() -> Self {
        let slots = SOUND_COUNTS
            .iter()
            .enumerate()
            .filter(|(_, count)| **count > 0)
            .map(|(id, count)| {
                (
                    id as u32,
                    SoundSlot {
                        last_tick: None,
                        next_allowed: -1,
                        last_volume: 0.0,
                        // Original startup loads configured sounds before first Title.
                        loaded: true,
                        samples: (0..*count)
                            .map(|_| SampleSettings {
                                looping: false,
                                gain: 1.0,
                                pitch: 1.0,
                            })
                            .collect(),
                    },
                )
            })
            .collect();
        Self {
            sfx_volume: 0.3,
            slots,
            next_sequence: 0,
            events: Vec::new(),
            pending: Vec::new(),
            next_volume_sequence: 0,
            volume_callbacks: Vec::new(),
            next_stop_sequence: 0,
            stop_looping_callbacks: Vec::new(),
        }
    }
}

impl AudioState {
    pub fn admit(
        &mut self,
        shared: &mut Mt19937,
        system_counter: u32,
        request: SoundRequest,
    ) -> Option<SoundEvent> {
        let count = u32::from(*SOUND_COUNTS.get(request.id as usize)?);
        if count == 0 {
            return None;
        }
        let slot = self.slots.get_mut(&request.id)?;
        // Native loading precedes the cooldown check. Output availability never
        // suppresses this logical state or the admitted shared-stream draw.
        slot.loaded = true;
        let half_tick = (system_counter as i32) >> 1;
        if slot.next_allowed >= 0 && half_tick < slot.next_allowed {
            return None;
        }
        slot.last_tick = Some(half_tick);
        slot.next_allowed = half_tick.wrapping_add(request.delay);
        slot.last_volume = request.volume;
        let sample_index = shared.bounded(count);
        let gain = (self.sfx_volume * request.volume).min(1.0);
        slot.samples[sample_index as usize] = SampleSettings {
            looping: request.looping,
            gain,
            pitch: request.pitch,
        };
        let event = SoundEvent {
            sequence: self.next_sequence,
            id: request.id,
            sample_index,
            system_counter,
            half_tick,
            looping: request.looping,
            pitch: request.pitch,
            gain,
        };
        self.next_sequence = self
            .next_sequence
            .checked_add(1)
            .expect("audio event sequence exhausted");
        self.events.push(event.clone());
        Some(event)
    }

    pub fn set_volume(&mut self, volume: f32) -> Result<(), String> {
        if !volume.is_finite() || !(0.0..=1.0).contains(&volume) {
            return Err("SFX volume must be finite and within 0..1".into());
        }
        let sequence = self.next_volume_sequence;
        self.next_volume_sequence = sequence
            .checked_add(1)
            .ok_or("SFX volume callback sequence exhausted")?;
        self.sfx_volume = volume;
        self.volume_callbacks.push(VolumeCallback {
            sequence,
            before_sound_sequence: self.next_sequence,
            volume,
            loaded_slot_volumes: self
                .slots
                .iter()
                .filter(|(_, slot)| slot.loaded)
                .map(|(id, slot)| (*id, slot.last_volume))
                .collect(),
        });
        Ok(())
    }

    pub fn stop_looping(&mut self) -> Result<(), String> {
        let sequence = self.next_stop_sequence;
        self.next_stop_sequence = sequence
            .checked_add(1)
            .ok_or("SFX Stop callback sequence exhausted")?;
        self.stop_looping_callbacks.push(StopLoopingCallback {
            sequence,
            before_sound_sequence: self.next_sequence,
            before_volume_sequence: self.next_volume_sequence,
            loaded_slots: self
                .slots
                .iter()
                .filter(|(_, slot)| slot.loaded)
                .map(|(id, _)| *id)
                .collect(),
        });
        Ok(())
    }

    pub fn begin_command(&mut self) {
        self.events.clear();
        self.volume_callbacks.clear();
        self.stop_looping_callbacks.clear();
    }

    pub fn flush_pending(&mut self, shared: &mut Mt19937, system_counter: u32) {
        for request in std::mem::take(&mut self.pending) {
            self.admit(shared, system_counter, request);
        }
    }
}
