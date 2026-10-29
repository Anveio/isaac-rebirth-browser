//! Shared Familiar behaviour primitives, each ported from the original
//! function Update 4704A0's cases call: Follow 466B50, Orbit 466DB0 and its
//! layer respacing 466780, Shooter 4702E0 and Fire 467170. A case of the
//! Update switch is a composition of these (familiar_variants::UpdateCase).
use crate::{
    actors::{ActorKind, ActorRef},
    familiar_variants::{self, FireParams},
    familiars::Familiar,
    motion::WorldVector,
    State,
};

fn held(state: &State, item: usize) -> bool {
    state
        .player_items
        .as_ref()
        .is_some_and(|owner| owner.owned_counts.get(item).copied().unwrap_or(0) > 0)
}

/// 466B50(fast): follow the first eligible familiar ahead (+498), or the
/// Player. Past 20 units the position moves (d - 20)/d * 0.1 of the gap
/// (halved when `fast`, the FastUpdate form); only the full form then
/// multiplies friction +470 by 0.8.
pub(crate) fn follow(state: &mut State, index: usize, fast: bool) -> Result<(), String> {
    let mut ahead = state.familiars[index].follow_parent;
    let mut target = None;
    while let Some(id) = ahead {
        let parent = state
            .familiars
            .iter()
            .find(|familiar| familiar.id == id)
            .ok_or("Familiar +498 names no Familiar")?;
        // 466B92..466C60: Little Chubby 3 and Bob's Brain 59 while their F58
        // runs, and Holy Water 25 of subtype (Entity+18) 1, are passed over.
        let skipped = (matches!(parent.variant, 3 | 59) && parent.fire_cooldown != 0)
            || (parent.variant == 25 && parent.subtype == 1);
        if !skipped {
            target = Some(parent.position);
            break;
        }
        ahead = parent.follow_parent;
    }
    let target = target.unwrap_or_else(|| crate::bombs::player_world(state));
    let familiar = &mut state.familiars[index];
    let delta = WorldVector {
        x: target.x - familiar.position.x,
        y: target.y - familiar.position.y,
    };
    let length = (delta.x * delta.x + delta.y * delta.y).sqrt();
    if length > 20.0 {
        let step = (length - 20.0) / length * 0.1;
        let step = delta * step;
        // 466D17: the fast (FastUpdate) step is halved (612B30).
        familiar.position = familiar.position
            + if fast {
                WorldVector {
                    x: step.x / 2.0,
                    y: step.y / 2.0,
                }
            } else {
                step
            };
    }
    if fast {
        // 466BEE: the fast form leaves friction alone.
        return if familiar.position.is_finite() {
            Ok(())
        } else {
            Err("Familiar follow overflowed".into())
        };
    }
    familiar.current_friction *= 0.8;
    if !familiar.position.is_finite() {
        return Err("Familiar follow overflowed".into());
    }
    Ok(())
}

/// 466780(layer, include_self): every live familiar of the Player on the
/// layer, in manager order (plus `index` itself when included), takes
/// phase i*2pi/n and the layer speed: 0x66CAC0 with Guardian Angel 112,
/// else 0x66CAD0, times 1/3 with Big Fan 279; Best Bud 60 triples it.
pub(crate) fn respace(state: &mut State, layer: usize, index: usize, include_self: bool) {
    let self_id = state.familiars[index].id;
    let mut members: Vec<usize> = state
        .actor_scheduler
        .full_phase()
        .into_iter()
        .filter(|actor| actor.kind == ActorKind::Familiar && actor.id != self_id)
        .filter_map(|actor| {
            state.familiars.iter().position(|familiar| {
                familiar.id == actor.id
                    && familiar.active
                    && !familiar.removed
                    && familiar.orbit_layer == layer as i32
            })
        })
        .collect();
    if include_self {
        members.push(index);
    }
    let factor = if held(state, 279) { 1.0_f32 / 3.0 } else { 1.0 };
    let speeds = if held(state, 112) {
        familiar_variants::ORBIT_SPEED_FAST
    } else {
        familiar_variants::ORBIT_SPEED
    };
    let count = members.len() as f32 as f64;
    for (i, member) in members.into_iter().enumerate() {
        let familiar = &mut state.familiars[member];
        familiar.orbit_phase = ((i as i32) as f64 * std::f64::consts::TAU / count) as f32;
        familiar.orbit_speed = factor * speeds[layer];
        if familiar.variant == 60 {
            familiar.orbit_speed *= 3.0;
        }
    }
}

