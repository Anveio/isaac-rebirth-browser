//! Actual GridBase seed owners and the supported fresh-room record path.
//! Geometry-only/unsupported objects retain no invented secondary seed.
use crate::{grid::GridEntity, items::Rng, mt::Mt19937, Room};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GridOwner {
    pub secondary_seed: u32,
    /// Opaque native object+14 word; do not infer a lifetime from its value.
    pub source_word14: u32,
    pub initialized: bool,
    pub birth_game_update: Option<u32>,
    pub local_rng: Rng,
}

/// Seven semantic fields in each native 0x1c-byte grid record. Padding beside
/// the initialized byte is not a gameplay owner and is not fabricated here.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GridRecord {
    pub kind: u32,
    pub variant: u32,
    pub state: u32,
    pub source_word14: u32,
    pub primary_seed: u32,
    pub initialized: bool,
    pub secondary_seed: u32,
}

impl GridRecord {
    pub fn empty(shared: &mut Mt19937) -> Self {
        Self {
            kind: 0,
            variant: 0,
            state: 0,
            source_word14: 0,
            primary_seed: shared.next_u32(),
            initialized: false,
            secondary_seed: shared.next_u32(),
        }
    }

    pub fn seed_pair(&self) -> [u32; 2] {
        [self.primary_seed, self.secondary_seed]
    }
}

impl GridEntity {
    /// Base560ff0 stores two real raw words and constructs RNG5/9/7 without a
    /// shared draw. Primary stays in the actual object's existing spawn field.
    fn construct_owner(&mut self, shared: &mut Mt19937) -> Result<(), String> {
        if self.source_owner.is_some() {
            return Err("grid object already has its actual constructor owner".into());
        }
        self.spawn_seed = shared.next_u32();
        self.source_owner = Some(GridOwner {
            secondary_seed: shared.next_u32(),
            source_word14: 0,
            initialized: false,
            birth_game_update: None,
            local_rng: Rng::new(0xaa17_414f, [5, 9, 7]),
        });
        Ok(())
    }

    fn finish_init(&mut self, game_update: u32) -> Result<(), String> {
        let owner = self
            .source_owner
            .as_mut()
            .ok_or("grid Init requires its constructor")?;
        owner.source_word14 = owner.source_word14.wrapping_add(1);
        owner.initialized = true;
        owner.birth_game_update = Some(game_update);
        Ok(())
    }

    /// InitSeed561040 changes only the primary owner. Supported derived Init
    /// callbacks retain their existing modeled collision/animation profile.
    fn init_seed(&mut self, primary: u32, game_update: u32) -> Result<(), String> {
        self.spawn_seed = primary;
        self.finish_init(game_update)
    }

    pub fn record(&self) -> Option<GridRecord> {
        self.source_owner.as_ref().map(|owner| GridRecord {
            kind: self.kind,
            variant: self.variant,
            state: self.state,
            source_word14: owner.source_word14,
            primary_seed: self.spawn_seed,
            initialized: owner.initialized,
            secondary_seed: owner.secondary_seed,
        })
    }

    /// Actual factory construction precedes InitRecord561090. These raw words
    /// create object state before the saved fields overwrite both seeds.
    pub fn init_record(
        &mut self,
        shared: &mut Mt19937,
        record: &GridRecord,
        game_update: u32,
    ) -> Result<(), String> {
        if !matches!(record.kind, 1 | 15 | 16) {
            return Err("record factory kind is outside supported object ownership".into());
        }
        self.construct_owner(shared)?;
        self.kind = record.kind;
        self.variant = record.variant;
        self.state = record.state;
        self.spawn_seed = record.primary_seed;
        let owner = self.source_owner.as_mut().expect("constructed grid owner");
        owner.secondary_seed = record.secondary_seed;
        owner.source_word14 = record.source_word14;
        owner.initialized = record.initialized;
        self.finish_init(game_update)
    }
}

/// Supported fresh normal room admission5cecc0: authored groups first, enabled
/// door slots ascending, top/bottom columns, then left/right rows. Additional
/// room/effect decoration and other derived constructors remain unadmitted.
pub fn admit_fresh(room: &mut Room, shared: &mut Mt19937, game_update: u32) -> Result<(), String> {
    let descriptor = room
        .descriptor
        .as_ref()
        .ok_or("grid admission requires descriptor")?;
    if descriptor.template.room_type == 16 {
        return Err("special room wall replacements are outside fresh grid ownership".into());
    }
    let spawns = room
        .spawns
        .as_ref()
        .ok_or("grid admission requires authored selectors")?;
    let grid = room
        .grid
        .as_mut()
        .ok_or("grid admission requires live geometry")?;
    let width = usize::from(grid.width);
    let height = usize::from(grid.height);
    if width < 3 || height < 3 || width * height != grid.cells.len() {
        return Err("grid admission dimensions disagree with actual cells".into());
    }
    for selector in &spawns.selectors {
        // Source5ce858/5ce8b0 maps raw0 to GridDecoration1 and keeps variant.
        if selector.entity_type != 0 {
            continue;
        }
        let cell = grid
            .cells
            .get_mut(usize::from(selector.grid_index))
            .ok_or("authored decoration outside grid")?;
        if cell
            .entity
            .as_ref()
            .is_some_and(|entity| entity.source_owner.is_some())
        {
            continue;
        }
        let mut entity = GridEntity {
            kind: 1,
            variant: u32::from(selector.variant),
            state: 0,
            collision_class: 0,
            spawn_seed: 0,
            source_owner: None,
        };
        entity.construct_owner(shared)?;
        entity.init_seed(selector.spawn_seed, game_update)?;
        cell.entity = Some(entity);
    }
    for slot in 0..8 {
        if descriptor.door_mask & (1 << slot) == 0 {
            continue;
        }
        let door = room
            .appearance
            .doors
            .iter()
            .find(|door| usize::from(door.slot) == slot)
            .ok_or("enabled source door mask lacks authored position")?;
        let column = ((door.world_position.x - 40.0) / 40.0) as usize;
        let row = ((door.world_position.y - 120.0) / 40.0) as usize;
        let entity = grid
            .cells
            .get_mut(column + row * width)
            .and_then(|cell| cell.entity.as_mut())
            .filter(|entity| entity.kind == 16)
            .ok_or("enabled door lacks actual door geometry")?;
        entity.construct_owner(shared)?;
        // Room5c5722 draws a fresh caller seed after actual construction.
        entity.init_seed(shared.next_u32(), game_update)?;
    }
    for column in 0..width {
        admit_wall(grid, shared, column, game_update)?;
        admit_wall(grid, shared, column + (height - 1) * width, game_update)?;
    }
    for row in 0..height - 1 {
        admit_wall(grid, shared, row * width, game_update)?;
        admit_wall(grid, shared, row * width + width - 1, game_update)?;
    }
    Ok(())
}

fn admit_wall(
    grid: &mut crate::grid::GridDescriptor,
    shared: &mut Mt19937,
    index: usize,
    game_update: u32,
) -> Result<(), String> {
    let entity = grid.cells[index]
        .entity
        .as_mut()
        .ok_or("normal boundary lacks wall geometry")?;
    // A prior actual authored object/door suppresses factory admission. Geometry
    // wall placeholders admit once; top corners then skip on the side pass.
    if entity.kind != 15 || entity.source_owner.is_some() {
        return Ok(());
    }
    entity.construct_owner(shared)?;
    // Room5c55bf overwrites only the constructor primary after its two draws.
    entity.init_seed(shared.next_u32(), game_update)
}
