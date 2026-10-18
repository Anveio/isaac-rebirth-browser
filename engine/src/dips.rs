//! Ordinary Dip217: source4949c0, missing-Appear staging and Blood7 trail.
use crate::entities::{Birth, Family, NpcWorld, SpawnContext};
use crate::{
    actors::{ActorKind, ActorRef},
    audio::SoundRequest,
    content,
    familiars::Parent,
    gib_counts::{self, AmountScale},
    grid::{Collider, CollisionGeometry, Grid},
    motion::WorldVector,
    mt::Mt19937,
    npcs::Npc,
    phase::Phase,
    poop_effects,
    room_spawns::{SpawnCategory, SpawnSelector},
};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DipState {
    pub entity_flags: u32,
    pub birth_game_update: u32,
    pub current_friction: f32,
    pub flip_x: bool,
    pub tint: [f32; 4],
    pub offsets: [f32; 3],
    /// Base441d81/441e66 per-spawn owners, not permanent eligibility constants.
    pub render_offset: WorldVector,
    pub status_countdown: i32,
    /// Entity+7C0, seeded by441CB0/441CBF from the actual Spawn seed.
    /// A restored state without that owner remains unknown; never reseed it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity_rng: Option<crate::items::Rng>,
}

const CONFIG: &content::EntityConfig = content::entity(217, 0);

pub fn from_selector(id: u32, selector: &SpawnSelector, stage: u32) -> Result<Option<Npc>, String> {
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 217
        || selector.variant != 0
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 || stage == 0 {
        return Err("Dip requires bounded authored position, source seed and actual stage".into());
    }
    let hp = crate::content::initial_hp(CONFIG, stage);
    Ok(Some(Npc {
        hp,
        max_hp: hp,
        animation: "Idle".into(),
        animation_playing: false, // Config47f573 SetFrame(default,0).
        dip: Some(DipState {
            entity_flags: 4,
            birth_game_update: 0,
            current_friction: CONFIG.friction,
            flip_x: false,
            tint: [1.0; 4],
            offsets: [0.0; 3],
            render_offset: ZERO,
            status_countdown: 0,
            entity_rng: Some(crate::items::Rng::new(selector.spawn_seed, [4, 3, 17])),
        }),
        ..Npc::authored(id, CONFIG, selector)
    }))
}

fn play(npc: &mut Npc, name: &str) {
    if npc.animation != name || !npc.animation_playing {
        npc.animation = name.into();
        npc.animation_clock = 0.0;
        npc.animation_frame = 0;
        npc.animation_playing = true;
    }
}

fn ai(npc: &mut Npc, random: &mut Mt19937, world: &mut NpcWorld<'_>) -> Result<(), String> {
    match npc.state {
        0 => {
            npc.state = 3;
            let dip = npc.dip.as_mut().unwrap();
            //494ada..494b58: actual byte-to-f32 conversion uses multiplication.
            let unit = f32::from_bits(0x3b80_8081);
            dip.tint = [0.0, 0.0, 0.0, 255.0 * unit];
            dip.offsets = [65.0 * unit, 41.0 * unit, 35.0 * unit];
            return Ok(()); // Initializer does not run the shared-random tail.
        }
        3 => {
            play(npc, "Idle");
            if random.bounded(10) == 0 {
                npc.state = 4;
                let speed = random.uniform() * 5.0 + 5.0;
                let dip = npc.dip.as_ref().unwrap();
                let direction = if dip.entity_flags & 0x800 != 0 {
                    //494cc5/6129d0 computes own position minus target position.
                    let delta = WorldVector {
                        x: npc.position.x - world.player.x,
                        y: npc.position.y - world.player.y,
                    };
                    let length = (delta.x * delta.x + delta.y * delta.y).sqrt();
                    if length > 0.0 {
                        delta * (1.0 / length)
                    } else {
                        delta
                    }
                } else {
                    crate::npcs::direction(random)
                };
                let friction = dip.current_friction;
                if friction != 0.0 {
                    let impulse = direction * speed;
                    npc.velocity = npc.velocity
                        + WorldVector {
                            x: impulse.x / friction,
                            y: impulse.y / friction,
                        };
                }
            }
            npc.dip.as_mut().unwrap().current_friction *= 0.8;
        }
        4 => {
            play(npc, "Move");
            npc.dip.as_mut().unwrap().current_friction *= 0.95;
            //612950 is squared length; it consumes no square root or draw.
            if npc.velocity.x * npc.velocity.x + npc.velocity.y * npc.velocity.y < 1.0 {
                npc.state = 3;
            }
        }
        _ => {} //4949e3: all other nonzero states execute the common tail.
    }
    npc.dip.as_mut().unwrap().flip_x = npc.velocity.x < 0.0; //4949ef reads464 (X).
    if random.bounded(10) == 0 {
        let scale = random.uniform() * 0.5 + 0.2;
        let seed = random.next_u32();
        let id = world
            .effects
            .admit_blood(seed, npc.position, [1.0; 4], [0.0; 3])?;
        world.effects.set_blood_scale(id, scale)?;
        world.effects.advance_blood_initial(id, random)?;
        //494a6d calls virtual58 AFTER480fe0's immediate7 callback. The white
        //initial alpha draw therefore remains in the real shared stream.
        let dip = npc.dip.as_ref().unwrap();
        world.effects.set_blood_color(id, dip.tint, dip.offsets)?;
    }
    if npc.damage.pending_damage() > 0.0 && random.bounded(2) == 0 {
        world.audio.admit(
            random,
            world.system_counter,
            SoundRequest {
                id: 60,
                volume: 0.8,
                delay: 15,
                looping: false,
                pitch: 1.0,
            },
        );
    }
    Ok(())
}