/// 466DB0(center): the orbit point. The angle is
/// fmod(-F78 * (frame + parity*0.5) * pi * 0.5 + F6C, 2pi) in double
/// (x87 fprem), narrowed to f32, plus the layer offset 0x66CAA0. The
/// Manager's 60 Hz parity is 0 in every full Update. Sacrificial Dagger
/// 35 also holds Rotation frame 6 and takes the angle as its rotation
/// (466EF0).
pub(crate) fn orbit_point(
    state: &mut State,
    index: usize,
    center: WorldVector,
    parity: i32,
) -> WorldVector {
    let frame = crate::knives::source_game_counter(state.game_update_count) as i32 as f32;
    let familiar = &mut state.familiars[index];
    let turn = -familiar.orbit_speed * (frame + parity as f32 * 0.5);
    let angle = ((turn as f64 * std::f64::consts::PI * 0.5 + familiar.orbit_phase as f64)
        % std::f64::consts::TAU) as f32;
    if familiar.variant == 35 {
        // 466EF0: SetFrame("Rotation", 6) (paused), then layer 0's rotation.
        familiar.sprite.animation = "Rotation".into();
        familiar.sprite.frame = 6.0;
        familiar.sprite.playing = false;
        familiar.sprite.rotation = ((angle * 180.0) as f64 / std::f64::consts::PI) as f32;
    }
    let layer = familiar.orbit_layer.max(0) as usize;
    let angle = angle + familiar_variants::ORBIT_OFFSET[layer.min(3)];
    let (sin, cos) = (crate::fmath::sin(angle), crate::fmath::cos(angle));
    WorldVector {
        x: cos * familiar.orbit_radius.x,
        y: sin * familiar.orbit_radius.y,
    } + center
}

/// The orbital case 471759/470CC0: the position takes the orbit point
/// around the Player and the velocity is zeroed.
pub(crate) fn orbit(state: &mut State, index: usize) {
    let center = crate::bombs::player_world(state);
    let point = orbit_point(state, index, center, 0);
    let familiar = &mut state.familiars[index];
    familiar.position = point;
    familiar.velocity = WorldVector { x: 0.0, y: 0.0 };
}

/// FastUpdate 46F900: the live position takes the orbit point at the
/// Manager's odd 60 Hz parity; the velocity is left alone.
pub(crate) fn orbit_half(state: &mut State, index: usize) {
    let center = crate::bombs::player_world(state);
    let point = orbit_point(state, index, center, 1);
    state.familiars[index].position = point;
}

/// 467BA0 (Float*) and 4683B0 (FloatShoot*): direction 0 left (flipped
/// FloatSide), 1 up, 2 right, 3 down. Play keeps a clip already playing.
/// Robo-Baby 6 and Robo-Baby 2.0 53 (467BCC, 4683DC) face left with their
/// own Side2 clips instead of a flipped Side.
fn pose(familiar: &mut Familiar, direction: i32, shooting: bool) {
    let robo = matches!(familiar.variant, 6 | 53);
    let side = match direction {
        0 if robo => "Side2",
        0 | 2 => "Side",
        1 => "Up",
        3 => "Down",
        _ => return,
    };
    familiar.sprite.flip_x = direction == 0 && !robo;
    let clip = format!("Float{}{side}", if shooting { "Shoot" } else { "" });
    play(familiar, &clip);
}

