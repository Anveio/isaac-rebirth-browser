//! Source Entity_NPC::Update4f98e0 for authored NPCs without a subclass.
//! A family supplies only its AI routine (the update_ai4f9670 target); this
//! base owns the generic appear schedule, sprite clock, BaseUpdate motion,
//! FIFO damage drain, death and next-full retirement.
use crate::{
    audio::SoundRequest,
    content,
    entities::{Birth, NpcWorld},
    motion::WorldVector,
    mt::Mt19937,
    npcs::Npc,
    phase::Phase,
    roundworm::Shot,
};
use serde::{Deserialize, Serialize};

/// Entity_NPC projectile cooldown+f68 and delay+f6c.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShooterTimers {
    pub cooldown: i32,
    pub delay: i32,
}

impl Default for ShooterTimers {
    /// Init47ee0f/47ee19: no cooldown and no pending shot.
    fn default() -> Self {
        Self {
            cooldown: 0,
            delay: -1,
        }
    }
}

/// A family's update_ai target. It may scale this update's friction.
pub(crate) type Ai =
    fn(&mut Npc, &mut Mt19937, &mut NpcWorld, &mut Vec<Birth>, &mut f32) -> Result<(), String>;

/// Generic4fbe4a Poof15 subtype: radius>=18 is large; otherwise radius>=13
/// is medium. Unscaled authored sprites keep +30c below the 30 threshold.
fn poof_subtype(npc: &Npc) -> u32 {
    if npc.collision_radius >= 18.0 {
        3
    } else if npc.collision_radius >= 13.0 {
        2
    } else {
        1
    }
}

/// Sprite.Play40fd80: a forced play restarts; otherwise only a different or
/// stopped clip restarts.
pub(crate) fn play(npc: &mut Npc, name: &str, force: bool) {
    if force || npc.animation != name || !npc.animation_playing {
        npc.animation = name.into();
        npc.animation_frame = 0;
        npc.animation_clock = 0.0;
        npc.animation_playing = true;
    }
}

/// IsFinished40fba0 for the current clip.
pub(crate) fn finished(npc: &Npc) -> bool {
    !npc.animation_playing
}

fn update_sprite(npc: &mut Npc) -> Result<(), String> {
    if !npc.animation_playing {
        return Ok(());
    }
    let clip = content::clip(&npc.actor, &npc.animation)
        .ok_or_else(|| format!("{} has no authored clip {}", npc.actor, npc.animation))?;
    let next = npc.animation_clock + 1.0;
    if next.floor() < clip.frames as f32 {
        npc.animation_clock = next;
    } else if clip.looping {
        npc.animation_clock = 0.0;
    } else {
        npc.animation_clock = (clip.frames - 1) as f32;
        npc.animation_playing = false;
    }
    npc.animation_frame = npc.animation_clock.floor() as u32;
    Ok(())
}

pub(crate) fn advance(
    npc: &mut Npc,
    phase: Phase,
    random: &mut Mt19937,
    world: &mut NpcWorld<'_>,
    births: &mut Vec<Birth>,
    ai: Ai,
) -> Result<(), String> {
    if npc.removed {
        return Ok(());
    }
    if phase != Phase::Update {
        npc.presented_position = npc.position + npc.velocity * 0.5;
        return Ok(());
    }
    if npc.dead {
        // Dead NPCs keep negative HP through the next intermediate phase and
        // retire at the following full pass.
        npc.removed = true;
        return Ok(());
    }
    let mut friction = npc.base_friction;
    if npc.state == 1 {
        friction *= 0.01;
        match npc.frame_age {
            0 => {
                // 4f9a2e runs update_ai with state0 as the AI initializer,
                // then 4fb1d4 hides the actor and saves the chosen state.
                npc.state = 0;
                ai(npc, random, world, births, &mut friction)?;
                npc.saved_initialized_state = npc.state;
                npc.state = 1;
                npc.visible = false;
            }
            1 => {
                let seed = random.next_u32();
                npc.appearance_effect_seed = Some(seed);
                world.effects.admit_npc(npc, poof_subtype(npc), seed)?;
            }
            4 => {
                npc.visible = true;
                if content::clip(&npc.actor, "Appear").is_some() {
                    play(npc, "Appear", true);
                }
            }
            age if age > 19 && (npc.animation != "Appear" || finished(npc)) => {
                npc.state = npc.saved_initialized_state;
                friction = npc.base_friction;
            }
            _ => {}
        }
        npc.frame_age = npc
            .frame_age
            .checked_add(1)
            .ok_or("NPC appearance counter exhausted")?;
        if npc.state != 1 {
            ai(npc, random, world, births, &mut friction)?;
        }
    } else {
        ai(npc, random, world, births, &mut friction)?;
    }
    update_sprite(npc)?;
    npc.velocity = npc.velocity * friction;
    npc.position = npc.position + npc.velocity;
    npc.presented_position = npc.position;
    let mut health = npc.health();
    npc.damage.advance(&mut health, phase)?;
    npc.hp = health.hp;
    if npc.hp < 0.0 {
        npc.dead = true;
        // Source44641d death sound precedes the real Blood Splat seed. Gib
        // particles for authored gib counts are not yet admitted.
        world.audio.admit(
            random,
            world.system_counter,
            SoundRequest {
                id: 30,
                volume: 0.8,
                delay: 2,
                looping: false,
                pitch: 1.0,
            },
        );
        let seed = random.next_u32();
        npc.death_effect_seed = Some(seed);
        births.push(Birth::Blood(seed));
    }
    if !npc.position.is_finite() || !npc.velocity.is_finite() {
        return Err("NPC source motion overflowed actual world vectors".into());
    }
    Ok(())
}

