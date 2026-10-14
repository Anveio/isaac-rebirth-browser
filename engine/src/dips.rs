//! Ordinary Dip217: source4949c0, missing-Appear staging and Blood7 trail.
use crate::{
    audio::{AudioState, SoundRequest},
    combat, content,
    familiars::NpcEffectAdmission,
    grid::{Collider, CollisionGeometry, Grid},
    motion::WorldVector,
    mt::Mt19937,
    npcs::Npc,
    phase::Phase,
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
}

pub(crate) struct World<'a> {
    pub player: WorldVector,
    pub grid: &'a Grid,
    pub audio: &'a mut AudioState,
    pub system_counter: u32,
    pub effects: NpcEffectAdmission<'a>,
}

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
    //446b18..446b3f: stage is capped before the later-stage f32 factor.
    let scale = stage.min(10) as f32 * if stage > 4 { 0.8 } else { 1.0 };
    let hp = scale * content::NPC_217_STAGEHP + content::NPC_217_BASEHP;
    Ok(Some(Npc {
        id,
        kind: 217,
        variant: 0,
        subtype: 0,
        spawn_seed: selector.spawn_seed,
        position: selector.position,
        presented_position: selector.position,
        velocity: ZERO,
        target: ZERO,
        wander_velocity: ZERO,
        hp,
        max_hp: hp,
        collision_radius: content::NPC_217_COLLISIONRADIUS,
        collision_mass: content::NPC_217_COLLISIONMASS,
        collision_damage: content::NPC_217_COLLISIONDAMAGE,
        base_friction: content::NPC_217_FRICTION,
        shadow_size: content::NPC_217_SHADOWSIZE,
        grid_collision_class: content::NPC_217_GRID_CLASS,
        entity_collision_class: 4,
        can_shut_doors: content::NPC_217_CAN_SHUT_DOORS,
        boss: content::NPC_217_BOSS,
        state: 1,
        frame_age: 0,
        animation_frame: 0,
        animation: "Idle".into(),
        animation_clock: 0.0,
        animation_playing: false, // Config47f573 SetFrame(default,0).
        visible: true,
        saved_initialized_state: 0,
        appearance_effect_seed: None,
        death_effect_seed: None,
        dead: false,
        removed: false,
        damage: combat::DamageQueue::default(),
        keeper: None,
        fireplace: None,
        monstro: None,
        level2_fly: None,
        dip: Some(DipState {
            entity_flags: 4,
            birth_game_update: 0,
            current_friction: content::NPC_217_FRICTION,
            flip_x: false,
            tint: [1.0; 4],
            offsets: [0.0; 3],
        }),
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

fn ai(npc: &mut Npc, random: &mut Mt19937, world: &mut World<'_>) -> Result<(), String> {
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

fn collider() -> &'static Collider {
    static COLLIDER: OnceLock<Collider> = OnceLock::new();
    COLLIDER.get_or_init(|| {
        Collider::new(CollisionGeometry {
            radius: content::NPC_217_COLLISIONRADIUS,
            scale: WorldVector { x: 1.0, y: 1.0 },
            points: content::NPC_217_GRID_POINTS,
        })
        .expect("authored Dip collision geometry")
    })
}

pub(crate) fn post_contact_grid(npc: &mut Npc, grid: &Grid) -> Result<(), String> {
    collider().resolve_nonplayer_ground(grid, &mut npc.position, &mut npc.velocity, false)?;
    npc.presented_position = npc.position;
    Ok(())
}

pub(crate) fn advance(
    npc: &mut Npc,
    phase: Phase,
    random: &mut Mt19937,
    world: &mut World<'_>,
) -> Result<(), String> {
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
        collider().resolve_nonplayer_ground(
            world.grid,
            &mut npc.presented_position,
            &mut velocity,
            true,
        )?;
        return Ok(());
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
        //Real Poop43 admission before Remove remains required and unimplemented.
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
    for event in npc.damage.events() {
        if !npc.dead && npc.max_hp > 0.0 {
            npc.hp -= event.amount;
            if npc.hp < 0.0 {
                npc.dead = true;
                //445c60's five-poop-gib58 + Blood7 death consumer is a tracked
                //required gap. Never substitute Fly's zero-gib Splat or draws.
            }
        }
    }
    npc.damage.clear();
    if !npc.position.is_finite() || !npc.velocity.is_finite() || !npc.hp.is_finite() {
        return Err("Dip source state overflowed".into());
    }
    Ok(())
}
