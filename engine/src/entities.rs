//! NPC family registry. A family owns construction, the full/intermediate
//! update callback and the post-contact grid query for its authored types.
//! Game owns one shared update context and admits each family's births in
//! producer order right after its callback returns, so shared-MT draws and
//! actor identities keep the source order. Families are never batched: the
//! scheduler still visits one actor callback at a time.
use crate::{
    audio::AudioState,
    enemy_projectiles::ProjectileBirth,
    familiars::NpcEffectAdmission,
    fireplaces::{LootBirth, ParticleBirth},
    grid::{Grid, GridDescriptor},
    motion::WorldVector,
    mt::Mt19937,
    npcs::Npc,
    phase::Phase,
    player_items::PlayerItems,
    room_costs::RoomCosts,
    room_spawns::SpawnSelector,
    roundworm::Shot,
};

pub(crate) struct SpawnContext {
    pub stage: u32,
    pub game_update_count: u32,
}

/// The union of the owners any supported NPC callback reads or writes.
pub(crate) struct NpcWorld<'a> {
    /// Actual current Game full-update counter (`State::native_game_frame`),
    /// including admission callbacks.
    pub game_counter: u32,
    pub player: WorldVector,
    pub player_velocity: WorldVector,
    pub grid: &'a Grid,
    pub grid_descriptor: Option<&'a mut GridDescriptor>,
    pub path_costs: Option<&'a mut RoomCosts>,
    /// FreePosition occupancy, gathered only for families that ask.
    pub occupied: &'a [WorldVector],
    /// Room+16ac plus Room+16b0: pending and current countable actors.
    pub countable_total: i32,
    pub audio: &'a mut AudioState,
    pub system_counter: u32,
    pub player_items: Option<&'a PlayerItems>,
    pub effects: NpcEffectAdmission<'a>,
    pub item_death_pickups: crate::item_death_effects::PickupAdmission<'a>,
    /// Room+0x10 descriptor type of the current room.
    pub room_type: u32,
    /// Room::GetFrameCount 5cbbb0: Game+ce9f4 minus the Room.Init stamp
    /// +159c, -1 while the room is not loaded; None without that stamp.
    pub room_frame: Option<i32>,
    /// The room's other NPCs, live, for families that link actors.
    pub peers: Peers<'a>,
    /// The last contact pass's tear partition at live positions and radii
    /// (tiny-fly kill checks run inside the Duke's AI).
    pub tear_probe: &'a [(WorldVector, f32)],
    /// Knife positions and radii in list order.
    pub knife_probe: &'a [(WorldVector, f32)],
}

/// The room NPC list around the updating actor, in list order. Peers read
/// what earlier actors already wrote this update.
pub(crate) struct Peers<'a> {
    pub before: &'a mut [Npc],
    pub after: &'a mut [Npc],
}

impl Peers<'_> {
    /// Every other NPC in room list order.
    pub fn iter(&self) -> impl Iterator<Item = &Npc> {
        self.before.iter().chain(self.after.iter())
    }

    pub fn get(&self, id: u32) -> Option<&Npc> {
        self.iter().find(|npc| npc.id == id && !npc.removed)
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut Npc> {
        self.before
            .iter_mut()
            .chain(self.after.iter_mut())
            .find(|npc| npc.id == id && !npc.removed)
    }
}

/// Spawner-written fields of a mid-room NPC.
pub(crate) struct SpawnedNpc {
    pub selector: SpawnSelector,
    pub velocity: WorldVector,
    /// Entity+1c: the spawner's type (0 for a NULL spawner).
    pub spawner_type: u32,
    /// +320: spawned by a boss (Attack Fly chase friction x0.9).
    pub boss_spawned: bool,
    /// Sprite scale (+2c4) multiplier.
    pub scale: f32,
    /// HP (+484) multiplier.
    pub health_scale: f32,
    /// Max HP (+488) multiplier.
    pub max_health_scale: f32,
    /// Added to HP (+484) after the scales (Gurdy's flies lose 2).
    pub health_delta: f32,
    /// Whether the APPEAR flag stays set (Gurdy's summons appear).
    pub appear: bool,
}