/// is_frame444890: Game+ce9f4 plus the actor's manager sequence, modulo n.
fn on_frame(npc: &Npc, world: &NpcWorld, period: u32) -> Result<bool, String> {
    let sequence = world
        .effects
        .scheduler
        .sequence_id(crate::actors::ActorRef {
            kind: crate::actors::ActorKind::Npc,
            id: npc.id,
        })
        .ok_or("NPC frame gate requires its manager sequence")?;
    // Native Game+ce9f4 reads one more than game_update_count: every paired
    // entity-horf capture shows fullGameUpdates = gameUpdateCount + 1 (the
    // run-loader origin recorded as absoluteCounterPendingLoader).
    let native_counter = world.game_counter.wrapping_add(1);
    Ok(native_counter.wrapping_add(sequence) % period == 0)
}

pub(crate) struct ShotParameters {
    pub range: f32,
    pub cooldown: i32,
    pub delay: i32,
    pub speed: f32,
    /// Default ProjectileParams47eaf0 angle, drawn by the caller every update.
    pub parameter_angle: f32,
}

/// try_shoot_player4ba430 with a nonpredictive single FireProjectiles(mode0)
/// shot. Returns whether this update chose to shoot; the shot itself follows
/// once the delay counts down to zero.
pub(crate) fn try_shoot_player(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    shot: ShotParameters,
) -> Result<bool, String> {
    let delta = WorldVector {
        x: world.player.x - npc.position.x,
        y: world.player.y - npc.position.y,
    };
    let mut chosen = false;
    let gate = on_frame(npc, world, 7)?;
    let timers = npc
        .shooter
        .as_mut()
        .ok_or("shooting NPC requires its projectile timers")?;
    if gate {
        if timers.cooldown > 0 {
            timers.cooldown -= 1;
        } else if delta.x * delta.x + delta.y * delta.y < shot.range * shot.range
            && world.grid.projectile_line_clear(
                npc.position,
                world.player,
                world
                    .path_costs
                    .as_deref()
                    .ok_or("NPC line check requires source Room.Init path costs")?,
            )
        {
            timers.cooldown = shot.cooldown;
            timers.delay = shot.delay;
            chosen = true;
        }
    }
    if timers.delay > 0 {
        timers.delay -= 1;
    } else if timers.delay == 0 {
        timers.delay = -1;
        let length = (delta.x * delta.x + delta.y * delta.y).sqrt();
        let direction = if length > 0.0 {
            delta * (1.0 / length)
        } else {
            delta
        };
        let spawn_seed = random.next_u32();
        let initial_falling_speed = (random.uniform() - 0.5) * 0.2;
        births.push(Birth::Shot(Shot {
            position: npc.position,
            velocity: direction * shot.speed,
            spawn_seed,
            initial_falling_speed,
            parameter_angle: shot.parameter_angle,
        }));
    }
    Ok(chosen)
}
