//! Actual50E3F0/479490 ParentPlayer ordinary7/1/0 callback.
//! The NPC476120 body remains a separate origin in the same actor store.
use crate::{
    actors::{ActorKind, ActorRef},
    attack_color::AttackColor,
    attack_flags::AttackFlags,
    audio::SoundRequest,
    grid::Grid,
    lasers::Impact,
    lasers::{Laser, LaserOrigin},
    motion::WorldVector,
    phase::Phase,
    State,
};
include!(concat!(env!("OUT_DIR"), "/laser_root_catalog.rs"));

const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };
const PLAYER: ActorRef = ActorRef {
    kind: ActorKind::Player,
    id: 0,
};

#[derive(Clone, Copy)]
pub(crate) struct OrdinaryBrimBirth {
    pub(crate) player_position: WorldVector,
    pub(crate) aim_db4_cardinal: WorldVector,
    ///50B720(kind4), after the actual118 Head*Shoot wardrobe callback.
    pub(crate) render_offset_from_50b720: WorldVector,
    pub(crate) flags_c98: AttackFlags,
    pub(crate) color_ccc: AttackColor,
}

#[derive(Clone, Copy)]
pub(crate) struct OrdinaryTechnologyBirth {
    pub(crate) player_position: WorldVector,
    pub(crate) aim_cardinal: WorldVector,
    ///Actual50B720(kind0) directed eye lookup, never Brimstone's Mouth.
    pub(crate) render_offset_from_50b720: WorldVector,
    pub(crate) flags_c98: AttackFlags,
    pub(crate) color_ccc: AttackColor,
}

///Actual50E1C0 mode1/r8false input. The caller owns the real selected
///RightEye point before Raw; C88/C98/CCC are read by the factory after Spawn.
#[derive(Clone, Copy)]
pub(crate) struct Technology2Birth {
    pub(crate) player_position: WorldVector,
    pub(crate) aim: WorldVector,
    pub(crate) render_offset_from_50b720: WorldVector,
}

#[derive(Clone, Copy)]
struct TechnologyFactoryBirth {
    player_position: WorldVector,
    aim: WorldVector,
    render_offset_from_50b720: WorldVector,
}

#[derive(Clone, Copy)]
enum TechnologyPropertySource {
    Kind2 {
        flags: AttackFlags,
        color: AttackColor,
        damage: fn(&State) -> Result<f32, String>,
    },
    CurrentTechnology2Cache,
}

///A familiar's thin Laser (Robo-Baby 467010): ShootAngle 476120(variant 2,
///position, timeout 3, offset, the familiar, angle) draws the seed and
///spawns 7/2/0 with the familiar as spawner and Parent (447FC0, offset
///+7D4 = position - Parent position); the caller then sets the damage
///(+48C), the depth (+7E4) and runs one explicit Update.
pub(crate) struct FamiliarTechnologyBirth {
    pub(crate) familiar: u32,
    pub(crate) familiar_variant: u32,
    pub(crate) position: WorldVector,
    pub(crate) angle: f32,
    pub(crate) render_offset: WorldVector,
    pub(crate) damage: f32,
    pub(crate) depth: f32,
}

pub(crate) fn admit_familiar_technology(
    state: &mut State,
    birth: FamiliarTechnologyBirth,
) -> Result<ActorRef, String> {
    let parent = state
        .familiars
        .iter()
        .find(|familiar| familiar.id == birth.familiar)
        .ok_or("familiar Laser Parent is unresolved")?
        .position;
    let id = state.next_laser;
    let next = id.checked_add(1).ok_or("Laser handle exhausted")?;
    let seed = state.boot_lifecycle.shared.next_u32(); //476151.
    let mut laser = Laser::initialize_technology(id, seed, state.game_update_count);
    laser.origin = LaserOrigin::FamiliarTechnology;
    laser.grid_collision_class = 3; //Laser Init; 476120 writes no class.
    laser.position = birth.position;
    laser.angle = birth.angle;
    laser.remaining_updates = 3;
    laser.render_offset = birth.render_offset;
    laser.spawner_type = 3;
    laser.spawner_variant = Some(birth.familiar_variant);
    laser.parent = Some(ActorRef {
        kind: ActorKind::Familiar,
        id: birth.familiar,
    });
    laser.parent_offset = delta(birth.position, parent);
    laser.line_flags = Some(laser.attack_flags);
    laser.collision_damage = birth.damage;
    laser.render_sort_offset = birth.depth;
    let actor = ActorRef {
        kind: ActorKind::Laser,
        id,
    };
    state.lasers.push(laser);
    state.next_laser = next;
    state.actor_scheduler.admit(actor)?;
    Ok(actor)
}