/// Actors an NPC callback creates. Game admits them in this order.
pub(crate) enum Birth {
    Blood(u32),
    Projectile(ProjectileBirth),
    Shot(Shot),
    Ember(ParticleBirth),
    Loot(LootBirth),
    Laser(crate::lasers::LaserBirth),
    /// Spawn550a40 of an NPC mid-room (no champion roll in a live room).
    Npc(SpawnSelector),
    /// Creep 5516d0 hurt of the player from a spawner-run first update.
    CreepHit,
    /// Game::Fart 5522d0's knockback around `position` (see `fatty::fart`).
    Fart {
        position: WorldVector,
        radius: f32,
        source: u32,
    },
    /// A projectile 9.v other than the blood shot, e.g. Mega Fatty's puke 9.3.
    VariantProjectile(ProjectileBirth, u32),
    /// A FireProjectiles shot of variant `.1` whose ProjectileParams carry
    /// flags and a centre (49ee5d/49ee8c).
    FlaggedProjectile(
        ProjectileBirth,
        u32,
        crate::enemy_projectiles::ProjectileFlags,
    ),
    /// Mega Maw's Summon (4e03fe): Spawn550a40 of a Hopper with no spawner,
    /// then its +440 landing target, state 4, +f58 5 and a forced Hop clip
    /// seeked to frame 5 (40fed0).
    Hop {
        selector: SpawnSelector,
        target: WorldVector,
    },
    /// Player TakeDamage(1.0, flags 0, countdown 30) from an NPC's AI (Mega
    /// Fatty's fart hits a player within 100 px).
    PlayerHurt,
    /// Mega Fatty's SUCK 4dd438: every pulled actor's velocity gains
    /// `normalize(center - pos)·k / friction`, `k = clamp(0.9(600 - L)/600, 0, 0.9)`.
    Suck(WorldVector),
    /// Game::ShakeScreen 551da0: Game+ce9fc = the duration (no draws).
    Shake(i32),
    /// Room5cbb30 Hurt(damage) on the grid cell under `position` (a no-op
    /// outside the room or on an empty cell).
    GridHurt {
        position: WorldVector,
        damage: i32,
    },
    /// Spawn550a40 of the bomb explosion effect 1000.1 (Chub's swallowed
    /// bomb), spawner NULL.
    ExplosionEffect {
        seed: u32,
        position: WorldVector,
    },
    /// Spawn550a40 of a pickup-less bomb (Troll Bomb variant 3), spawner NULL.
    Bomb {
        variant: u32,
        seed: u32,
        position: WorldVector,
    },
    /// Mulligan EXPLODE 4e8200: its blast, effect seeds, cardinal burst, Kill
    /// and dead branch, in the spawner's update order.
    MulliganExplode,
    /// A mid-room NPC spawned with a velocity and its APPEAR flag cleared.
    FlungNpc(SpawnSelector, WorldVector),
    /// A mid-room NPC with spawner-written fields (Spawn550a40 then the
    /// caller's stores), APPEAR cleared.
    Spawned(SpawnedNpc),
    /// A mid-room NPC the spawner parents to itself and adopts as its child
    /// (SetParent447fc0 + SetChild447f40), updated at once (vtable+0x18).
    Child(SpawnSelector),
    /// A mid-room NPC parented to its spawner (SetParent447fc0 only; the
    /// spawner's child stays unset, no immediate update): Peep's eyes.
    Parented(SpawnSelector),
    /// Envy's split child 48c9d0: config row `variant % 10`, the split
    /// variant, the parent's seed, APPEAR cleared, updated at once.
    EnvyChild {
        position: WorldVector,
        variant: u32,
        seed: u32,
    },
    /// Super Envy's third child: bounded(2) at admission (after the first
    /// two children's nested Updates), spawned when level < roll + 1.
    EnvyThird {
        position: WorldVector,
        variant: u32,
        seed: u32,
        level: u32,
    },
    /// A mid-room NPC parented to the spawner, without entity collision,
    /// hidden and with APPEAR cleared (Gemini's umbilical cord pieces).
    Attached(SpawnSelector),
    /// Base kill 4fb0a2: Fly Explosion 1000.3 with the seed drawn at the
    /// kill, its position and the actor's SplatColor.
    FlyExplosion {
        seed: u32,
        position: WorldVector,
        color: crate::sprite_color::Color,
    },
    /// Kill47FC80 Have234 (Infestation 2): Player50c980 would spawn a
    /// BlueSpider familiar 3.73 at the dying NPC. Port gap: that familiar is
    /// unported, so nothing is spawned and the caller records the frontier.
    UnportedBlueSpider,
    /// Spawn550a40 or a Room producer of a type no family or owner ports
    /// (Mom's Clotty/Maw summons, her death's Room+2480 particle rain):
    /// nothing is spawned and the room records the factory frontier.
    UnportedFactory {
        entity_type: u32,
        variant: u32,
        subtype: u32,
    },
    /// Mom's init (4d3b80): Room::RemoveDoor 5cbee0 for slots 0..7, each
    /// existing door replaced by a wall seeded with Random(), then the
    /// current descriptor's door mask (+0x10) cleared.
    RemoveDoors,
    /// Mom's init (4d3c5b): every player's position (+448) is set.
    PlacePlayer(WorldVector),
    /// Mom's stomp (4d4819): Game554490 at `position`, damage 300, radius
    /// 50, no line check, source Mom, DamageFlags 4.
    MomStomp {
        position: WorldVector,
        source: u32,
    },
}