/// 4702E0: F58 counts down. Without the Player's fire direction (+DA4) the
/// pose hold F5C runs out to the idle pose; with it and a spent F58, Fire
/// 467170 runs, F5C becomes 18 and F58 30. FloatShoot* shows while F5C > 8.
pub(crate) fn shoot(state: &mut State, index: usize) -> Result<(), String> {
    let mut direction = state.player_fire_direction;
    let familiar = &mut state.familiars[index];
    if familiar.fire_cooldown > 0 {
        familiar.fire_cooldown -= 1;
    }
    if direction == -1 {
        if familiar.shoot_hold >= 0 {
            familiar.shoot_hold -= 1;
            if familiar.shoot_hold == -1 {
                pose(familiar, 3, false);
            }
        }
        return Ok(());
    }
    // 4703C0: Abel shoots the opposite way.
    if familiar.variant == 8 {
        direction = (direction + 2) % 4;
    }
    if familiar.fire_cooldown <= 0 {
        let params = familiar_variants::fire_params(familiar.variant);
        if familiar.variant == 54 {
            if !rotten_fly(state, index)? {
                let familiar = &mut state.familiars[index];
                if familiar.shoot_hold > 8 {
                    pose(familiar, direction, true);
                    familiar.shoot_hold -= 1;
                } else {
                    pose(familiar, direction, false);
                    familiar.shoot_hold = 8;
                }
                return Ok(());
            }
        } else if !params.ported {
            familiar.unresolved_ai_updates = familiar.unresolved_ai_updates.saturating_add(1);
        } else {
            fire(state, index, direction, params)?;
        }
        let familiar = &mut state.familiars[index];
        familiar.shoot_hold = 18;
        // 47041A: Mongo Baby 74 reloads in 15.
        familiar.fire_cooldown = if familiar.variant == 74 { 15 } else { 30 };
    }
    let familiar = &mut state.familiars[index];
    if familiar.shoot_hold > 8 {
        pose(familiar, direction, true);
        familiar.shoot_hold -= 1;
    } else {
        pose(familiar, direction, false);
        familiar.shoot_hold = 8;
    }
    Ok(())
}

/// 467170: the direction times 10 plus 2 x 0.6 x the Player's velocity
/// (subtracted for Abel), renormalized to 10 below length 100; then each
/// tear of the variant's FireParams.
fn fire(state: &mut State, index: usize, direction: i32, params: FireParams) -> Result<(), String> {
    let aim = match direction {
        0 => WorldVector { x: -1.0, y: 0.0 },
        1 => WorldVector { x: 0.0, y: -1.0 },
        2 => WorldVector { x: 1.0, y: 0.0 },
        3 => WorldVector { x: 0.0, y: 1.0 },
        _ => return Err("Familiar fire direction outside 0..3".into()),
    };
    let carried = (crate::bombs::player_velocity(state) * 0.6_f32) * 2.0;
    let aim = aim * 10.0;
    let mut velocity = if state.familiars[index].variant == 8 {
        WorldVector {
            x: aim.x - carried.x,
            y: aim.y - carried.y,
        }
    } else {
        aim + carried
    };
    let length = (velocity.x * velocity.x + velocity.y * velocity.y).sqrt();
    if 100.0 > length {
        velocity = crate::knives::normalized(velocity) * 10.0;
    }
    // 467265: BFFS! 247 doubles the damage.
    let params = if params.rainbow {
        familiar_variants::rainbow_row(state.boot_lifecycle.shared.bounded(9) + 1)
    } else {
        params
    };
    let damage = params.damage * if held(state, 247) { 2.0 } else { 1.0 };
    let mut params = params;
    if params.charm_roll > 0 {
        let roll = state.boot_lifecycle.shared.bounded(params.charm_roll);
        if roll == 0 || params.charm_always {
            params.flags |= 0x2000;
        }
    }
    let origin = state.familiars[index].position;
    if params.double {
        // 467490: two tears at velocity +/- 0.2 of its perpendicular.
        let left = velocity
            + WorldVector {
                x: -velocity.y * 0.2,
                y: 0.2 * velocity.x,
            };
        let right = velocity
            + WorldVector {
                x: 0.2 * velocity.y,
                y: -velocity.x * 0.2,
            };
        for shot in [left, right] {
            crate::tear_factory::fire_familiar(state, origin, shot, damage, params)?;
        }
    } else {
        crate::tear_factory::fire_familiar(state, origin, velocity, damage, params)?;
    }
    Ok(())
}

/// Dry Baby 471401: when its Hit clip has finished, play Idle, use The
/// Necronomicon (522250(35, false, true, false)) and request its GiantBook
/// with no delay; then Follow, and F58 counts down.
pub(crate) fn dry_baby(state: &mut State, index: usize) -> Result<(), String> {
    let familiar = &mut state.familiars[index];
    if familiar.sprite.animation == "Hit" && !familiar.sprite.playing {
        familiar.sprite.animation = "Idle".into();
        familiar.sprite.frame = 0.0;
        familiar.sprite.playing = true;
        crate::active_items::nested_use_from(state, 35)?;
        // 4714AC: then GiantBook 5805C0(1, 0), the Necronomicon page.
        if !state.information_animation.request_authored(1, 0) {
            return Err("Necronomicon GiantBook definition absent".into());
        }
    }
    follow(state, index, false)?;
    let familiar = &mut state.familiars[index];
    if familiar.fire_cooldown > 0 {
        familiar.fire_cooldown -= 1;
    }
    Ok(())
}