///Literal50E1C0 selected mode0/r8true: own variant2/lifetime3 factory.
///No C18 write: the caller immediately invokes Full on the returned actor.
pub(crate) fn admit_ordinary_technology(
    state: &mut State,
    birth: OrdinaryTechnologyBirth,
    kind2_damage: fn(&State) -> Result<f32, String>,
) -> Result<ActorRef, String> {
    if !birth.player_position.is_finite()
        || !birth.render_offset_from_50b720.is_finite()
        || !birth.color_ccc.is_finite()
        || !cardinal(birth.aim_cardinal)
    {
        return Err("Technology requires finite actual cardinal aim and directed eye owner".into());
    }
    admit_technology_factory(
        state,
        TechnologyFactoryBirth {
            player_position: birth.player_position,
            aim: birth.aim_cardinal,
            render_offset_from_50b720: birth.render_offset_from_50b720,
        },
        TechnologyPropertySource::Kind2 {
            flags: birth.flags_c98,
            color: birth.color_ccc,
            damage: kind2_damage,
        },
    )
}

///Literalmode1 factory only. Caller531623..53166B separately applies *.2,
///TTL=-1, actual registered C40 and immediateFull, in that source order.
///No birth, getter, wardrobe or caller capability is inferred by this API.
pub(crate) fn admit_technology2(
    state: &mut State,
    birth: Technology2Birth,
) -> Result<ActorRef, String> {
    if !birth.player_position.is_finite()
        || !birth.render_offset_from_50b720.is_finite()
        || !birth.aim.is_finite()
        || (birth.aim.x == 0.0 && birth.aim.y == 0.0)
    {
        return Err(
            "Technology2 requires finite actual nonzero aim and directed RightEye owner".into(),
        );
    }
    admit_technology_factory(
        state,
        TechnologyFactoryBirth {
            player_position: birth.player_position,
            aim: birth.aim,
            render_offset_from_50b720: birth.render_offset_from_50b720,
        },
        TechnologyPropertySource::CurrentTechnology2Cache,
    )
}

///One canonical7/2/0 Raw/Init/store/Parent/property factory for both actual
///50E1C0 callers. Mode2 random flags and other r8 combinations stay unadmitted.
fn admit_technology_factory(
    state: &mut State,
    birth: TechnologyFactoryBirth,
    properties: TechnologyPropertySource,
) -> Result<ActorRef, String> {
    let id = state.next_laser;
    let next = id.checked_add(1).ok_or("Laser handle exhausted")?;
    let angle = (crate::fmath::atan2(f64::from(birth.aim.y), f64::from(birth.aim.x)) as f32)
        * f32::from_bits(0x4265_2ee0);
    let seed = state.boot_lifecycle.shared.next_u32(); //476151 after50B720.
    let mut laser = Laser::initialize_technology(id, seed, state.game_update_count);
    if matches!(
        properties,
        TechnologyPropertySource::CurrentTechnology2Cache
    ) {
        laser.origin = LaserOrigin::PlayerTechnology2;
    }
    laser.position = birth.player_position;
    laser.angle = angle;
    laser.remaining_updates = 3; //50E235, independently of Brim15.
    laser.render_offset = birth.render_offset_from_50b720;
    laser.spawner_type = 1;
    laser.parent = Some(PLAYER);
    laser.parent_offset = delta(birth.player_position, crate::bombs::player_world(state));
    let actor = ActorRef {
        kind: ActorKind::Laser,
        id,
    };
    state.lasers.push(laser);
    state.next_laser = next;
    state.actor_scheduler.admit(actor)?;
    let index = state.lasers.len() - 1;
    //50E350 reads current C98/CCC after Spawn for mode1. Mode0 retains its
    //independent caller values and actual kind2 getter after v58.
    let (flags, color) = match properties {
        TechnologyPropertySource::Kind2 { flags, color, .. } => (flags, color),
        TechnologyPropertySource::CurrentTechnology2Cache => (
            AttackFlags(state.weapon_cache.flags_c98.0 & !1),
            state.weapon_cache.color_ccc,
        ),
    };
    if !color.is_finite() {
        return Err("Technology actual current CCC is nonfinite".into());
    }
    state.lasers[index].attack_flags = flags;
    state.lasers[index].line_flags = Some(flags);
    state.lasers[index].source_color = color;
    state.lasers[index].current_color = color;
    state.lasers[index].sprite.tint = color.tint;
    state.lasers[index].sprite.offsets = color.offsets;
    let damage = match properties {
        TechnologyPropertySource::Kind2 { damage, .. } => damage(state)?,
        //50E391/v60 reads current C88; it never calls50D090(kind2).
        TechnologyPropertySource::CurrentTechnology2Cache => state.player_stats.damage,
    };
    if !damage.is_finite() || damage < 0.0 {
        return Err("Technology kind2 returned invalid actual damage".into());
    }
    state.lasers[index].collision_damage = damage;
    state.lasers[index].grid_collision_class = 5; //50E2BF.
                                                  //50E2CB..E5 selected r8true: y<-.5 or x>.5 takes raw7E4=-10.
    state.lasers[index].render_sort_offset = match properties {
        TechnologyPropertySource::Kind2 { .. } => {
            if birth.aim.y < -0.5 || birth.aim.x > 0.5 {
                -10.0
            } else {
                3000.0
            }
        }
        //50E320..33A r8false takes -10 on y<=-.5 or x<=-.5.
        TechnologyPropertySource::CurrentTechnology2Cache => {
            if birth.aim.y > -0.5 && birth.aim.x > -0.5 {
                3000.0
            } else {
                -10.0
            }
        }
    };
    Ok(actor)
}

