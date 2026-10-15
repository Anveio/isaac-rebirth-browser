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
    npcs::{Npc, SpriteEvents},
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

/// Sprite.Play40fd80: a forced play restarts; otherwise only a different
/// clip, or the same looping clip while stopped (+124 clear, e.g. after
/// SetFrame), restarts. A finished nonlooping clip stays finished.
pub(crate) fn play(npc: &mut Npc, name: &str, force: bool) {
    let stopped_loop =
        !npc.animation_playing && content::clip(&npc.actor, name).is_some_and(|clip| clip.looping);
    if force || npc.animation != name || stopped_loop {
        npc.animation = name.into();
        npc.animation_frame = 0;
        npc.animation_clock = 0.0;
        npc.animation_playing = true;
        // 40f8a0 restart clears both event masks, then scans frame 0.
        let now = triggers_at(npc, 0);
        npc.sprite_events = SpriteEvents { now, seen: now };
    }
}

/// 40e9b0: the current clip's triggers authored at `frame`, as mask bits.
fn triggers_at(npc: &Npc, frame: u32) -> u32 {
    content::clip(&npc.actor, &npc.animation).map_or(0, |clip| {
        clip.events
            .iter()
            .enumerate()
            .filter(|(_, (_, at))| *at == frame)
            .fold(0, |mask, (bit, _)| mask | (1 << bit))
    })
}

fn event_mask(npc: &Npc, event: &str) -> u32 {
    content::clip(&npc.actor, &npc.animation).map_or(0, |clip| {
        clip.events
            .iter()
            .enumerate()
            .filter(|(_, (name, _))| *name == event)
            .fold(0, |mask, (bit, _)| mask | (1 << bit))
    })
}

/// IsEventTriggered410790: the last sprite step landed on the event's frame.
pub(crate) fn event_triggered(npc: &Npc, event: &str) -> bool {
    npc.sprite_events.now & event_mask(npc, event) != 0
}

/// WasEventTriggered4107b0: the event's frame was reached since the restart.
pub(crate) fn event_was_triggered(npc: &Npc, event: &str) -> bool {
    npc.sprite_events.seen & event_mask(npc, event) != 0
}

/// Sprite.SetFrame40fef0: select the clip, seek the frame and stop.
pub(crate) fn set_frame(npc: &mut Npc, name: &str, frame: u32) {
    npc.animation = name.into();
    npc.animation_frame = frame;
    npc.animation_clock = frame as f32;
    npc.animation_playing = false;
}

/// anim_walkframe4807e0 for unchampioned walkers: below `threshold` speed the
/// vertical clip holds frame 0; otherwise the dominant axis picks the clip and
/// horizontal walking faces the velocity.
pub(crate) fn walk_frame(npc: &mut Npc, horizontal: &str, vertical: &str, threshold: f32) {
    let speed = (npc.velocity.x * npc.velocity.x + npc.velocity.y * npc.velocity.y).sqrt();
    if threshold > speed {
        set_frame(npc, vertical, 0);
    } else if npc.velocity.x.abs() > npc.velocity.y.abs() {
        play(npc, horizontal, false);
        npc.flip_x = npc.velocity.x < 0.0;
    } else {
        play(npc, vertical, false);
    }
}

/// 4fc3d6: a boss plays "Death"; Larry Jr. (4fc8e8) picks DeathBody01 for
/// a segment with a parent and DeathHead otherwise.
fn death_clip(npc: &Npc) -> &'static str {
    if content::clip(&npc.actor, "Death").is_some() {
        "Death"
    } else if npc.parent.is_some() {
        "DeathBody01"
    } else {
        "DeathHead"
    }
}

/// A dead boss plays its death clip in state16 before 4f2d10 removes it.
fn dying_boss(npc: &mut Npc, mut friction: f32) -> Result<(), String> {
    let mut frame = npc.death_frame;
    if crate::boss_death::step(npc, &mut frame, &mut friction)? == 0 {
        let clip = death_clip(npc);
        play(npc, clip, false);
    }
    npc.death_frame = frame;
    update_sprite(npc)?;
    npc.friction = friction;
    npc.velocity = npc.velocity * friction;
    npc.position = npc.position + npc.velocity;
    npc.presented_position = npc.position;
    crate::boss_death::finish_if_done(npc);
    Ok(())
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
        // 40ea30 scans each newly reached integer frame.
        let now = triggers_at(npc, next.floor() as u32);
        npc.sprite_events.now = now;
        npc.sprite_events.seen |= now;
    } else if clip.looping {
        npc.animation_clock = 0.0;
        let now = triggers_at(npc, 0);
        npc.sprite_events = SpriteEvents { now, seen: now };
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
    let appear = !npc.no_appear;
    advance_with(npc, phase, random, world, births, ai, appear)
}

/// `advance` for a family whose state0 initializer may clear the APPEAR flag
/// (4): with `appear` false, 4f9a46 skips the hide/save, so the actor stays
/// visible, never draws the age-1 Poof seed and runs its AI a second time in
/// the age-0 update.
pub(crate) fn advance_with(
    npc: &mut Npc,
    phase: Phase,
    random: &mut Mt19937,
    world: &mut NpcWorld<'_>,
    births: &mut Vec<Birth>,
    ai: Ai,
    appear: bool,
) -> Result<(), String> {
    advance_full(npc, phase, random, world, births, ai, appear, false)
}

