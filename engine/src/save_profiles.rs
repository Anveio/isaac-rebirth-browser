//! Versioned application save transport over the actual supported Save DTO.
//! It does not serialize Game/State or accept native binary files. Import only
//! installs retained profile records; public Continue owns live constructors.
use crate::{mt, saved_game, Profile, State};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const VERSION: u32 = 2;
pub const MAX_BYTES: usize = 4 * 1024 * 1024;
///The existing string-only save ABI reports this expected capacity refusal.
///Hosts acknowledge it as a failed write; other export errors stay errors.
pub const CAPACITY_ERROR: &str = "saved profiles exceed the supported 4 MiB format";

///Commands clone Game for atomic validation. Retained profile records are
///immutable between actual Save/Clear requests, so that hot path shares them.
pub mod shared {
    use super::Profiles;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::sync::Arc;
    pub fn serialize<S: Serializer>(
        value: &Option<Arc<Profiles>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        value.as_deref().serialize(serializer)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<Arc<Profiles>>, D::Error> {
        Option::<Profiles>::deserialize(deserializer).map(|value| value.map(Arc::new))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Profiles {
    pub version: u32,
    pub profiles: Vec<ProfileRecord>,
}

///Own V1 has explicit179/95 arrays and no collected history. Strict DTOs
///prevent V2 omissions or foreign unknown fields from becoming a reset.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LegacyProfiles {
    version: u32,
    profiles: Vec<LegacyProfileRecord>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LegacyProfileRecord {
    profile: LegacyProfile,
    saved: Option<RunRecord>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LegacyProfile {
    slot: u8,
    unlocked_characters: Vec<String>,
    selected_character: String,
    achievements: Vec<bool>,
    counters: Vec<i32>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProfileRecord {
    pub profile: Profile,
    pub saved: Option<RunRecord>,
}

///Which Continue load caller admits a saved run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContinueDomain {
    ///The fresh starting room with neutral items and no saved actors.
    InitialRoom,
    ///Any other living normal Isaac save: the current room resumes as a
    ///revisited Room with its saved persistent records.
    VisitedRoom,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RunRecord {
    pub run: saved_game::SavedRun,
    pub seeds: mt::SavedGameSeeds,
    pub summary: saved_game::SavedRunSummary,
    pub context: RunContext,
}

///Level/Game stores omitted by the original diagnostic-only summary. Every
///field is copied from the actual save boundary, never inferred at load.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RunContext {
    pub character: String,
    /// Game+10ECD8, saved at 54f294 and restored at 54ed4d.
    pub difficulty: crate::difficulty::Difficulty,
    pub challenge_id: u32,
    pub player_type: Option<u32>,
    pub source_effect_mode: Option<u32>,
    pub run_seed_value: u32,
    pub stage_seed: u32,
    pub level_entry: Option<RoomEntry>,
    pub level_progress: crate::level_progress::LevelProgress,
    pub player_manager_rng: Option<crate::items::Rng>,
    ///Actual listed Bomb4 bodies at Save. Legacy absence is unknown; this
    ///first load caller has no saved Bomb body reconstruction owner.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub listed_bomb_actor_count: Option<usize>,
    ///Room.Save's live Bomb4 records. Legacy absence restores no bombs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bomb_records: Option<Vec<crate::bombs::SavedBombRecord>>,
    pub visited_rooms: BTreeSet<usize>,
    pub room_display_flags: Vec<u32>,
    pub revealed_secret_connections: BTreeSet<[usize; 2]>,
    pub persisted_rooms: BTreeMap<usize, crate::Room>,
    ///Actual Room.Save actor records; missing legacy ownership stays unknown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ambient_records: Option<Vec<crate::critters::SavedAmbientRecord>>,
}

///Actual Level818C/8198/819C stores at a supported admission boundary.
///Unknown previous/entry writers cannot acquire these facts from current P.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RoomEntry {
    pub previous_room: i32,
    pub slot: i32,
    pub position: crate::motion::WorldVector,
}

impl RoomEntry {
    ///58BEF2 initializes8198=-1;58C34B/351 copies the actual start index
    ///to8188/818C.58BF1C/24 loads Source280/320 into the separate
    ///Level819C placement vector; this is not the current Player position.
    pub fn initial(index: usize) -> Result<Self, String> {
        Ok(Self {
            previous_room: i32::try_from(index)
                .map_err(|_| "room grid cell exceeds source range")?,
            slot: -1,
            position: crate::motion::WorldVector { x: 320.0, y: 280.0 },
        })
    }
}

impl RunContext {
    pub fn capture(
        state: &State,
        ambient_records: Option<Vec<crate::critters::SavedAmbientRecord>>,
    ) -> Self {
        Self {
            character: state.character.clone(),
            difficulty: state.difficulty,
            challenge_id: state.challenge_id,
            player_type: state.weapon_admission.source_player_type,
            source_effect_mode: state
                .run_seeds
                .as_ref()
                .and_then(|seeds| seeds.source_effect_mode),
            run_seed_value: state.run_seed_value,
            stage_seed: state.stage_seed,
            level_entry: state.saved_room_entry.clone(),
            level_progress: state.level_progress.clone(),
            player_manager_rng: state.player_items.as_ref().map(|owner| owner.manager_rng),
            listed_bomb_actor_count: Some(state.bombs.len()),
            bomb_records: Some(crate::bombs::capture_saved_records(state)),
            visited_rooms: state.visited_rooms.clone(),
            room_display_flags: state.room_display_flags.clone(),
            revealed_secret_connections: state.revealed_secret_connections.clone(),
            persisted_rooms: state.persisted_rooms.clone(),
            ambient_records,
        }
    }
}

impl Profiles {
    pub fn fresh(profiles: &[Profile]) -> Self {
        Self {
            version: VERSION,
            profiles: profiles
                .iter()
                .cloned()
                .map(|profile| ProfileRecord {
                    profile,
                    saved: None,
                })
                .collect(),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > MAX_BYTES {
            return Err(CAPACITY_ERROR.into());
        }
        let header: serde_json::Value =
            serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
        let version = header
            .get("version")
            .and_then(serde_json::Value::as_u64)
            .ok_or("own profile version is absent")?;
        let value = match version {
            1 => {
                let old: LegacyProfiles =
                    serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
                if old.version != 1 {
                    return Err("invalid own V1 profile version".into());
                }
                Self {
                    version: VERSION,
                    profiles: old
                        .profiles
                        .into_iter()
                        .map(|record| ProfileRecord {
                            profile: Profile {
                                slot: record.profile.slot,
                                unlocked_characters: record.profile.unlocked_characters,
                                selected_character: record.profile.selected_character,
                                achievements: record.profile.achievements,
                                counters: record.profile.counters,
                                collected_history:
                                    crate::persistent::CollectedHistory::legacy_own_v1(),
                            },
                            saved: record.saved,
                        })
                        .collect(),
                }
            }
            2 => serde_json::from_slice(bytes).map_err(|error| error.to_string())?,
            _ => return Err("unsupported own profile version; preserve original document".into()),
        };
        value.validate()?;
        Ok(value)
    }

    ///Serialize every retained owner under the same byte budget as import.
    ///Capacity is an expected failed write, not an invalid simulation state.
    pub fn write_packet(&self) -> Result<SaveWritePacket, String> {
        self.validate()?;
        let mut output = SavePacketWriter::default();
        match serde_json::to_writer(&mut output, self) {
            Ok(()) => Ok(SaveWritePacket::Ready(std::sync::Arc::new(output.bytes))),
            Err(_) if output.capacity_exceeded => Ok(SaveWritePacket::CapacityExceeded),
            Err(error) => Err(error.to_string()),
        }
    }

    ///Validate the complete retained document before changing the live Game.
    ///Unknown versions and absent fields do not become a fresh default save.
    pub fn validate(&self) -> Result<(), String> {
        if self.version != VERSION || self.profiles.len() != 3 {
            return Err("saved profiles require version2 and exactly three slots".into());
        }
        for (index, record) in self.profiles.iter().enumerate() {
            let profile = &record.profile;
            profile.collected_history.validate()?;
            if usize::from(profile.slot) != index + 1
                || profile.achievements.len() != 179
                || profile.counters.len() != 95
                || profile.unlocked_characters.is_empty()
                || profile.unlocked_characters.len() > 10
                || profile.selected_character.len() > 32
                || !profile.unlocked_characters.iter().all(|name| {
                    crate::character_catalog()
                        .iter()
                        .any(|definition| &definition.name == name)
                })
                || !crate::character_catalog()
                    .iter()
                    .any(|definition| definition.name == profile.selected_character)
            {
                return Err(
                    "saved profile contains an unsupported slot or incomplete owners".into(),
                );
            }
            if let Some(run) = &record.saved {
                run.validate()?;
            }
        }
        Ok(())
    }
}

impl RunRecord {
    /// The record format's supported per-room actor counts.
    pub(crate) fn rooms_within_supported_counts(&self) -> bool {
        std::iter::once(&self.run.room)
            .chain(self.context.persisted_rooms.values())
            .all(|room| room.npcs.len() <= 512 && room.pickups.len() <= 512)
    }

    pub fn validate(&self) -> Result<(), String> {
        let run = &self.run;
        let descriptors = &run.floor.descriptors;
        let context = &self.context;
        if context.run_seed_value == 0
            || crate::seed::decode(&run.run_seed)? != context.run_seed_value
            || descriptors.is_empty()
            || descriptors.len() > 127
            || run.floor.starting_descriptor >= descriptors.len()
            || run.current_room >= descriptors.len()
            || context.room_display_flags.len() != descriptors.len()
            || context
                .visited_rooms
                .iter()
                .any(|index| *index >= descriptors.len())
            || context
                .persisted_rooms
                .keys()
                .any(|index| *index >= descriptors.len())
            || context
                .revealed_secret_connections
                .iter()
                .any(|[left, right]| *left >= descriptors.len() || *right >= descriptors.len())
            || run.stage != run.floor.stage
            || run.floor.stage_seed != context.stage_seed
            || run.floor.grid.len() > 169
            || run
                .floor
                .grid
                .iter()
                .any(|index| *index < -1 || *index >= descriptors.len() as i32)
            || run.grid_records.len()
                != run
                    .room
                    .grid
                    .as_ref()
                    .ok_or("saved room has no grid")?
                    .cells
                    .len()
            || self.summary.run_seed != run.run_seed
            || self.summary.stage != run.stage
            || self.summary.current_room_record != run.current_room
            || self.summary.game_update_count != run.game_update_count
        {
            return Err("saved run has incomplete or inconsistent Level/Game owners".into());
        }
        run.item_pools.validate_saved_trinkets()?;
        if let Some(records) = &context.ambient_records {
            crate::critters::validate_saved_records(records)?;
        }
        for (index, descriptor) in descriptors.iter().enumerate() {
            if descriptor.index != index
                || descriptor
                    .neighbors
                    .iter()
                    .flatten()
                    .any(|neighbor| *neighbor >= descriptors.len())
                || descriptor.template.width == 0
                || descriptor.template.height == 0
            {
                return Err("saved Level contains an invalid descriptor".into());
            }
        }
        //Room paint is append-only. The complete document byte limit, not
        //an unrelated stain count, bounds this retained transport owner.
        for room in std::iter::once(&run.room).chain(context.persisted_rooms.values()) {
            let grid = room
                .grid
                .as_ref()
                .ok_or("saved room is missing its actual grid")?;
            if grid.cells.len() > crate::grid::MAX_GRID_CELLS
                || usize::from(grid.width) * usize::from(grid.height) != grid.cells.len()
            {
                return Err("saved room is outside supported bounds".into());
            }
        }
        if !self.rooms_within_supported_counts() {
            return Err("saved room is outside supported bounds".into());
        }
        if run.player.max_hp > 24
            || !(0..=24).contains(&run.player.soul_hp)
            || run.player.eternal_hearts < 0
            || run.player.eternal_hearts > 1
            || !run.player.x.is_finite()
            || !run.player.y.is_finite()
            || !run.player.vx.is_finite()
            || !run.player.vy.is_finite()
            || run.player.hp > run.player.max_hp
            || run.inventory.coins > 99
            || run.inventory.keys > 99
            || run.inventory.bombs > 99
            || run.player_items.owned_counts.len() != 347
            || run.player_items.item_rng_states.len() != 347
            || run.player_items.trinket_rng_states.len() != 62
            || run.player_items.pocket_rng_states.len() != 41
        {
            return Err("saved run has incomplete Player owners".into());
        }
        Ok(())
    }

    ///Selects the load caller for Main Continue. Every record a living run
    ///saved loads (docs/save-system.md): the initial-room caller keeps its
    ///constructor-exact reconstruction of an untouched start room, and every
    ///other save resumes through the visited-room caller, which follows the
    ///original Room.Init revisit: Room.Save5ce2d0 kept only the persistent
    ///families (5c57c0: bombs, pickups, slots, shopkeepers, fireplaces and
    ///listed effects), so an uncleared room rebuilds its NPCs from the layout.
    ///Owners the live run itself tracked as unknown (costume registrations,
    ///transformation counters, the golden key) are restored as unknown, the
    ///state the run was in when it was saved. Only corrupt or inconsistent
    ///records and lost runs are refused.
    pub fn admit_continue(&self) -> Result<ContinueDomain, String> {
        if self.admit_initial_room().is_ok() {
            return Ok(ContinueDomain::InitialRoom);
        }
        self.validate()?;
        let run = &self.run;
        let context = &self.context;
        if run.player.dead || (run.player.hp == 0 && run.player.soul_hp <= 0) {
            return Err("a lost run has no Continue".into());
        }
        if context.character != "Isaac" || context.player_type.is_some_and(|kind| kind != 0) {
            return Err("saved run names a character this build cannot construct".into());
        }
        if context
            .player_manager_rng
            .is_none_or(|rng| rng.state == 0 || [rng.right_a, rng.left, rng.right_b] != [5, 7, 22])
        {
            return Err("saved run is missing its Player manager RNG".into());
        }
        let entry = context
            .level_entry
            .as_ref()
            .ok_or("saved run is missing its Level entry")?;
        if entry.slot >= 8 {
            return Err("saved Level entry door slot is outside the eight Level slots".into());
        }
        if let Some(records) = &context.ambient_records {
            crate::critters::validate_saved_records(records)?;
        }
        if let Some(records) = &context.bomb_records {
            crate::bombs::validate_saved_records(records)?;
        }
        if !context.visited_rooms.contains(&run.current_room) {
            return Err("saved current room was never entered".into());
        }
        Ok(ContinueDomain::VisitedRoom)
    }

    ///This first load caller reconstructs the complete supported initial room.
    ///This is a source-domain boundary, never a seed or witness allowlist.
    pub fn admit_initial_room(&self) -> Result<(), String> {
        self.validate()?;
        let run = &self.run;
        let context = &self.context;
        let descriptor = &run.floor.descriptors[run.current_room];
        if context.character != "Isaac"
            || context.challenge_id != 0
            || context.player_type != Some(0)
            || context.source_effect_mode != Some(0)
            || run.stage != 1
            || run.current_room != run.floor.starting_descriptor
            || context.visited_rooms != BTreeSet::from([run.current_room])
            || !context.persisted_rooms.is_empty()
            || !context.revealed_secret_connections.is_empty()
            || !run.room.npcs.is_empty()
            || !run.room.pickups.is_empty()
            || context.listed_bomb_actor_count != Some(0)
            || run.player.dead
            || run.player.hp == 0
            || run.player.golden_key.is_none()
            || run.player.transformation_counts != Some([0, 0])
            //508540/5083B0 high-Max/Soul achievement1/80 callers are separate.
            || ((run.player.max_hp > 13 || run.player.soul_hp > 7) && !run.seeded_run)
            || run.room.trapdoor.is_some()
            || !run.room.bomb_rock_fuses.is_empty()
            || !run.room.saved_regular_pickups.is_empty()
            || run
                .room
                .dynamic_crater_records
                .as_ref()
                .is_none_or(|records| !records.is_empty())
            //Flags8 Blood7 paint, including generic splats and footprints, shares
            //a transient backdrop target. Room.Init
            //5C5B30→42A170→428020 clears that same target before rebuilding it;
            //5C57C0 also excludesBlood7 from persisted entity records. The
            //diagnostic savedRoom still retains its real preSave paint, while
            //Continue constructs a fresh Room and never copies those stains.
            || run
                .room
                .background_stains
                .iter()
                .any(|stain| stain.variant != 7)
        {
            return Err("Continue has no owned load caller for this saved run yet".into());
        }
        if context
            .player_manager_rng
            .is_none_or(|rng| rng.state == 0 || [rng.right_a, rng.left, rng.right_b] != [5, 7, 22])
        {
            return Err(
                "Continue requires actual saved Player constructor/manager RNG stores".into(),
            );
        }
        let entry = context
            .level_entry
            .as_ref()
            .ok_or("Continue has no owned saved Level entry stores")?;
        //The admitted initial entry must match the same Source constructor;
        //a corrected owned initial vector must not retain a stale zero guard.
        if entry != &RoomEntry::initial(descriptor.grid_index)? {
            return Err("Continue entry requires its supported initial Level owner".into());
        }
        crate::critters::validate_saved_records(context.ambient_records.as_deref().ok_or(
            "Continue requires owned saved ambient records; missing legacy data is unknown",
        )?)?;
        //This caller has no other saved actor/familiar reconstruction owner yet.
        //Its fully neutral item domain is established by actual saved stores,
        //not by the destination constructor's apparent current loadout.
        let items = &run.player_items;
        if items.owned_counts.iter().any(|count| *count != 0)
            || items.active_id != 0
            || items.trinket_slots != Some([0, 0])
            || items
                .costume_registrations
                .as_ref()
                .is_none_or(|history| !history.configuration_ids.is_empty())
            || items
                .held_pockets
                .as_ref()
                .is_none_or(|held| held.slots.iter().any(|record| record.id != 0))
        {
            return Err(
                "Continue item callbacks require a separately admitted saved grant domain".into(),
            );
        }
        let descriptor = &run.floor.descriptors[run.current_room];
        if descriptor.template.room_type != 1
            || descriptor.template.shape_index() != 0
            || descriptor
                .persistent_spawn_cells
                .as_ref()
                .is_none_or(|cells| !cells.present.is_empty() || !cells.unknown.is_empty())
            || run.room.spawns.as_ref().is_none_or(|spawns| {
                spawns
                    .selectors
                    .iter()
                    .any(|selector| selector.category == crate::room_spawns::SpawnCategory::Entity)
            })
        {
            return Err("Continue requires the supported normal initial Room".into());
        }
        for record in run.grid_records.iter().flatten() {
            if !matches!(record.kind, 0 | 1 | 12 | 14 | 15 | 16) {
                return Err("Continue grid record lacks its actual factory/Init owner".into());
            }
        }
        if run.grid_records.iter().any(Option::is_none) {
            return Err("Continue cannot reconstruct an unowned occupied GridRecord".into());
        }
        Ok(())
    }
}

///Application IO suspends the current public Tick before System.advance.
///The reply token identifies the request, not the revision of stored bytes:
///a failed write can legitimately reload an older successful own save.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MenuReadmission {
    pub protocol: u32,
    pub request_revision: u64,
    pub profile: u8,
    pub remaining_ticks: u32,
    pub timestamp_millis: Option<u64>,
}

///The packet captured by one Save/Clear. An oversized document remains in
///the retained owners, while its attempted write exports no truncated bytes.
#[derive(Clone, Debug, PartialEq)]
pub enum SaveWritePacket {
    Ready(std::sync::Arc<Vec<u8>>),
    CapacityExceeded,
}

impl SaveWritePacket {
    pub fn export(&self) -> Result<Vec<u8>, String> {
        match self {
            Self::Ready(bytes) => Ok(bytes.as_ref().clone()),
            Self::CapacityExceeded => Err(CAPACITY_ERROR.into()),
        }
    }
}

///Never allocate an oversized export before discovering its byte length.
#[derive(Default)]
struct SavePacketWriter {
    bytes: Vec<u8>,
    capacity_exceeded: bool,
}

impl std::io::Write for SavePacketWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > MAX_BYTES - self.bytes.len() {
            self.capacity_exceeded = true;
            return Err(std::io::Error::other(CAPACITY_ERROR));
        }
        let required = self.bytes.len() + bytes.len();
        if required > self.bytes.capacity() {
            let capacity = required.next_power_of_two().min(MAX_BYTES);
            self.bytes.reserve_exact(capacity - self.bytes.len());
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

///An actual host-enabled Save/Clear owns one immutable request packet.
///The public Tick may resume only after its real write attempt is acknowledged.
#[derive(Clone, Debug, PartialEq)]
pub struct SaveWriteRequest {
    pub protocol: u32,
    pub revision: u64,
    pub packet: SaveWritePacket,
    pub remaining_ticks: Option<u32>,
}