pub(crate) fn advance_ordinary_technology_initial(
    state: &mut State,
    actor: ActorRef,
) -> Result<(), String> {
    if actor.kind != ActorKind::Laser
        || !state
            .lasers
            .iter()
            .any(|laser| laser.id == actor.id && laser.origin.is_technology() && laser.first_update)
    {
        return Err("Technology initial callback has an unresolved actor".into());
    }
    advance_actor(state, Phase::Update, actor.id)
}

fn frontier(state: &mut State, index: usize, channel: &str) {
    if state.lasers[index].origin == LaserOrigin::PlayerTechnology2 {
        crate::technology2::diagnostic(state, channel);
        return;
    }
    state
        .weapon_admission
        .record_unadmitted(state.weapon_cache.primary, channel);
}

fn cardinal(vector: WorldVector) -> bool {
    matches!(
        (vector.x, vector.y),
        (-1.0, 0.0) | (1.0, 0.0) | (0.0, -1.0) | (0.0, 1.0)
    )
}

///Caller admission validates supported modifier/wardrobe history before Raw.
///The real kind2 callback runs after Spawn and before caller519130/C18 binding.
pub(crate) fn admit_ordinary_brim(
    state: &mut State,
    birth: OrdinaryBrimBirth,
    kind2_damage: fn(&State) -> Result<f32, String>,
) -> Result<ActorRef, String> {
    if !birth.player_position.is_finite()
        || !birth.render_offset_from_50b720.is_finite()
        || !birth.color_ccc.is_finite()
        || !cardinal(birth.aim_db4_cardinal)
    {
        return Err(
            "ordinary Brimstone requires actual finite cardinal DB4 and caller properties".into(),
        );
    }
    let id = state.next_laser;
    let next = id.checked_add(1).ok_or("Laser handle exhausted")?;
    //50E407..41A: atan2 widens the source f32 inputs, narrows its result,
    //then6124B0 multiplies by literal f32 radians-to-degrees68EF48.
    let angle = (crate::fmath::atan2(
        f64::from(birth.aim_db4_cardinal.y),
        f64::from(birth.aim_db4_cardinal.x),
    ) as f32)
        * f32::from_bits(0x4265_2ee0);
    let seed = state.boot_lifecycle.shared.next_u32(); //476151.
    let mut laser = Laser::initialize(id, 0, seed, state.game_update_count)?;
    laser.origin = LaserOrigin::PlayerCharged;
    laser.position = birth.player_position;
    laser.angle = angle;
    laser.remaining_updates = 15;
    laser.render_offset = birth.render_offset_from_50b720;
    laser.spawner_type = 1;
    laser.parent = Some(PLAYER);
    //4761D8 requestedP - actualParentP, independently of render458/DC4.
    laser.parent_offset = delta(birth.player_position, crate::bombs::player_world(state));
    let actor = ActorRef {
        kind: ActorKind::Laser,
        id,
    };
    state.lasers.push(laser);
    state.next_laser = next;
    state.actor_scheduler.admit(actor)?;
    let index = state.lasers.len() - 1;
    state.lasers[index].attack_flags = birth.flags_c98;
    state.lasers[index].line_flags = Some(birth.flags_c98);
    state.lasers[index].source_color = birth.color_ccc;
    state.lasers[index].current_color = birth.color_ccc;
    state.lasers[index].sprite.tint = birth.color_ccc.tint;
    state.lasers[index].sprite.offsets = birth.color_ccc.offsets;
    let damage = kind2_damage(state)?; //50E4A7 actual50D090(kind2).
    if !damage.is_finite() || damage < 0.0 {
        return Err("ordinary Brimstone kind2 returned invalid actual damage".into());
    }
    //Actual Laser.v60=448120 stores48C only; it never rescales the sprite.
    state.lasers[index].collision_damage = damage;
    state.lasers[index].render_sort_offset = if birth.aim_db4_cardinal.y > -0.5 {
        3000.0
    } else {
        -10.0
    };
    Ok(actor)
}