fn squared(a: WorldVector, b: WorldVector) -> f32 {
    let (x, y) = (a.x - b.x, a.y - b.y);
    x * x + y * y
}

fn vulnerable(npc: &crate::npcs::Npc) -> Result<bool, String> {
    crate::target_eligibility::eligible_target(crate::target_eligibility::TargetEligibility {
        kind: npc.kind,
        collision_class: npc.entity_collision_class,
        raw_494: npc.raw_494,
        dead: npc.dead,
        entity_flags: crate::entities::family(npc.kind).map(|_| npc.entity_flags()),
    })
}

/// 4688D0(interval, radius): a target (+4A8) that stopped being vulnerable
/// (444810) is dropped; then, on the 444890(interval) frame and without a
/// target, QueryRadius 4536F0 over the enemy partition (mask 8, from the
/// last 4545D0 rebuild, which ran after the previous frame's actors) offers each
/// enemy whose circle reaches the point, and the last vulnerable one nearer
/// (squared) to the familiar or to the Player than the shrinking bound
/// (radius squared, then the smaller of its two distances) is taken.
fn find_target(state: &mut State, index: usize, interval: u32, radius: f32) -> Result<(), String> {
    if let Some(id) = state.familiars[index].target {
        let keep = match state
            .room
            .npcs
            .iter()
            .find(|npc| npc.id == id && !npc.removed)
        {
            Some(npc) => vulnerable(npc)?,
            None => false,
        };
        if !keep {
            state.familiars[index].target = None;
        }
    }
    let familiar = &state.familiars[index];
    let sequence = state
        .actor_scheduler
        .sequence_id(ActorRef {
            kind: ActorKind::Familiar,
            id: familiar.id,
        })
        .unwrap_or(0);
    let frame = crate::knives::source_game_counter(state.game_update_count);
    if !crate::familiar_contact::is_frame(frame, interval, sequence) || familiar.target.is_some() {
        return Ok(());
    }
    let position = familiar.position;
    let player = crate::bombs::player_world(state);
    let candidates = match state.enemy_query_partition.as_ref() {
        Some(snapshot) => snapshot.query(position, radius)?,
        None => Vec::new(),
    };
    let mut bound = radius * radius;
    let mut target = None;
    for actor in candidates {
        let Some(npc) = state.room.npcs.iter().find(|npc| npc.id == actor.id) else {
            continue;
        };
        let reach = radius + npc.collision_radius;
        if squared(position, npc.position) >= reach * reach || !vulnerable(npc)? {
            continue;
        }
        let near_self = squared(npc.position, position);
        let near_player = squared(npc.position, player);
        if bound > near_self || bound > near_player {
            bound = near_player.min(near_self);
            target = Some(npc.id);
        }
    }
    state.familiars[index].target = target;
    Ok(())
}

/// Whether the familiar's anm2 names `clip`.
fn has_clip(familiar: &Familiar, clip: &str) -> bool {
    crate::familiars::definition(3, familiar.variant)
        .is_some_and(|config| config.clips.iter().any(|known| known.name == clip))
}

/// ANM2 Play 40FD80(name, force): a name the anm2 lacks changes nothing
/// (Ball of Bandages L3/L4 have no Float* clips for the Shooter's poses);
/// without force a clip already set keeps running.
pub(crate) fn play_clip(familiar: &mut Familiar, clip: &str, force: bool) {
    if !has_clip(familiar, clip) || (!force && familiar.sprite.animation == clip) {
        return;
    }
    familiar.sprite.animation = clip.into();
    familiar.sprite.frame = 0.0;
    familiar.sprite.playing = true;
}

/// Sprite Play(name, false).
pub(crate) fn play(familiar: &mut Familiar, clip: &str) {
    play_clip(familiar, clip, false);
}

