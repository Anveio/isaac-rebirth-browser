//! The Player's costumes (Player+AC0 entries, AD8 slot table), one framework
//! path for every collectible. Each costume comes from the imported
//! costumes2.xml row (`content::COSTUME_CONFIGS`); nothing here is per item.
//!
//! - Add (52CC90): 52CF00 clears every assigned entry's override, then a
//!   collectible whose ItemConfig has a costume (57BF00) registers it
//!   (511060): 5110B0/511400 erase its old entries, 5110D1 appends one entry
//!   and 51133A a copy when its actor has both head- and body-group layers.
//! - 50EDC0 assigns the 13 Player layer slots by layer name and priority,
//!   picks the head/body skin colours and resets each slot's Down0 pose.
//! - Player update (52FAED) syncs head entries to the head clip; the full
//!   update syncs body entries to the walk clip (532621/533A90) and advances
//!   each assigned entry once (532090).
//! - Render (5096A0/509700/5097D0/509870) draws slots 0..11, each from its
//!   costume entry's layer or else the Player's own layer.
//!
//! The sprite scale (Player+2C4) and the per-layer scales of the Player's
//! sprite are kept here too: 52DDD0 (Magic Mushroom) and 52D408/52D609 (Odd
//! Mushroom) write them, and 532090 copies them to the costume sprites.
use crate::{content, player_costume118::CostumeSprite, State};
use serde::{Deserialize, Serialize};

pub const SLOTS: usize = 13;
/// The Player actor's layer names (001.000_player.anm2 layers 0..12).
const PLAYER_LAYERS: [&str; SLOTS] = [
    "glow", "body", "body0", "body1", "head", "head0", "head1", "head2", "head3", "head4", "head5",
    "top0", "extra",
];