///5205C1 explicit full callback follows caller519130/C18 binding, even when
///the outer Player callback has fast bit4000. It is never an extra clock step.
pub(crate) fn advance_ordinary_brim_initial(
    state: &mut State,
    actor: ActorRef,
) -> Result<(), String> {
    if actor.kind != ActorKind::Laser
        || !state.lasers.iter().any(|laser| {
            laser.id == actor.id && laser.origin == LaserOrigin::PlayerCharged && laser.first_update
        })
    {
        return Err("ordinary Brimstone initial callback has an unresolved actor".into());
    }
    advance_actor(state, Phase::Update, actor.id)
}

///Literal479490: sound first, Raw Effect50/Parent/current44B color/sort, its
///immediate Update, then Player7F5=true/Line50mode0/width120 and478320.
fn initialize_first(state: &mut State, index: usize) -> Result<(), String> {
    state.audio.admit(
        &mut state.boot_lifecycle.shared,
        state.system_clock.system_counter.wrapping_sub(1),
        //Actual table66CDB8 variant1→476320→ParentPlayer476410.
        SoundRequest {
            id: if state.lasers[index].variant == 2 {
                133
            } else {
                7
            },
            //4763FD actual666118=.6; variant1/476435 keeps its own.5. A
            //non-Player Parent takes 4762A3's 1.0.
            volume: if state.lasers[index]
                .parent
                .is_some_and(|parent| parent != PLAYER)
            {
                1.0
            } else if state.lasers[index].variant == 2 {
                0.6
            } else {
                0.5
            },
            delay: 2,
            looping: false,
            pitch: 1.0,
        },
    );
    spawn_initial_impact(state, index)?;
    let laser = &mut state.lasers[index];
    laser.line_enabled = true;
    laser.line.initialize(50, 0);
    laser.line.width = Some(120.0);
    Ok(())
}

///4796BC actual variant2 Impact factory, shared by ordinary subtype0 and
///controlled subtype1. The controlled sound dispatcher has already returned.
pub(crate) fn initialize_controlled_technology_impact(
    state: &mut State,
    index: usize,
) -> Result<(), String> {
    let laser = &state.lasers[index];
    if laser.origin != LaserOrigin::PlayerControlled
        || laser.variant != 2
        || laser.subtype != 1
        || !laser.first_update
        || laser.parent != Some(PLAYER)
        || laser.child.is_some()
        || laser.impact.is_some()
    {
        return Err("controlled Technology first Impact requires actual fresh7/2/1".into());
    }
    spawn_initial_impact(state, index)
}