fn advance_sprite(npc: &mut Npc) {
    if npc.animation_playing {
        let frames = match npc.animation.as_str() {
            "Idle" => content::DIP_IDLE_FRAMES,
            "Move" => content::DIP_MOVE_FRAMES,
            _ => unreachable!("only authored Dip clips are selected"),
        };
        let next = npc.animation_clock + 1.0;
        npc.animation_clock = if next.floor() < frames as f32 {
            next
        } else {
            0.0
        };
        npc.animation_frame = npc.animation_clock.floor() as u32;
    }
}

/// First dead31f transition4464c0 calls445c60 before the next FIFO record.
fn die(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld<'_>,
    births: &mut Vec<Birth>,
) -> Result<(), String> {
    //47FD22 Have234 (Infestation2): unless the NPC's +360 records already
    //hold a 3.73, 50C980 spawns a BlueSpider at its position before 4464C0.
    //Port gap (#295): no BlueSpider familiar exists, so none is spawned and
    //no substitute is admitted; the birth records the unported factory.
    if world
        .player_items
        .is_some_and(|items| items.owned_counts[234] != 0)
    {
        births.push(Birth::UnportedBlueSpider);
    }
    if npc.dip.as_ref().unwrap().status_countdown > 0 {
        return Err(
            "Dip death with positive Entity400 requires the actual status/SFX137 owner".into(),
        );
    }
    //47FC80 owns the item effect BEFORE4464D3 sets dead31F and before
    //445C60's SFX/gib draws. Use the persistent Entity RNG, not shared MT.
    let drops_half_heart = crate::item_death_effects::gimpy_half_heart(
        world
            .player_items
            .ok_or("Gimpy callback Player owner absent")?,
        npc.max_hp,
        &mut npc.dip.as_mut().unwrap().entity_rng,
    )?;
    if drops_half_heart {
        let seed = random.next_u32();
        world.item_death_pickups.admit_half_heart(
            seed,
            npc.position,
            world.room_type,
            world.effects.scheduler,
        )?;
    }
    npc.dead = true;
    let dip = npc.dip.as_ref().unwrap();
    world.audio.admit(
        random,
        world.system_counter,
        SoundRequest {
            id: 30,
            volume: 1.0,
            delay: 2,
            looping: false,
            pitch: 1.0,
        },
    );
    // This caller is admitted by the existing normal-run menu. Challenge23
    // and other death modifiers require their actual run/actor owner.
    let counts = gib_counts::sample(
        CONFIG.gib_amount,
        CONFIG.gib_flags,
        AmountScale::Ordinary,
        random,
    )?;
    for _ in 0..counts[5] {
        let speed = random.uniform() * 4.0 + 2.0;
        let seed = random.next_u32();
        let direction = crate::npcs::direction(random);
        let velocity = WorldVector {
            x: direction.x,
            y: direction.y * 0.5,
        } * speed;
        let offset = (crate::npcs::direction(random) * 0.5) * npc.collision_radius;
        let mut admission = poop_effects::Admission {
            game_update_count: world.effects.game_update_count,
            next_effect: world.effects.next_effect,
            effects: world.effects.effects,
            scheduler: world.effects.scheduler,
        };
        let id = admission.admit_particle(poop_effects::ParticleBirth {
            seed,
            position: npc.position + offset,
            velocity,
            spawner: Some(Parent {
                actor: ActorRef {
                    kind: ActorKind::Npc,
                    id: npc.id,
                },
                entity_type: 217,
                variant: npc.variant,
            }),
        })?;
        admission.advance_particle_initial(id, random)?;
    }
    if dip.entity_flags & 0x10000 == 0 {
        let seed = random.next_u32();
        npc.death_effect_seed = Some(seed);
        //446287 copies the inherited color BEFORE the explicit7 Update.
        let id = world
            .effects
            .admit_blood(seed, npc.position, dip.tint, dip.offsets)?;
        world.effects.advance_blood_initial(id, random)?;
    }
    Ok(())
}