/// `advance` for a boss whose AI handles its own death: Entity_NPC::Update
/// calls update_ai once more with dead set (the base kill follows the AI),
/// and only the post-AI handling enters state16 in that same update.
pub(crate) fn advance_dead_ai(
    npc: &mut Npc,
    phase: Phase,
    random: &mut Mt19937,
    world: &mut NpcWorld<'_>,
    births: &mut Vec<Birth>,
    ai: Ai,
) -> Result<(), String> {
    advance_full(npc, phase, random, world, births, ai, true, true)
}

#[expect(clippy::too_many_arguments, reason = "the shared appear and dead-AI switches of advance_with and advance_dead_ai")]
fn advance_full(
    npc: &mut Npc,
    phase: Phase,
    random: &mut Mt19937,
    world: &mut NpcWorld<'_>,
    births: &mut Vec<Birth>,
    ai: Ai,
    appear: bool,
    dead_ai: bool,
) -> Result<(), String> {
    if npc.removed {
        return Ok(());
    }
    if phase != Phase::Update {
        npc.presented_position = npc.position + npc.velocity * 0.5;
        return Ok(());
    }
    let mut friction = npc.base_friction;
    if npc.dead {
        if !npc.boss {
            // Dead NPCs keep negative HP through the next intermediate phase
            // and retire at the following full pass.
            npc.removed = true;
            return Ok(());
        }
        if dead_ai && npc.state != 16 {
            ai(npc, random, world, births, &mut friction)?;
        }
        return dying_boss(npc, friction);
    }
    if npc.state == 1 {
        friction *= 0.01;
        match npc.frame_age {
            0 => {
                // 4f9a2e runs update_ai with state0 as the AI initializer,
                // then 4fb1d4 hides the actor and saves the chosen state.
                npc.state = 0;
                ai(npc, random, world, births, &mut friction)?;
                if appear {
                    npc.saved_initialized_state = npc.state;
                    npc.state = 1;
                    npc.visible = false;
                }
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
    if npc.dead && npc.boss {
        // A boss that killed itself in its AI (Larry Jr.'s lone head) enters
        // state16 in the same update's post-AI handling.
        return dying_boss(npc, friction);
    }
    update_sprite(npc)?;
    npc.friction = friction;
    npc.velocity = npc.velocity * friction;
    npc.position = npc.position + npc.velocity;
    npc.presented_position = npc.position;
    if npc.invincible {
        // BaseUpdate447729: an invincible actor (+494) discards its queue.
        npc.damage.clear();
    }
    let mut health = npc.health();
    let applied = npc.damage.advance(&mut health, phase)?;
    npc.advance_sprite_color(phase, &applied);
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
pub(crate) fn on_frame(npc: &Npc, world: &NpcWorld, period: u32) -> Result<bool, String> {
    let sequence = world
        .effects
        .scheduler
        .sequence_id(crate::actors::ActorRef {
            kind: crate::actors::ActorKind::Npc,
            id: npc.id,
        })
        .ok_or("NPC frame gate requires its manager sequence")?;
    Ok(world.game_counter.wrapping_add(sequence) % period == 0)
}

/// FireProjectiles49ebc0 modes admitted so far.
#[derive(Clone, Copy)]
pub(crate) enum ShotMode {
    /// Mode0: one shot along the aim.
    Single,
    /// Mode1 (49f1c5): two shots at the aim rotated by about ±16 degrees.
    Pair,
}

pub(crate) struct ShotParameters {
    pub mode: ShotMode,
    pub range: f32,
    pub cooldown: i32,
    pub delay: i32,
    pub speed: f32,
    /// Default ProjectileParams47eaf0 angle, drawn by the caller every update.
    pub parameter_angle: f32,
}

/// try_shoot_player4ba430 with a nonpredictive FireProjectiles shot. Returns whether this update chose to shoot; the shot itself follows
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
        let aim = direction * shot.speed;
        let velocities = match shot.mode {
            ShotMode::Single => vec![aim],
            ShotMode::Pair => {
                // 49f1c5: cos 0.961, sin 0.276, spread params+0x24 = 1.0.
                let (cos, sin, spread) = (0.961_f32, 0.276_f32, 1.0_f32);
                vec![
                    WorldVector {
                        x: aim.x * cos - aim.y * sin * spread,
                        y: sin * aim.x * spread + cos * aim.y,
                    },
                    WorldVector {
                        x: aim.x * cos + aim.y * sin * spread,
                        y: cos * aim.y - sin * aim.x * spread,
                    },
                ]
            }
        };
        for velocity in velocities {
            // Each shot draws its spawn seed, then Init its falling speed.
            let spawn_seed = random.next_u32();
            let initial_falling_speed = (random.uniform() - 0.5) * 0.2;
            births.push(Birth::Shot(Shot {
                position: npc.position,
                velocity,
                spawn_seed,
                initial_falling_speed,
                parameter_angle: shot.parameter_angle,
            }));
        }
    }
    Ok(chosen)
}