fn spawn_initial_impact(state: &mut State, index: usize) -> Result<(), String> {
    let id = state.next_effect;
    let next = id.checked_add(1).ok_or("Effect handle exhausted")?;
    let seed = state.boot_lifecycle.shared.next_u32(); //47955E before550A40.
    let actor = ActorRef {
        kind: ActorKind::Effect,
        id,
    };
    let parent = ActorRef {
        kind: ActorKind::Laser,
        id: state.lasers[index].id,
    };
    let laser = &state.lasers[index];
    let mut impact = Impact {
        birth_game_update: Some(state.game_update_count),
        id,
        parent: Some(parent),
        parent_known: true,
        spawn_seed: seed,
        position: laser.position,
        position_offset: ZERO,
        scale: WorldVector { x: 1.0, y: 1.0 },
        rotation: 0.0,
        //4795F4 writes Brim sort-5; variant2 leaves real Effect Init7F8=0.
        depth: if laser.variant == 2 {
            0.0
        } else {
            laser.render_sort_offset - 5.0
        },
        actor: if laser.variant == 2 {
            "gfx/1000.050_laserimpact.anm2"
        } else {
            "gfx/1000.050_brimstoneimpact.anm2"
        }
        .into(),
        source_subtype: Some(if laser.variant == 2 { 0 } else { 1 }),
        sprite_flip_x: None,
        raw_814: None,
        animation: if laser.variant == 2 { "Idle" } else { "Start" }.into(),
        animation_clock: 0.0,
        animation_playing: true,
        source_color: Some(laser.current_color),
        current_color: Some(laser.current_color),
        dead: false,
        removed: false,
    };
    state.next_effect = next;
    state.actor_scheduler.admit(actor)?;
    state.lasers[index].child = Some(actor);
    if state.lasers[index].variant == 2 {
        //479726..77B, AFTER actual Effect Spawn/SetChild/SetParent and BEFORE
        //v58 color/immediate Full. These are two real shared Bound consumers.
        impact.sprite_flip_x = Some(state.boot_lifecycle.shared.bounded(2) == 0);
        impact.raw_814 = Some(state.boot_lifecycle.shared.bounded(4) as f32 * 90.0);
        impact.source_color = Some(state.lasers[index].current_color);
        impact.current_color = Some(state.lasers[index].current_color);
    }
    crate::laser_npc_runtime::advance_impact_full(&mut impact, false)?;
    state.lasers[index].impact = Some(impact);
    Ok(())
}

///Authored real Layer0 frame owner consumed by47842E..47F. The actor has two
///layers; its XPosition+Width is not an assumed zero displacement.
fn root_extent(laser: &Laser) -> Result<f32, String> {
    if laser.origin.is_technology() {
        if laser.variant != 2 || laser.sprite.animation != "RedLaser" || THIN_LASER_LAYER_COUNT != 1
        {
            return Err("Technology requires its actual one-layer RedLaser owner".into());
        }
        return Ok(0.0); // Authored7/2 layer count1 ->47842C skips root lookup.
    }
    if LASER_LAYER_COUNT <= 1 {
        return Ok(0.0);
    }
    if laser.sprite.animation != "LargeRedLaser" {
        return Err("ordinary Laser current authored clip is outside its source owner".into());
    }
    let mut frame = laser.sprite.frame.floor() as u32;
    for &(delay, extent) in LASER_ROOT_EXTENTS {
        if frame < delay {
            return Ok(extent);
        }
        frame -= delay;
    }
    Err("Laser source root frame exceeds its authored clip".into())
}

fn delta(a: WorldVector, b: WorldVector) -> WorldVector {
    WorldVector {
        x: a.x - b.x,
        y: a.y - b.y,
    }
}