fn collider() -> &'static Collider {
    static COLLIDER: OnceLock<Collider> = OnceLock::new();
    COLLIDER.get_or_init(|| {
        Collider::new(CollisionGeometry {
            radius: CONFIG.collision_radius,
            scale: WorldVector { x: 1.0, y: 1.0 },
            points: CONFIG.grid_points,
        })
        .expect("authored Dip collision geometry")
    })
}

pub(crate) fn post_contact_grid(npc: &mut Npc, grid: &Grid) -> Result<(), String> {
    collider().resolve_nonplayer_ground_bouncy(
        grid,
        &mut npc.position,
        &mut npc.velocity,
        false,
    )?;
    npc.presented_position = npc.position;
    Ok(())
}

pub(crate) fn advance(
    npc: &mut Npc,
    phase: Phase,
    random: &mut Mt19937,
    world: &mut NpcWorld<'_>,
    births: &mut Vec<Birth>,
) -> Result<(), String> {
    world
        .player_items
        .ok_or("Dip death requires actual Player inventory")?;
    if npc.removed || phase == Phase::Skipped {
        return Ok(());
    }
    if phase == Phase::Interpolate {
        let dip = npc.dip.as_mut().unwrap();
        if dip.entity_flags & 2 != 0 {
            dip.entity_flags &= !2;
            return Ok(());
        }
        let mut velocity = npc.velocity;
        npc.presented_position = npc.position + (velocity * dip.current_friction) * 0.5;
        collider().resolve_nonplayer_ground_bouncy(
            world.grid,
            &mut npc.presented_position,
            &mut velocity,
            true,
        )?;
        return Ok(());
    }
    if npc.dip.as_ref().unwrap().status_countdown > 0 {
        return Err("Dip Entity400 countdown/expiry needs its actual status callback".into());
    }
    if npc.state == 1 {
        npc.dip.as_mut().unwrap().current_friction *= 0.01;
        match npc.frame_age {
            0 => {
                npc.state = 0;
                ai(npc, random, world)?;
                npc.saved_initialized_state = npc.state;
                npc.state = 1;
                npc.visible = false;
            }
            1 => {
                let seed = random.next_u32();
                npc.appearance_effect_seed = Some(seed);
                world.effects.admit_npc(npc, 1, seed)?;
            }
            4 => npc.visible = true, // Dip has no authored Appear clip.
            age if age > 19 => {
                npc.state = npc.saved_initialized_state;
                npc.dip.as_mut().unwrap().current_friction = npc.base_friction;
            }
            _ => {}
        }
        npc.frame_age = npc.frame_age.wrapping_add(1);
        if npc.state != 1 || npc.dead {
            ai(npc, random, world)?;
        }
    } else {
        //4949c0 has no dead31f early return: the next full generic callback
        //still runs the family tail before4fc0a3/4fb652 retirement.
        ai(npc, random, world)?;
    }
    if npc.dead {
        //4fc0a3: family AI has finished; factory gets physicalP+458, V0,
        //NULL spawner and no initial Update. The captured old tail stays fixed.
        let seed = random.next_u32();
        let position = npc.position + npc.dip.as_ref().unwrap().render_offset;
        poop_effects::Admission {
            game_update_count: world.effects.game_update_count,
            next_effect: world.effects.next_effect,
            effects: world.effects.effects,
            scheduler: world.effects.scheduler,
        }
        .admit_explosion(seed, position)?;
        npc.removed = true;
    }
    advance_sprite(npc);
    let dip = npc.dip.as_mut().unwrap();
    npc.velocity = npc.velocity * dip.current_friction;
    npc.position = npc.position + npc.velocity;
    npc.presented_position = npc.position;
    dip.current_friction = npc.base_friction;
    //44773a skips remaining records once the actual death byte is set. All
    //ordered flags currently represented by DamageKind use ordinary HP<0.
    for index in 0..npc.damage.events().len() {
        let amount = npc.damage.events()[index].amount;
        if !npc.dead && npc.max_hp > 0.0 {
            npc.hp -= amount;
            if npc.hp < 0.0 {
                die(npc, random, world, births)?;
            }
        }
    }
    npc.damage.clear();
    if !npc.position.is_finite() || !npc.velocity.is_finite() || !npc.hp.is_finite() {
        return Err("Dip source state overflowed".into());
    }
    Ok(())
}

pub(crate) const FAMILY: Family = Family {
    kinds: &[217],
    spawn: spawn_family,
    advance,
    post_contact_grid,
    entity_flags: family_flags,
    occupancy: crate::entities::Occupancy::None,
};

fn spawn_family(
    id: u32,
    selector: &SpawnSelector,
    context: &SpawnContext,
) -> Result<Option<Npc>, String> {
    let mut npc = from_selector(id, selector, context.stage)?;
    if let Some(dip) = npc.as_mut().and_then(|npc| npc.dip.as_mut()) {
        dip.birth_game_update = context.game_update_count;
    }
    Ok(npc)
}

fn family_flags(npc: &Npc) -> u32 {
    npc.dip.as_ref().map_or(4, |dip| dip.entity_flags)
}