/// Chase 469A00 (Cube of Meat L3/L4 46/47, Leech 56, Ball of Bandages
/// L3/L4 71/72): 4688D0(13, 15 x damage + 65) picks a target; beyond 20
/// units of it (80 of the Player without one) the pathfinder 5B41E0 walks
/// there at 0.83 x (2.2 for 47 and 56, 1.8 otherwise). The Cube's L3/L4 leave a blood
/// splat on Random(4) = 0. Friction takes 0.85, then the walk clip by the
/// dominant velocity axis (Leech: Move/Attack by target).
pub(crate) fn chase(state: &mut State, index: usize) -> Result<(), String> {
    let damage = state.familiars[index].collision_damage;
    find_target(state, index, 13, 15.0 * damage + 65.0)?;
    let familiar = &state.familiars[index];
    let variant = familiar.variant;
    let (goal, threshold) = match familiar
        .target
        .and_then(|id| state.room.npcs.iter().find(|npc| npc.id == id))
    {
        Some(npc) => (npc.position, 400.0),
        None => (crate::bombs::player_world(state), 6400.0),
    };
    let mut attacking = false;
    if squared(goal, familiar.position) > threshold {
        // 469AC1: only the Cube's L4 (47) and the Leech (56) take 2.2.
        let speed = if matches!(variant, 47 | 56) { 2.2_f32 } else { 1.8 } * 0.83_f32;
        let grid = crate::grid::Grid::new(
            state
                .room
                .grid
                .clone()
                .ok_or("Familiar chase requires the room grid")?,
        )?;
        let sequence = state
            .actor_scheduler
            .sequence_id(ActorRef {
                kind: ActorKind::Familiar,
                id: familiar.id,
            })
            .ok_or("Familiar chase requires its manager sequence")?;
        let game_counter = state.native_game_frame();
        let room_type = state
            .room
            .descriptor
            .as_ref()
            .map_or(1, |descriptor| descriptor.template.room_type);
        let familiar = &mut state.familiars[index];
        let mut path = std::mem::replace(
            &mut familiar.pathfinder,
            crate::pathfinder::Pathfinder::new(),
        );
        let mut friction = familiar.current_friction;
        let result = crate::pathfinder::find_with(
            crate::pathfinder::PathOwner {
                position: familiar.position,
                velocity: &mut familiar.velocity,
                grid_collision_class: familiar.grid_collision_class,
                sight_frame: game_counter.wrapping_add(sequence) % 10 == 0,
            },
            &mut path,
            &mut state.boot_lifecycle.shared,
            &mut crate::pathfinder::PathRoom {
                game_counter,
                grid: &grid,
                path_costs: state.room.path_costs.as_mut(),
                room_type,
            },
            goal,
            speed,
            0,
            &mut friction,
        );
        familiar.pathfinder = path;
        familiar.current_friction = friction;
        result?;
        attacking = familiar.target.is_some();
    }
    if matches!(variant, 46 | 47) && state.boot_lifecycle.shared.bounded(4) == 0 {
        // 469DF2: Spawn(1000, 7) at the familiar, its scale Random*0.3 + 0.1,
        // then one explicit Update.
        let seed = state.boot_lifecycle.shared.next_u32();
        let position = state.familiars[index].position;
        let mut effects = crate::familiars::NpcEffectAdmission {
            game_update_count: state.game_update_count,
            next_effect: &mut state.next_effect,
            effects: &mut state.sprite_effects,
            scheduler: &mut state.actor_scheduler,
        };
        let id = effects.admit_blood(seed, position, [1.0; 4], [0.0; 3])?;
        let scale = state.boot_lifecycle.shared.uniform() * 0.3 + 0.1;
        effects.set_blood_scale(id, scale)?;
        effects.advance_blood_initial(id, &mut state.boot_lifecycle.shared)?;
    }
    let familiar = &mut state.familiars[index];
    familiar.current_friction *= 0.85;
    let velocity = familiar.velocity;
    if (velocity.x * velocity.x + velocity.y * velocity.y).sqrt() < 2.0 {
        if variant == 56 {
            // 40FED0: the Leech's clip holds at its first frame.
            familiar.sprite.frame = 0.0;
        } else {
            play(familiar, "Idle");
        }
        return Ok(());
    }
    if velocity.x.abs() > velocity.y.abs() {
        let clip = match (variant, attacking) {
            (56, true) => "Attack Hori",
            (56, false) => "Move Hori",
            _ => "WalkHori",
        };
        play(familiar, clip);
        familiar.sprite.flip_x = 0.0 > velocity.x;
    } else {
        let clip = match (variant, attacking, velocity.y > 0.0) {
            (56, true, true) => "Attack Down",
            (56, true, false) => "Attack Up",
            (56, false, true) => "Move Down",
            (56, false, false) => "Move Up",
            _ => "WalkVert",
        };
        play(familiar, clip);
    }
    Ok(())
}