fn body(state: &mut State, index: usize, grid: &Grid, dc4: WorldVector) -> Result<bool, String> {
    let player = crate::bombs::player_world(state);
    let familiar_parent = match state.lasers[index].parent {
        Some(ActorRef {
            kind: ActorKind::Familiar,
            id,
        }) => state
            .familiars
            .iter()
            .find(|familiar| familiar.id == id && !familiar.removed)
            .map(|familiar| familiar.position),
        _ => None,
    };
    let extent = root_extent(&state.lasers[index])?;
    let age = state.game_update_count;
    let laser = &mut state.lasers[index];
    match laser.parent {
        Some(PLAYER) => laser.position = (player + laser.parent_offset) + dc4, //479180/197.
        //47835E: any other Parent, its position plus +7D4.
        _ => {
            if let Some(position) = familiar_parent {
                laser.position = position + laser.parent_offset;
            }
        }
    }
    let radians = laser.angle * f32::from_bits(0x3c8e_fa35); //6124A0.
    let heading = WorldVector {
        x: crate::fmath::cos(radians),
        y: crate::fmath::sin(radians),
    };
    let root_offset = heading * extent;
    let start = laser.position + root_offset;
    let bounds = crate::laser_npc_runtime::room_bounds_from_grid(grid);
    let mut endpoint =
        crate::laser_npc_runtime::end_point(start, heading, laser.render_offset, bounds);
    let edge = delta(endpoint, start);
    if (edge.x * root_offset.x + edge.y * root_offset.y) < 0.0 || endpoint == start {
        //6128E0 uses a separately rounded length/divide/per-axis multiply.
        let length = (root_offset.x * root_offset.x + root_offset.y * root_offset.y).sqrt();
        let resized = if length > 0.0 {
            root_offset * (10.0 / length)
        } else {
            root_offset
        };
        endpoint = start + resized;
    }
    laser.endpoint = endpoint;
    laser.line.begin_curve();
    laser.line.origin = start;
    laser.line.offset = endpoint;
    laser.line.straight_controls(); //5791F0(mode0), no target mutation.
                                    //479250 variant2 explicitly selects .7; other admitted ordinary paths
                                    //reach478FF8's .3. These are source calls, not an accuracy setting.
    laser
        .line
        .resample(if laser.variant == 2 { 0.7 } else { 0.3 })?;
    //5786A0 starts at point2. Caller class3 discards every terrain result;
    //actual Technology class5 erases [first blocked point,end), excluding
    //that point. Terrain0/1/4/5 continue; no fitted endpoint/range clamp.
    let mut cut = None;
    for (i, point) in laser.line.points.iter().enumerate().skip(2) {
        let collision = grid.collision_at(*point + laser.render_offset * 0.4);
        if laser.grid_collision_class == 5 && collision > 1 && collision != 4 && collision != 5 {
            cut = Some(i);
            break;
        }
    }
    if let Some(index) = cut {
        laser.line.points.truncate(index);
    }
    let end = *laser
        .line
        .points
        .last()
        .ok_or("ordinary Laser line has no endpoint")?;
    let before = laser.line.points[laser.line.points.len() - 2];
    //4787A0 passes penultimate as RDI and end as RSI to6129D0 (RDI-RSI).
    let edge = delta(before, end);
    let magnitude = (edge.x * edge.x + edge.y * edge.y).sqrt();
    let normalized = if magnitude > 0.0 {
        edge * (1.0 / magnitude)
    } else {
        edge
    };
    //4786A4/6B0 follows the actual registered Child4A0, not retained storage.
    if let Some(child) = laser.child {
        let impact = laser
            .impact
            .as_mut()
            .filter(|impact| child.kind == ActorKind::Effect && impact.id == child.id)
            .ok_or("ordinary Laser actual Child identity is unresolved")?;
        if laser.variant == 2 {
            //478FA0: thin Laser writes its child's physical position directly.
            //It does not execute variant1/3 scale,458 or angle callbacks.
            impact.position = end + laser.render_offset;
        } else {
            impact.scale = WorldVector {
                x: laser.sprite.scale.x * 0.5 + 0.5,
                y: laser.sprite.scale.y,
            };
            impact.position = laser.position;
            impact.position_offset = delta(end + laser.render_offset, laser.position);
            let angle = (crate::fmath::atan2(f64::from(normalized.y), f64::from(normalized.x))
                as f32)
                * f32::from_bits(0x4265_2ee0);
            impact.rotation = crate::source_angles::wrap_180(angle - 90.0);
        }
    }
    //Missing Parent deliberately leaves31F unchanged (478AD7→478A60).
    if laser.parent == Some(PLAYER) && state.player.dead {
        laser.dead = true;
    }
    if laser.dead || laser.contact_disabled {
        if laser.variant == 2 {
            laser.remove(); //478AFC/478B04→478B0D, no thick-Laser fade.
        } else {
            laser.sprite.scale.x *= 0.895;
            if laser.sprite.scale.x < 0.08 {
                laser.remove();
            }
        }
    }
    //Actual478A7A passes edx0 to444890. Entity sequence is not added here.
    let contact_due = !laser.dead
        && !laser.fast
        && laser
            .entity_flags
            .ok_or("ordinary Laser actual flags318 history absent")?
            & 0x4000
            == 0
        && (age as i32) % 2 == 0
        && age.wrapping_sub(laser.birth_game_update) as i32 > 0;
    //478B23→478C80 simplifies every line callback that did not dispatch
    //477BB0. That real contact path simplifies after its grid callbacks.
    if !contact_due {
        laser.line.simplify(0.001);
    }
    Ok(contact_due)
}