/// 50EE2B: slots 1..3 and 11 form the body group, every other slot the head.
pub fn body_group(slot: usize) -> bool {
    (1..=3).contains(&slot) || slot == 11
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Entry {
    /// The collectible whose ItemConfig carries this costume (+2E0).
    pub item: u32,
    pub actor: String,
    /// The entry's own AnimationState; `override_active` is +2EC.
    pub sprite: CostumeSprite,
    /// The `_Overlay` clip of a hasOverlay costume.
    pub overlay: Option<CostumeSprite>,
    pub priority: i32,
    pub is_flying: bool,
    pub has_overlay: bool,
    pub has_skin_alt: bool,
    pub skin_color: i32,
    pub overwrite_color: bool,
    /// +2F4: the skin colour its spritesheet was last loaded with.
    pub sheet_skin: i32,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Slot {
    pub entry: i32,
    pub layer: i32,
    pub priority: i32,
}
const EMPTY: Slot = Slot {
    entry: -1,
    layer: -1,
    priority: -1,
};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Wardrobe {
    pub entries: Vec<Entry>,
    pub slots: [Slot; SLOTS],
    pub head_skin: i32,
    pub body_skin: i32,
    /// Player+30DC: the skin colour of the Player's own spritesheet.
    pub player_skin: i32,
    /// Player+2C4.
    pub sprite_scale: [f32; 2],
    /// The Player sprite's layer scales (LayerState+24), copied to each
    /// costume layer that fills the slot.
    pub layer_scales: [[f32; 2]; SLOTS],
}

/// Normal Isaac's players.xml skinColor.
const ISAAC_SKIN: i32 = -1;

impl Wardrobe {
    /// Player.Init: 50EABA constructs the empty vector, 519A6E sets the
    /// sprite scale to (1, 1).
    pub fn fresh() -> Self {
        Self {
            entries: Vec::new(),
            slots: [EMPTY; SLOTS],
            head_skin: ISAAC_SKIN,
            body_skin: ISAAC_SKIN,
            player_skin: ISAAC_SKIN,
            sprite_scale: [1.0, 1.0],
            layer_scales: [[1.0, 1.0]; SLOTS],
        }
    }

    fn layer(actor: &str, slot: usize) -> Option<i32> {
        content::COSTUME_CONFIGS
            .iter()
            .find(|config| config.actor == actor)?
            .layers
            .iter()
            .find(|(_, name)| *name == PLAYER_LAYERS[slot])
            .map(|(id, _)| *id)
    }

    /// 511060 for one costume config.
    fn register(&mut self, item: u32, config: &content::CostumeConfig, flying: bool) {
        self.entries.retain(|entry| entry.item != item);
        let entry = Entry {
            item,
            actor: config.actor.into(),
            sprite: CostumeSprite::stopped(""),
            overlay: None,
            priority: config.priority,
            is_flying: config.is_flying,
            has_overlay: config.has_overlay,
            has_skin_alt: config.has_skin_alt,
            skin_color: config.skin_color,
            overwrite_color: config.overwrite_color,
            sheet_skin: config.skin_color,
        };
        let (mut head, mut body) = (false, false);
        for slot in 0..SLOTS {
            if Self::layer(config.actor, slot).is_some() {
                if body_group(slot) {
                    body = true;
                } else {
                    head = true;
                }
            }
        }
        self.entries.push(entry.clone());
        if head && body {
            self.entries.push(entry);
        }
        self.assign(flying);
    }

    /// 50EDC0.
    fn assign(&mut self, flying: bool) {
        self.slots = [EMPTY; SLOTS];
        let (mut head_skin, mut body_skin) = (ISAAC_SKIN, ISAAC_SKIN);
        let (mut head_best, mut body_best) = (-1, -1);
        let (mut head_forced, mut body_forced) = (false, false);
        let mut index = 0;
        while index < self.entries.len() {
            let entry = &self.entries[index];
            let (mut base, mut current) = (index, index);
            let (mut count, mut first_group) = (0, false);
            for slot in 0..SLOTS {
                let Some(layer) = Self::layer(&entry.actor, slot) else {
                    continue;
                };
                let priority = entry.priority;
                let group = body_group(slot);
                // 50EF2E and 50EF46: a lower priority, or a non-flying
                // costume's body layer while flying, only feeds the skin.
                let assigned = self.slots[slot].priority <= priority
                    && !(flying && !entry.is_flying && group);
                if assigned {
                    let split = current == base && count != 0 && slot != 0 && group != first_group;
                    let target = if split && group {
                        current += 1;
                        current
                    } else if split {
                        base += 1;
                        base
                    } else if group {
                        current
                    } else {
                        base
                    };
                    self.slots[slot] = Slot {
                        entry: target as i32,
                        layer,
                        priority,
                    };
                    count += 1;
                    first_group = group;
                    if slot == 4 && entry.overwrite_color {
                        head_forced = true;
                        head_skin = entry.skin_color;
                    }
                    if slot == 1 && entry.overwrite_color {
                        body_forced = true;
                        body_skin = entry.skin_color;
                    }
                }
                if !head_forced && slot == 4 && entry.overwrite_color && priority >= head_best {
                    head_best = priority;
                    head_skin = entry.skin_color;
                }
                if !body_forced && slot == 1 && entry.overwrite_color && priority >= body_best {
                    body_best = priority;
                    body_skin = entry.skin_color;
                }
            }
            index = base.max(current) + 1;
        }
        self.head_skin = head_skin;
        self.body_skin = body_skin;
        // 50F1C0: a skin-alt costume reloads its sheet in the slot's colour.
        for slot in 0..SLOTS {
            let Ok(index) = usize::try_from(self.slots[slot].entry) else {
                continue;
            };
            let entry = &mut self.entries[index];
            if !entry.has_skin_alt {
                continue;
            }
            if entry.sheet_skin != head_skin && !body_group(slot) {
                entry.sheet_skin = head_skin;
            } else if entry.sheet_skin != body_skin && body_group(slot) {
                entry.sheet_skin = body_skin;
            }
        }
        // 50F562..50F803: Isaac's own sheet follows the head skin.
        self.player_skin = head_skin;
        // 50F820/50F898: every assigned entry stops at its Down0 pose.
        for slot in 0..SLOTS {
            let Ok(index) = usize::try_from(self.slots[slot].entry) else {
                continue;
            };
            let clip = if body_group(slot) { "WalkDown" } else { "HeadDown" };
            set_frame(&mut self.entries[index], clip, 0);
        }
    }
}

fn set_frame(entry: &mut Entry, clip: &str, frame: i32) {
    if let Some(found) = content::costume_clip(&entry.actor, clip) {
        entry.sprite.set_frame_with_frames(clip, frame, found.frames);
    }
}
fn play(sprite: &mut CostumeSprite, actor: &str, clip: &str) -> bool {
    match content::costume_clip(actor, clip) {
        Some(found) => {
            sprite.play_with_looping(clip, sprite.clip == clip && found.looping);
            true
        }
        None => false,
    }
}

fn direction(direction: crate::Direction) -> &'static str {
    match direction {
        crate::Direction::Left => "Left",
        crate::Direction::Right => "Right",
        crate::Direction::Up => "Up",
        crate::Direction::Down => "Down",
    }
}

fn wardrobe(state: &mut State) -> Option<&mut Wardrobe> {
    state.player_items.as_mut()?.wardrobe.as_mut()
}

/// 52CF00..52CF32 and 511060, called by every 52CC90 grant.
pub(crate) fn on_collectible_added(state: &mut State, id: u32) {
    let flying = state.player.flying == Some(true);
    let variant0 = state
        .player
        .entity_source
        .is_none_or(|source| source.variant == 0);
    let Some(wardrobe) = wardrobe(state) else {
        return;
    };
    for slot in wardrobe.slots {
        if let Ok(index) = usize::try_from(slot.entry) {
            wardrobe.entries[index].sprite.override_active = false;
        }
    }
    match id {
        // 52DDD0: Magic Mushroom scales the whole sprite by 1.25.
        12 => {
            wardrobe.sprite_scale = wardrobe.sprite_scale.map(|value| value * 1.25_f32);
        }
        // 52D408/52D609: Odd Mushroom scales the head-group layers.
        120 | 121 if variant0 => {
            let factor = if id == 121 { 1.1_f32 } else { 0.9_f32 };
            for (slot, scale) in wardrobe.layer_scales.iter_mut().enumerate() {
                if !body_group(slot) {
                    *scale = scale.map(|value| value * factor);
                }
            }
        }
        _ => {}
    }
    // 511074: a co-op baby (variant 1) has no costumes.
    if !variant0 || !crate::items::collectible_add_may_register_costume(id) {
        return;
    }
    if let Some(config) = content::collectible_costume(id) {
        wardrobe.register(id, config, flying);
    }
}

/// 52FAED: each head-group slot's entry follows the Player's head clip
/// unless an item controller holds it (+2EC).
pub(crate) fn sync_head(state: &mut State) {
    let clip = format!("Head{}", direction(state.player.facing));
    let frame = i32::from(state.player.head_frame);
    let Some(wardrobe) = wardrobe(state) else {
        return;
    };
    for slot in 0..SLOTS {
        if body_group(slot) {
            continue;
        }
        let Ok(index) = usize::try_from(wardrobe.slots[slot].entry) else {
            continue;
        };
        let entry = &mut wardrobe.entries[index];
        if entry.sprite.override_active {
            continue;
        }
        let Some(found) = content::costume_clip(&entry.actor, &clip) else {
            continue;
        };
        if found.looping {
            let actor = entry.actor.clone();
            play(&mut entry.sprite, &actor, &clip);
        } else if frame == 0
            && content::costume_clip(&entry.actor, &format!("{clip}_Idle")).is_some()
        {
            let actor = entry.actor.clone();
            play(&mut entry.sprite, &actor, &format!("{clip}_Idle"));
        } else {
            entry
                .sprite
                .set_frame_with_frames(&clip, frame, found.frames);
        }
        if entry.has_overlay {
            let name = format!("{clip}_Overlay");
            if let Some(found) = content::costume_clip(&entry.actor, &name) {
                let overlay = entry
                    .overlay
                    .get_or_insert_with(|| CostumeSprite::stopped(&name));
                if found.looping {
                    overlay.play_with_looping(&name, overlay.clip == name);
                } else {
                    overlay.set_frame_with_frames(&name, frame, found.frames);
                }
            }
        }
    }
}

/// 532621/533A90 then 532090, on the full update: body-group entries play
/// the walk clip while moving (WalkDown_Idle or WalkDown 0 at rest), then
/// every assigned entry advances once.
pub(crate) fn advance_full(state: &mut State) {
    let moving = state
        .pressed
        .iter()
        .any(|key| matches!(key.as_str(), "KeyA" | "KeyD" | "KeyW" | "KeyS"));
    let walk = format!("Walk{}", direction(state.player.body_facing));
    let walk_frame = i32::from(state.player.walk_frame);
    let flying = state.player.flying == Some(true);
    let Some(wardrobe) = wardrobe(state) else {
        return;
    };
    for slot in 0..SLOTS {
        if !body_group(slot) {
            continue;
        }
        let Ok(index) = usize::try_from(wardrobe.slots[slot].entry) else {
            continue;
        };
        let entry = &mut wardrobe.entries[index];
        if entry.sprite.override_active {
            continue;
        }
        let actor = entry.actor.clone();
        if flying {
            play(&mut entry.sprite, &actor, "WalkDown");
        } else if moving {
            // The port's own walk is a frame of the Walk clip, not a played
            // animation; the costume shows the same frame.
            if let Some(found) = content::costume_clip(&actor, &walk) {
                entry
                    .sprite
                    .set_frame_with_frames(&walk, walk_frame, found.frames);
            }
        } else if !play(&mut entry.sprite, &actor, "WalkDown_Idle") {
            set_frame(entry, "WalkDown", 0);
        }
    }
    let mut advanced = vec![false; wardrobe.entries.len()];
    for slot in wardrobe.slots {
        let Ok(index) = usize::try_from(slot.entry) else {
            continue;
        };
        if std::mem::replace(&mut advanced[index], true) {
            continue;
        }
        let entry = &mut wardrobe.entries[index];
        for sprite in std::iter::once(&mut entry.sprite).chain(entry.overlay.as_mut()) {
            if let Some(found) = content::costume_clip(&entry.actor, &sprite.clip) {
                sprite.advance_full_with_clip(found.frames, found.looping);
            }
        }
    }
}

/// 508820: an item's own controller plays `clip` on that item's costume
/// entries in the head (or body) group and holds them (+2EC). A negative
/// frame plays the clip; otherwise it stops on that frame.
pub(crate) fn play_item(state: &mut State, item: u32, body: bool, clip: &str, frame: i32) {
    let Some(wardrobe) = wardrobe(state) else {
        return;
    };
    for slot in 0..SLOTS {
        if body_group(slot) != body {
            continue;
        }
        let Ok(index) = usize::try_from(wardrobe.slots[slot].entry) else {
            continue;
        };
        let entry = &mut wardrobe.entries[index];
        if entry.item != item {
            continue;
        }
        entry.sprite.override_active = true;
        let actor = entry.actor.clone();
        if frame < 0 {
            play(&mut entry.sprite, &actor, clip);
        } else {
            set_frame(entry, clip, frame);
        }
    }
}

/// 50B000: every assigned entry returns to the Player's pose.
pub(crate) fn release(state: &mut State) {
    let Some(wardrobe) = wardrobe(state) else {
        return;
    };
    for slot in wardrobe.slots {
        if let Ok(index) = usize::try_from(slot.entry) {
            wardrobe.entries[index].sprite.override_active = false;
        }
    }
}

/// 531C53..531C7D: an override entry stopped on a non-looping clip returns
/// to the Player's pose.
pub(crate) fn release_finished(state: &mut State) {
    let Some(wardrobe) = wardrobe(state) else {
        return;
    };
    for slot in wardrobe.slots {
        let Ok(index) = usize::try_from(slot.entry) else {
            continue;
        };
        let entry = &mut wardrobe.entries[index];
        let looping = content::costume_clip(&entry.actor, &entry.sprite.clip)
            .is_some_and(|clip| clip.looping);
        if entry.sprite.override_active && !entry.sprite.playing && !looping {
            entry.sprite.override_active = false;
        }
    }
}