/// 4545D0's mask-8 rebuild, after the full actor pass and before contacts:
/// every NPC body with a collision class (45483C/444800), at its position
/// then. PORT GAP: 454B53 also inserts bombs (type 4) into this partition;
/// they never pass the target search's NPC filter, but they can change a
/// bucket's order.
pub(crate) fn rebuild_enemy_partition(state: &mut State) -> Result<(), String> {
    let members: Vec<_> = state
        .actor_scheduler
        .full_phase()
        .into_iter()
        .filter(|actor| actor.kind == ActorKind::Npc)
        .filter_map(|actor| {
            let npc = state.room.npcs.iter().find(|npc| npc.id == actor.id)?;
            (npc.entity_collision_class != 0 && (10..=999).contains(&npc.kind)).then_some((
                actor,
                crate::contact_candidates::Bounds {
                    id: 0,
                    position: npc.position,
                    half_extents: WorldVector {
                        x: npc.collision_radius,
                        y: npc.collision_radius,
                    },
                },
            ))
        })
        .collect();
    state.enemy_query_partition = Some(crate::contact_candidates::QuerySnapshot::rebuild(
        state.game_update_count,
        &members,
    )?);
    Ok(())
}

/// 46DF10, for each familiar in manager order from Player 50A334: F90 rises,
/// then the variant's RoomClearCase. A drop takes FindFreePosition
/// 5C7F30(position, 0), a shared seed, Spawn 550A40(5, variant, subtype)
/// with the familiar as spawner, and Play("Spawn", force). Hearts with
/// Daemon's Tail (trinket 22) first need Random(5) == 0 (46E1D8).
pub(crate) fn room_clear(state: &mut State, index: usize) -> Result<(), String> {
    use crate::familiar_variants::{RoomClearCase, RoomClearPickup};
    let familiar = &mut state.familiars[index];
    familiar.rooms_cleared = familiar.rooms_cleared.wrapping_add(1);
    let cleared = familiar.rooms_cleared;
    let (divisor, wrap4, pickup, sound) = match familiar_variants::room_clear_case(familiar.variant) {
        RoomClearCase::None => return Ok(()),
        RoomClearCase::Unported(_) => {
            familiar.unresolved_ai_updates = familiar.unresolved_ai_updates.saturating_add(1);
            return Ok(());
        }
        RoomClearCase::Drop {
            divisor,
            wrap4,
            pickup,
            sound,
        } => (divisor, wrap4, pickup, sound),
    };
    if let Some(id) = sound {
        state.audio.admit(
            &mut state.boot_lifecycle.shared,
            state.system_clock.system_counter.wrapping_sub(1),
            crate::audio::SoundRequest {
                id,
                volume: 1.0,
                delay: 2,
                looping: false,
                pitch: 1.0,
            },
        );
    }
    let mut n = match divisor {
        Some(divisor) => (cleared as f32 / divisor) as i32,
        None => cleared,
    };
    if wrap4 {
        n %= 4;
    }
    let (variant, subtype) = match pickup {
        RoomClearPickup::Fixed { variant, subtype } => (variant, subtype),
        RoomClearPickup::Mystery => {
            let shared = &mut state.boot_lifecycle.shared;
            match shared.bounded(4) {
                0 => (10, shared.bounded(6) + 1),
                1 => (20, shared.bounded(4) + 1),
                2 => (30, shared.bounded(3) + 1),
                _ => (40, shared.bounded(2) + 1),
            }
        }
    };
    if n <= 0 || n & 1 != 0 {
        return Ok(());
    }
    if variant == 10 {
        let owner = state
            .player_items
            .as_ref()
            .ok_or("Familiar room clear requires the Player's items")?;
        let daemons_tail = crate::item_queries::AppearanceQueries::new(
            owner,
            state.weapon_cache.primary,
            state.player.entity_source,
            None,
        )
        .has_trinket(22)
        .ok_or("Familiar heart drop requires the Player's trinkets")?;
        if daemons_tail && state.boot_lifecycle.shared.bounded(5) != 0 {
            return Ok(());
        }
    }
    let anchor = state.familiars[index].position;
    let Some(position) = crate::boss_reward::free_position(state, anchor)? else {
        return Err("Familiar room-clear drop requires the room grid and path costs".into());
    };
    let seed = state.boot_lifecycle.shared.next_u32();
    let spawner = Some(crate::pickups::SourceSpawner {
        entity_type: 3,
        variant: state.familiars[index].variant,
    });
    crate::clear_award::admit(
        state,
        variant,
        subtype,
        seed,
        position,
        WorldVector { x: 0.0, y: 0.0 },
        spawner,
    )?;
    play_clip(&mut state.familiars[index], "Spawn", true);
    Ok(())
}