pub(crate) type Advance =
    fn(&mut Npc, Phase, &mut Mt19937, &mut NpcWorld, &mut Vec<Birth>) -> Result<(), String>;

pub(crate) struct Family {
    /// Authored entity types this family's callbacks own.
    pub kinds: &'static [u32],
    pub spawn: fn(u32, &SpawnSelector, &SpawnContext) -> Result<Option<Npc>, String>,
    pub advance: Advance,
    pub post_contact_grid: fn(&mut Npc, &Grid) -> Result<(), String>,
    /// Actual Entity+318 view of the family's own flag owner.
    pub entity_flags: fn(&Npc) -> u32,
    /// Which actors FindFreePosition5c7f30 must avoid for this family.
    pub occupancy: Occupancy,
}

/// FindFreePosition occupancy, gathered into `NpcWorld::occupied` only for
/// families that ask.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Occupancy {
    None,
    /// avoidAll=false: bombs, pickups and slots.
    Items,
    /// avoidAll=true: also the player and every NPC.
    Actors,
}

/// Families without a subclass flag owner keep Init441c80's cleared flags
/// ORed with4.
pub(crate) fn ordinary_flags(_: &Npc) -> u32 {
    4
}

pub(crate) fn no_grid_query(_: &mut Npc, _: &Grid) -> Result<(), String> {
    Ok(())
}

/// Class3 flyers with authored sample points (the Duke of Flies): 442c3b
/// clamps the center, then only wall and door samples reflect (442b90).
pub(crate) fn wall_collision(npc: &mut Npc, grid: &Grid) -> Result<(), String> {
    let points = npc.config().grid_points;
    if npc.grid_collision_class != 3 || points == 0 {
        return clamp_walls(npc, grid);
    }
    let collider = crate::grid::Collider::new(crate::grid::CollisionGeometry {
        radius: npc.collision_radius,
        scale: crate::motion::WorldVector { x: 1.0, y: 1.0 },
        points,
    })?;
    npc.grid_collided = collider
        .resolve_nonplayer_walls(grid, &mut npc.position, &mut npc.velocity, false)?
        .is_some();
    npc.presented_position = npc.position;
    Ok(())
}

/// Source class3 clamps the center first. Families with zero grid
/// collision points have an empty sample loop after it.
/// Ground walkers (grid class 5, authored collision points) resolve against
/// rocks, pits and walls with the source nonplayer ground collider built from
/// their own radius and point count, as Dips do; other classes clamp.
pub(crate) fn ground_collision(npc: &mut Npc, grid: &Grid) -> Result<(), String> {
    // 442bc0 clears +34c even when this pass has no collision samples.
    npc.grid_collided = false;
    let points = npc.config().grid_points;
    if npc.grid_collision_class != 5 || points == 0 {
        return clamp_walls(npc, grid);
    }
    let collider = crate::grid::Collider::new(crate::grid::CollisionGeometry {
        radius: npc.collision_radius,
        scale: crate::motion::WorldVector { x: 1.0, y: 1.0 },
        points,
    })?;
    // 442df8 samples the centre the collider's 442c2d clamp yields.
    let sample_position = grid.clamp_room(npc.position, 0.0);
    if npc.kind == 220 {
        // 44364a: Squirt shares the Dip's 1.2 reflection scale.
        collider.resolve_nonplayer_ground_bouncy(
            grid,
            &mut npc.position,
            &mut npc.velocity,
            false,
        )?;
    } else {
        collider.resolve_nonplayer_ground(grid, &mut npc.position, &mut npc.velocity, false)?;
    }
    // 442df8 records a solid sample independently of the summed normal.
    npc.grid_collided = collider
        .samples()
        .iter()
        .any(|sample| grid.collision_at(sample_position + *sample) != 0);
    npc.presented_position = npc.position;
    Ok(())
}

