//! One owner for Player1388..13F4, indexed by the final Pill effect ID.
//! The old Move/Luck wire words are derived views, not mutable counter stores.
use crate::player_items::{LuckInputs, MoveSpeedInputs, PlayerItems};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub(crate) const WIRE_VERSION: u8 = 1;
pub const EFFECTS: usize = 28;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PillEffectHistory {
    pub words: [Option<i32>; EFFECTS],
}

impl PillEffectHistory {
    /// Actual reached normal519798..5197AB initializes all28 words to zero.
    pub(crate) const FRESH: Self = Self {
        words: [Some(0); EFFECTS],
    };
    pub(crate) const UNKNOWN: Self = Self {
        words: [None; EFFECTS],
    };

    pub(crate) fn word(&self, effect: usize) -> Option<i32> {
        self.words.get(effect).copied().flatten()
    }

    ///5284A3 increments the final resolved effect, before5284AC dispatch.
    ///Unknown prior words remain unknown; this never creates a new owner.
    pub(crate) fn increment_resolved(&mut self, effect: u32) -> Result<(), String> {
        let word = self
            .words
            .get_mut(effect as usize)
            .ok_or("Resolved Pill effect is outside the actual28-word block")?;
        if let Some(value) = word {
            *value = value.wrapping_add(1);
        }
        Ok(())
    }

    pub(crate) fn forget(&mut self) {
        self.words.fill(None);
    }
}

/// Read one independently owned raw counter. Source stat callbacks consume
/// this same word; held slots and unrelated query domains are separate owners.
pub(crate) fn word(owner: &PlayerItems, effect: usize) -> Option<i32> {
    owner.pill_effect_history.as_ref()?.word(effect)
}

pub(crate) fn callback_word(owner: &PlayerItems, effect: usize) -> Option<i32> {
    word(owner, effect)
}

pub(crate) fn forget(owner: &mut PlayerItems) {
    if let Some(history) = &mut owner.pill_effect_history {
        history.forget();
    }
}

/// A reached canonical source copy moves each Some/None unchanged. It never
///uses constructor defaults, current item counts or held Pill color as input.
pub(crate) fn copy_words(source: Option<&PillEffectHistory>) -> Option<PillEffectHistory> {
    source.copied()
}

///581970 resolves the pool's current color assignment before5284A3's write.
///Positive75/303's conversion table is a separate unimplemented operator;
///an unavailable reached health predicate also leaves the final ID unknown.
///Neither case guesses an effect from the color or the unmodified pool entry.
pub(crate) fn resolve_held_pill(state: &crate::State) -> Option<u32> {
    if !state
        .player
        .entity_source
        .is_some_and(|source| source.entity_type == 1 && source.variant == 0)
    {
        return None;
    }
    let owner = state.player_items.as_ref()?;
    let held = owner
        .held_pockets
        .as_ref()
        .filter(|held| held.frontier.is_none())?;
    let slot = held.slots[0];
    if slot.card || slot.id == 0 {
        return None;
    }
    let assigned = *state
        .item_pools
        .as_ref()?
        .pill_effects
        .get(slot.id as usize)?;
    let mut effect = u32::try_from(assigned)
        .ok()
        .filter(|id| *id < EFFECTS as u32)?;
    let queries = crate::item_queries::AppearanceQueries::new(
        owner,
        state.weapon_cache.primary,
        state.player.entity_source,
        None,
    );
    //Have75 true bypasses Have303. A positive branch has a real remapping
    //operator, so it cannot reuse the unmodified color assignment.
    if queries.have(75)? || queries.have(303)? {
        return None;
    }
    //581ADA reads actual max-red-HP for HealthDown6. The current
    //Damage operand history certifies red/soul only, so this branch stays
    //unresolved until the independent maxHP owner joins.
    if effect == 6 {
        return None;
    }
    if effect == 1 {
        owner
            .query_history
            .as_ref()
            .filter(|history| history.damage_health_known)?;
        if i32::from(state.player.hp).wrapping_add(state.player.soul_hp) <= 2 {
            effect = 5;
        }
    }
    Some(effect)
}

/// Existing JSON names are retained as source-word views. Only non-Pill D28
///and D44 live in MoveSpeedInputs after migration.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MoveSpeedView {
    pub source_d28: i32,
    pub source_d44: i32,
    pub source_13bc: i32,
    pub source_13c0: i32,
}

impl MoveSpeedView {
    pub(crate) fn from_owner(owner: &PlayerItems) -> Option<Self> {
        let terms = owner.move_speed_inputs?;
        Some(Self {
            source_d28: terms.source_d28,
            source_d44: terms.source_d44,
            source_13bc: word(owner, 13)?,
            source_13c0: word(owner, 14)?,
        })
    }

    pub(crate) fn terms(self) -> MoveSpeedInputs {
        MoveSpeedInputs {
            source_d28: self.source_d28,
            source_d44: self.source_d44,
        }
    }
}

/// Existing Luck JSON remains a derived view of D3C/D40/D58 plus the one
///canonical17/18 pair. It is not another mutable Pill history.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LuckView {
    pub source_d3c: i32,
    pub source_d40: i32,
    pub source_d58: f32,
    pub source_13cc: i32,
    pub source_13d0: i32,
}