/// 470CEE: Follow, then once Spawn has finished, Play FloatDown (The Relic
/// 23: Float, 4720F5) without force.
pub(crate) fn spawner(state: &mut State, index: usize) -> Result<(), String> {
    follow(state, index, false)?;
    let familiar = &mut state.familiars[index];
    if familiar.sprite.animation == "Spawn" && !familiar.sprite.playing {
        let clip = if familiar.variant == 23 { "Float" } else { "FloatDown" };
        play(familiar, clip);
    }
    Ok(())
}

/// Blue Fly 468AD0: with Random(3) == 0 the target search 4688D0(13, 200).
/// With a target its position is the goal and the fly leaves orbit layer 1
/// (F7C = -1, 466780(1, false)); without one it rejoins the layer when out
/// of it and the goal is the orbit point around the Player. The velocity
/// gains the goal offset scaled to min(0.15 x length, 2) (nothing below
/// length 0.1) and the friction takes x0.8. PORT GAP: Hive Mind's x1.25 on
/// Entity+2C4 (468BD9) is not applied.
pub(crate) fn blue_fly(state: &mut State, index: usize) -> Result<(), String> {
    if state.boot_lifecycle.shared.bounded(3) == 0 {
        find_target(state, index, 13, 200.0)?;
    }
    let target = state.familiars[index]
        .target
        .and_then(|id| state.room.npcs.iter().find(|npc| npc.id == id && !npc.removed))
        .map(|npc| npc.position);
    if let Some(position) = target {
        state.familiars[index].target_position = position;
        state.familiars[index].orbit_layer = -1;
        respace(state, 1, index, false);
    } else {
        if state.familiars[index].orbit_layer == -1 {
            state.familiars[index].orbit_layer = 1;
            respace(state, 1, index, false);
        }
        let center = crate::bombs::player_world(state);
        let point = orbit_point(state, index, center, 0);
        state.familiars[index].target_position = point;
    }
    let familiar = &mut state.familiars[index];
    let offset = WorldVector {
        x: familiar.target_position.x - familiar.position.x,
        y: familiar.target_position.y - familiar.position.y,
    };
    let length = (offset.x * offset.x + offset.y * offset.y).sqrt();
    let scale = if length >= 0.1 {
        (length * 0.15_f32).min(2.0) / length
    } else {
        0.0
    };
    familiar.velocity = familiar.velocity + offset * scale;
    familiar.current_friction *= 0.8;
    Ok(())
}

/// Rotten Baby 54 in the Shooter (4703EF): while its last Blue Fly (FD0)
/// lives it does not fire and keeps its cooldown; otherwise the target
/// search 4688D0(1, 200), AddBlueFlies 508740(1, position, target) and
/// FD0 takes the fly (470260). Returns whether the cooldown restarts.
fn rotten_fly(state: &mut State, index: usize) -> Result<bool, String> {
    if let Some(id) = state.familiars[index].spawned {
        if state
            .familiars
            .iter()
            .any(|familiar| familiar.id == id && !familiar.removed)
        {
            return Ok(false);
        }
        state.familiars[index].spawned = None;
    }
    find_target(state, index, 1, 200.0)?;
    let position = state.familiars[index].position;
    let target = state.familiars[index].target;
    let fly = crate::familiars::add_blue_flies(state, 1, position, target)?;
    state.familiars[index].spawned = fly;
    Ok(true)
}
