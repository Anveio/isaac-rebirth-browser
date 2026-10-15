//! Actual GridBase seed owners and the supported fresh-room record path.
//! Geometry-only/unsupported objects retain no invented secondary seed.
use crate::{grid::GridEntity, items::Rng, mt::Mt19937, room_costs::RoomCosts, Room};
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

    fn init_supported(&mut self, costs: &mut RoomCosts, index: usize) -> Result<(), String> {
        match self.kind {
            15 => {
                // Wall.Init5717e0 writes class4 and actual path cost1000.
                self.collision_class = 4;
                costs.store(index, 1000)?;
            }
            16 => {
                // Door.Init5628e0 maps state0/2 to1, then stores1000. These
                // actual stores are separate from later door animation updates.
                if matches!(self.state, 0 | 2) {
                    self.state = 1;
                }
                costs.store(index, 1000)?;
            }
            1 => {} // Supported Decoration.Init has no direct path-cost store.
            14 if self.variant == 0 && self.state <= 3 => {
                // Poop.Init569600: an ordinary unbroken poop blocks (class3)
                // and stores path cost1500. Its local RNG5/9/7 owns the
                // cosmetic +314 draw.
                self.collision_class = 3;
                costs.store(index, 1500)?;
                let owner = self
                    .source_owner
                    .as_mut()
                    .ok_or("Poop Init requires its constructor")?;
                owner.local_rng = Rng::new(self.spawn_seed, [5, 9, 7]);
                owner.local_rng.bounded(2);
            }
            // Rock (2/5/6) and Spike (8) Init stores are owned by the
            // recovered room geometry and path costs.
            2 | 5 | 6 | 8 => {}
            _ => return Err("derived grid Init is outside supported ownership".into()),
        }
        Ok(())
    }

    /// InitSeed561040 changes only primary, invokes the actual supported
    /// derived stores, then completes its initialized word and birth counter.
    fn init_seed(
        &mut self,
        primary: u32,
        game_update: u32,
        costs: &mut RoomCosts,
        index: usize,
    ) -> Result<(), String> {
        self.spawn_seed = primary;
        self.init_supported(costs, index)?;
        self.finish_init(game_update)
    }

    /// Room5c5070 runtime spawn: refused when the cell's path cost exceeds
    /// 900; otherwise the factory constructor draws its two raw words and
    /// InitSeed runs the derived Init. The caller replaces the cell's entity.
    pub(crate) fn spawn(
        kind: u32,
        variant: u32,
        seed: u32,
        shared: &mut Mt19937,
        game_update: u32,
        costs: &mut RoomCosts,
        index: usize,
    ) -> Result<Option<Self>, String> {
        if costs.get(index)? > 900 {
            return Ok(None);
        }
        let mut entity = Self {
            kind,
            variant,
            state: 0,
            collision_class: 0,
            spawn_seed: 0,
            source_owner: None,
        };
        entity.construct_owner(shared)?;
        entity.init_seed(seed, game_update, costs, index)?;
        Ok(Some(entity))
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
        costs: &mut RoomCosts,
        index: usize,
    ) -> Result<(), String> {
        if !matches!(record.kind, 1 | 14 | 15 | 16) {
            return Err("record factory kind is outside supported object ownership".into());
        }
        costs.get(index)?;
        self.construct_owner(shared)?;
        self.kind = record.kind;
        self.variant = record.variant;
        self.state = record.state;
        self.spawn_seed = record.primary_seed;
        let owner = self.source_owner.as_mut().expect("constructed grid owner");
        owner.secondary_seed = record.secondary_seed;
        owner.source_word14 = record.source_word14;
        owner.initialized = record.initialized;
        self.init_supported(costs, index)?;
        self.finish_init(game_update)
    }
}

/// Raw grid types whose Room.Init factory builds a GridBase object, so its
/// constructor 560ff0 draws two raw words. Raw1400 Fire becomes an NPC.
pub fn constructs_grid_object(entity_type: u16) -> bool {
    matches!(
        entity_type,
        0 | 1000..=1002 | 1300 | 1490..=1500 | 1900 | 1930 | 1931 | 1940 | 3000 | 4000 | 4500
    )
}

/// One authored grid selector of the 5cecc0 selector pass. Decorations and
/// the rocks/spikes recovered by room geometry own their constructor seeds;
/// other GridBase objects keep the two constructor draws without an owner.
pub fn admit_selector(
    room: &mut Room,
    shared: &mut Mt19937,
    selector: &crate::room_spawns::SpawnSelector,
    game_update: u32,
) -> Result<(), String> {
    if selector.category != crate::room_spawns::SpawnCategory::Grid
        || !constructs_grid_object(selector.entity_type)
    {
        return Ok(());
    }
    let grid = room
        .grid
        .as_mut()
        .ok_or("grid admission requires live geometry")?;
    let costs = room
        .path_costs
        .as_mut()
        .expect("source Room.Init path costs");
    let index = usize::from(selector.grid_index);
    let cell = grid
        .cells
        .get_mut(index)
        .ok_or("authored grid selector outside grid")?;
    if cell
        .entity
        .as_ref()
        .is_some_and(|entity| entity.source_owner.is_some())
    {
        return Ok(());
    }
    if selector.entity_type == 0 {
        // Source5ce858/5ce8b0 maps raw0 to GridDecoration1 and keeps variant.
        let mut entity = GridEntity {
            kind: 1,
            variant: u32::from(selector.variant),
            state: 0,
            collision_class: 0,
            spawn_seed: 0,
            source_owner: None,
        };
        entity.construct_owner(shared)?;
        entity.init_seed(selector.spawn_seed, game_update, costs, index)?;
        cell.entity = Some(entity);
        return Ok(());
    }
    match cell.entity.as_mut() {
        Some(entity) if matches!(entity.kind, 2 | 5 | 6 | 8) => {
            entity.construct_owner(shared)?;
            entity.init_seed(selector.spawn_seed, game_update, costs, index)
        }
        _ => {
            // Tinted rocks, poop, blocks, pits and other GridBase objects
            // have no runtime object yet; their constructor still draws.
            let _primary = shared.next_u32();
            let _secondary = shared.next_u32();
            Ok(())
        }
    }
}