pub(crate) fn advance_actor(state: &mut State, phase: Phase, id: u32) -> Result<(), String> {
    if phase == Phase::Skipped {
        return Ok(());
    }
    let index = state
        .lasers
        .iter()
        .position(|laser| laser.id == id)
        .ok_or("ordinary Laser scheduled handle is unresolved")?;
    //Pure validation precedes valid-unmapped history/property diagnostics.
    //It changes no Source owner, admission, queue, registry, audio or RNG.
    let laser = &state.lasers[index];
    laser.validate_contact_geometry()?;
    if !crate::bombs::player_world(state).is_finite()
        || state
            .attack_runtime
            .player_dc4
            .is_some_and(|value| !value.is_finite())
        || !laser.position.is_finite()
        || !laser.velocity.is_finite()
        || !laser.render_offset.is_finite()
        || !laser.parent_offset.is_finite()
        || !laser.angle.is_finite()
        || laser.previous_angle.is_some_and(|angle| !angle.is_finite())
        || !laser.max_distance.is_finite()
        || !laser.sprite.scale.is_finite()
        || !laser.sprite.frame.is_finite()
        || !laser.sprite.speed.is_finite()
        || !laser.source_color.is_finite()
        || !laser.current_color.is_finite()
        || !laser.collision_damage.is_finite()
        || laser.collision_damage < 0.0
    {
        return Err("ordinary Laser callback contains nonfinite actual numeric owners".into());
    }
    if phase == Phase::Update && !state.lasers[index].active {
        return Ok(());
    }
    let dc4 = if state.lasers[index].origin == LaserOrigin::FamiliarTechnology {
        Some(ZERO)
    } else {
        state.attack_runtime.player_dc4
    };
    let Some(dc4) = dc4 else {
        frontier(state, index, "ordinaryBrimstoneParentHistory");
        return Ok(());
    };
    if !dc4.is_finite() {
        return Err("ordinary Laser current DC4 is nonfinite".into());
    }
    let laser = &state.lasers[index];
    if laser.entity_flags.is_none()
        || laser.previous_angle.is_none()
        || laser.uninitialized.is_none()
    {
        frontier(state, index, "ordinaryBrimstoneActorHistory");
        return Ok(());
    }
    if laser.subtype != 0
        || !laser.line_enabled && !laser.first_update
        || match laser.origin {
            LaserOrigin::PlayerCharged => laser.variant != 1 || laser.grid_collision_class != 3,
            LaserOrigin::PlayerTechnology | LaserOrigin::PlayerTechnology2 => {
                laser.variant != 2 || laser.grid_collision_class != 5
            }
            LaserOrigin::FamiliarTechnology => {
                laser.variant != 2 || laser.grid_collision_class != 3
            }
            LaserOrigin::NpcShootAngle | LaserOrigin::PlayerControlled => true,
        }
        || !laser.has_supported_neutral_body_flags()
        || laser.rotation_enabled
        || laser.max_distance != 0.0
        || laser.line_controller != 0
        || laser.sprite.scale.x > 1.0
    {
        frontier(state, index, "ordinaryBrimstoneBodyProperties");
        return Ok(());
    }
    let grid = Grid::new(
        state
            .room
            .grid
            .clone()
            .ok_or("ordinary Laser actual grid absent")?,
    )?;
    //5791F0 mode0 queries the manager before returning without target writes.
    crate::laser_runtime::validate_mode0_query(state)?;
    let initial = state.lasers[index].first_update;
    if phase == Phase::Interpolate {
        if !state.lasers[index].begin_fast() {
            return Ok(());
        }
    } else if initial {
        initialize_first(state, index)?;
    }
    let due = body(state, index, &grid, dc4)?;
    if due {
        crate::laser_runtime::run_line_contacts(state, index, &grid)?;
    }
    let laser = &mut state.lasers[index];
    if laser.remaining_updates > 0 && !laser.fast {
        laser.remaining_updates -= 1;
        if laser.remaining_updates == 0 {
            if laser.variant == 2 {
                laser.dead = true; //4791CA; next body observes31F and Remove.
            } else {
                laser.contact_disabled = true; //479370 thick-Laser fade owner.
            }
        }
    }
    laser.previous_angle = Some(laser.angle);
    if initial {
        laser.first_update = false;
    } else if !laser.fast {
        laser.finish_full()?;
    }
    Ok(())
}