impl LuckView {
    fn from_owner(owner: &PlayerItems) -> Option<Self> {
        let terms = owner.luck_inputs?;
        Some(Self {
            source_d3c: terms.source_d3c,
            source_d40: terms.source_d40,
            source_d58: terms.source_d58,
            source_13cc: word(owner, 17)?,
            source_13d0: word(owner, 18)?,
        })
    }

    fn terms(self) -> LuckInputs {
        LuckInputs {
            source_d3c: self.source_d3c,
            source_d40: self.source_d40,
            source_d58: self.source_d58,
        }
    }
}

fn legacy_words(
    movement: Option<MoveSpeedView>,
    luck: Option<LuckView>,
) -> Option<PillEffectHistory> {
    if movement.is_none() && luck.is_none() {
        return None;
    }
    let mut history = PillEffectHistory::UNKNOWN;
    if let Some(known) = movement {
        history.words[13] = Some(known.source_13bc);
        history.words[14] = Some(known.source_13c0);
    }
    if let Some(known) = luck {
        history.words[17] = Some(known.source_13cc);
        history.words[18] = Some(known.source_13d0);
    }
    Some(history)
}

///Missing old data is distinguishable from an explicit current unknown
///owner. LegacyMissing is only a default for an omitted field; it is not a
///deserializable JSON tag or a way to refill a current missing word.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum SavedPillEffectHistory {
    #[default]
    #[serde(skip)]
    LegacyMissing,
    Current {
        version: u8,
        history: Option<PillEffectHistory>,
    },
}

impl SavedPillEffectHistory {
    pub(crate) fn capture(owner: &PlayerItems) -> Self {
        Self::Current {
            version: WIRE_VERSION,
            history: copy_words(owner.pill_effect_history.as_ref()),
        }
    }

    pub(crate) fn is_legacy_missing(&self) -> bool {
        matches!(self, Self::LegacyMissing)
    }

    pub(crate) fn restore(
        self,
        movement: Option<MoveSpeedView>,
        luck: Option<LuckView>,
    ) -> Result<Option<PillEffectHistory>, String> {
        match self {
            Self::LegacyMissing => Ok(legacy_words(movement, luck)),
            Self::Current {
                version: WIRE_VERSION,
                history,
            } => Ok(history),
            Self::Current { .. } => Err("Unknown Pill-effect history wire version".into()),
        }
    }
}

/// Borrowed State wire projection: no clone of PlayerItems or its RNG/count
///vectors, and no second mutable counter data. Its canonical non-Pill fields
///serialize as moveSpeedTerms/luckTerms; old names are derived below.
pub(crate) fn serialize_player_items<S>(
    owner: &Option<PlayerItems>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct View<'a> {
        #[serde(flatten)]
        owner: &'a PlayerItems,
        pill_effect_history: SavedPillEffectHistory,
        luck_inputs: Option<LuckView>,
        #[serde(skip_serializing_if = "Option::is_none")]
        move_speed_inputs: Option<MoveSpeedView>,
    }
    match owner {
        None => serializer.serialize_none(),
        Some(owner) => View {
            owner,
            pill_effect_history: SavedPillEffectHistory::capture(owner),
            luck_inputs: LuckView::from_owner(owner),
            move_speed_inputs: MoveSpeedView::from_owner(owner),
        }
        .serialize(serializer),
    }
}

///Legacy State imports seed only their actual copied partial views. New
///snapshots retain their independent terms even when all Pill words are None.
pub(crate) fn deserialize_player_items<'de, D>(
    deserializer: D,
) -> Result<Option<PlayerItems>, D::Error>
where
    D: Deserializer<'de>,
{
    //State imports are JSON. Move each actual field once; the remaining
    //object still goes through PlayerItems' deny_unknown_fields decoder.
    //No full PlayerItems clone or second mutable counter store is created.
    let Some(mut fields) =
        Option::<serde_json::Map<String, serde_json::Value>>::deserialize(deserializer)?
    else {
        return Ok(None);
    };
    let movement: Option<MoveSpeedView> = serde_json::from_value(
        fields
            .remove("moveSpeedInputs")
            .unwrap_or(serde_json::Value::Null),
    )
    .map_err(serde::de::Error::custom)?;
    let luck: Option<LuckView> = serde_json::from_value(
        fields
            .remove("luckInputs")
            .unwrap_or(serde_json::Value::Null),
    )
    .map_err(serde::de::Error::custom)?;
    let saved = match fields.remove("pillEffectHistory") {
        Some(value) => serde_json::from_value::<SavedPillEffectHistory>(value)
            .map_err(serde::de::Error::custom)?,
        None => SavedPillEffectHistory::LegacyMissing,
    };
    let legacy = saved.is_legacy_missing();
    if legacy && (fields.contains_key("moveSpeedTerms") || fields.contains_key("luckTerms")) {
        return Err(serde::de::Error::custom(
            "Current PlayerItems terms require the explicit Pill history field",
        ));
    }
    let mut owner: PlayerItems = serde_json::from_value(serde_json::Value::Object(fields))
        .map_err(serde::de::Error::custom)?;
    owner.pill_effect_history = saved
        .restore(movement, luck)
        .map_err(serde::de::Error::custom)?;
    if legacy {
        //This is an explicit legacy DTO conversion, not a fallback for a
        //new missing term/word or an inventory-derived constructor default.
        owner.move_speed_inputs = movement.map(MoveSpeedView::terms);
        owner.luck_inputs = luck.map(LuckView::terms);
    }
    Ok(Some(owner))
}