/// 442c2d: classes 3 and 5 both clamp the center to the room bounds before
/// the sample loop (empty for zero points, e.g. Larry Jr.'s class 5).
pub(crate) fn clamp_walls(npc: &mut Npc, grid: &Grid) -> Result<(), String> {
    let clamped = grid.clamp_room(npc.position, 0.0);
    match npc.grid_collision_class {
        3 | 5 => npc.position = clamped,
        // 442e40 / 442ef0: classes 1 and 2 clamp only x or only y, test no
        // cells, and flag a grid hit when the centre sits on that bound.
        1 => {
            npc.grid_collided = on_bound(clamped.x, grid, true);
            npc.position.x = clamped.x;
        }
        2 => {
            npc.grid_collided = on_bound(clamped.y, grid, false);
            npc.position.y = clamped.y;
        }
        _ => return Ok(()),
    }
    npc.presented_position = npc.position;
    Ok(())
}

fn on_bound(value: f32, grid: &Grid, horizontal: bool) -> bool {
    let low = grid.clamp_room(
        WorldVector {
            x: f32::MIN,
            y: f32::MIN,
        },
        0.0,
    );
    let high = grid.clamp_room(
        WorldVector {
            x: f32::MAX,
            y: f32::MAX,
        },
        0.0,
    );
    if horizontal {
        value == low.x || value == high.x
    } else {
        value == low.y || value == high.y
    }
}

/// One line per supported family.
pub(crate) static FAMILIES: &[&Family] = &[
    &crate::npcs::FLY_FAMILY,
    &crate::keepers::FAMILY,
    &crate::monstro::FAMILY,
    &crate::fireplaces::FAMILY,
    &crate::level2_flies::FAMILY,
    &crate::dips::FAMILY,
    &crate::roundworm::FAMILY,
    &crate::horfs::FAMILY,
    &crate::gluttony::FAMILY,
    &crate::dingle::FAMILY,
    &crate::peep::FAMILY,
    &crate::boil::FAMILY,
    &crate::deaths_head::FAMILY,
    &crate::chub::FAMILY,
    &crate::gurdy_jr::FAMILY,
    &crate::mega_maw::FAMILY,
    &crate::long_legs::FAMILY,
    &crate::fistula::FAMILY,
    &crate::squirt::FAMILY,
    &crate::wall_hugger::FAMILY,
    &crate::wall_creep::FAMILY,
    &crate::larry::FAMILY,
    &crate::maggots::FAMILY,
    &crate::duke::FAMILY,
    &crate::gemini::FAMILY,
    &crate::envy::FAMILY,
    &crate::gapers::FAMILY,
    &crate::gushers::FAMILY,
    &crate::hoppers::FAMILY,
    &crate::fatty::FAMILY,
    &crate::skinny::FAMILY,
    &crate::globin::FAMILY,
    &crate::greed::FAMILY,
    &crate::gurdy::FAMILY,
    &crate::mega_fatty::FAMILY,
    &crate::mulligans::FAMILY,
    &crate::mom::FAMILY,
];

/// Every NPC type a registered family constructs, in registry order. Tools
/// that measure coverage read the port's NPC surface here instead of keeping
/// a copy.
pub fn implemented_npc_kinds() -> impl Iterator<Item = u32> {
    FAMILIES
        .iter()
        .flat_map(|family| family.kinds.iter().copied())
}

pub(crate) fn family(kind: u32) -> Option<&'static Family> {
    FAMILIES
        .iter()
        .copied()
        .find(|family| family.kinds.contains(&kind))
}

/// Whether a registered family constructs this authored NPC selector. Scenario
/// discovery uses it to find rooms whose every NPC the port admits.
pub fn supports_npc_selector(selector: &SpawnSelector) -> bool {
    let context = SpawnContext {
        stage: 1,
        game_update_count: 0,
    };
    matches!(spawn(0, selector, &context), Ok(Some(_)))
}

/// The first family whose constructor admits the authored selector.
pub(crate) fn spawn(
    id: u32,
    selector: &SpawnSelector,
    context: &SpawnContext,
) -> Result<Option<Npc>, String> {
    for family in FAMILIES {
        if let Some(npc) = (family.spawn)(id, selector, context)? {
            return Ok(Some(npc));
        }
    }
    Ok(None)
}