/// Doors, walls and normal shape0 ambient decoration after the selectors.
/// `entered` marks a room admitted by a door transition. Its doors already
/// carry the state and path cost of the source's later reopening, which
/// follows Door.Init's state0/2 -> 1 remap; Init's draws still run.
pub fn admit_boundary(
    room: &mut Room,
    shared: &mut Mt19937,
    game_update: u32,
    entered: bool,
) -> Result<(), String> {
    let descriptor = room
        .descriptor
        .as_ref()
        .ok_or("grid admission requires descriptor")?;
    if descriptor.template.room_type == 16 {
        return Err("special room wall replacements are outside fresh grid ownership".into());
    }
    // Catalogue0 (special STB, holding the start room) and1 (Basement) share
    // the stage1 Basement backdrop and its shape0 ambient producer.
    if descriptor.asset_stage > 1 || descriptor.template.shape_index() != 0 {
        return Err("ambient grid admission requires a fresh Basement shape0 room".into());
    }
    let grid = room
        .grid
        .as_mut()
        .ok_or("grid admission requires live geometry")?;
    let width = usize::from(grid.width);
    let height = usize::from(grid.height);
    if width < 3
        || height < 3
        || width * height != grid.cells.len()
        || grid.cells.len() > crate::grid::MAX_GRID_CELLS
    {
        return Err("grid admission dimensions disagree with actual cells".into());
    }
    let costs = room
        .path_costs
        .as_mut()
        .expect("source Room.Init path costs");
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
        let index = column + row * width;
        let entity = grid
            .cells
            .get_mut(index)
            .and_then(|cell| cell.entity.as_mut())
            .filter(|entity| entity.kind == 16)
            .ok_or("enabled door lacks actual door geometry")?;
        let (state, cost) = (entity.state, costs.get(index)?);
        entity.construct_owner(shared)?;
        // Room5c5722 draws a fresh caller seed after actual construction.
        entity.init_seed(shared.next_u32(), game_update, costs, index)?;
        if entered {
            entity.state = state;
            costs.store(index, cost)?;
        }
    }
    for column in 0..width {
        admit_wall(grid, costs, shared, column, game_update)?;
        admit_wall(
            grid,
            costs,
            shared,
            column + (height - 1) * width,
            game_update,
        )?;
    }
    for row in 0..height - 1 {
        admit_wall(grid, costs, shared, row * width, game_update)?;
        admit_wall(grid, costs, shared, row * width + width - 1, game_update)?;
    }
    admit_ambient_shape0(grid, costs, shared, descriptor, game_update)?;
    Ok(())
}

fn admit_wall(
    grid: &mut crate::grid::GridDescriptor,
    costs: &mut RoomCosts,
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
    entity.init_seed(shared.next_u32(), game_update, costs, index)
}

/// Room5cf473..5cf532: the normal shape0 producer owns an independent RNG35
/// seeded from descriptor decoration+list index. Candidate occupancy, signed
/// path costs and each admitted constructor decide actual shared draws.
fn admit_ambient_shape0(
    grid: &mut crate::grid::GridDescriptor,
    costs: &mut RoomCosts,
    shared: &mut Mt19937,
    descriptor: &crate::floor::Descriptor,
    game_update: u32,
) -> Result<(), String> {
    let seed = (descriptor.decoration_seed & 0x7fff_ffff).wrapping_add(descriptor.index as u32);
    let mut rng = Rng::new(seed, [5, 9, 7]);
    let attempts = 2 * (rng.bounded(4) + rng.bounded(5));
    let mut attempt = 0;
    while attempt < attempts {
        let index = rng.bounded(grid.cells.len() as u32) as usize;
        if grid.cells[index].entity.is_none() {
            // Caller5cf504 consumes its seed before factory5c5070 checks cost.
            // A rejected null-cell attempt still advances the loop twice.
            let primary = shared.next_u32();
            if costs.get(index)? <= 900 {
                let mut entity = GridEntity {
                    kind: 1,
                    variant: 0,
                    state: 0,
                    collision_class: 0,
                    spawn_seed: 0,
                    source_owner: None,
                };
                entity.construct_owner(shared)?;
                entity.init_seed(primary, game_update, costs, index)?;
                grid.cells[index].entity = Some(entity);
            }
            attempt += 1;
        }
        attempt += 1;
    }
    Ok(())
}
