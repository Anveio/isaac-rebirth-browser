//! Source445c60 samples every category before the first real effect birth.
use crate::mt::Mt19937;

#[derive(Clone, Copy)]
pub enum AmountScale {
    Ordinary,
    Challenge23,
}

/// Slots are the source stack+20..38: blood, bone, gut, eye, unused, poop,
/// special63. Large16 changes birth behavior; it never samples a count.
pub fn sample(
    amount: u32,
    flags: u32,
    scale: AmountScale,
    shared: &mut Mt19937,
) -> Result<[u32; 7], String> {
    let mut counts = [0; 7];
    if amount == 0 {
        return Ok(counts);
    }
    let divisor = (flags & 0x27).count_ones();
    if flags & !0x7f != 0 || divisor == 0 {
        return Err("gib counts require mapped flags and a real source category divisor".into());
    }
    let amount = amount
        .checked_mul(match scale {
            AmountScale::Ordinary => 1,
            AmountScale::Challenge23 => 8,
        })
        .filter(|amount| *amount < i32::MAX as u32)
        .ok_or("gib amount exceeds the source signed domain")?;
    //445cde..445d36: B2 is consumed even when a category later samples zero.
    let budget = (amount + shared.bounded(2)) / divisor + 1;
    if flags & 1 != 0 {
        counts[0] = shared.bounded(budget) + shared.bounded(budget);
    }
    if flags & 2 != 0 {
        counts[1] = budget / 2 + shared.bounded(budget / 2);
    }
    if flags & 4 != 0 {
        counts[2] = shared.bounded(budget);
    }
    if flags & 8 != 0 {
        counts[3] = shared.bounded(budget.min(3));
    }
    if flags & 32 != 0 {
        counts[5] = shared.bounded(budget);
    }
    if flags & 64 != 0 {
        counts[6] = shared.bounded(budget);
    }
    Ok(counts)
}

/// The 445f40 gib loop for a dying NPC, category by category: speed
/// Float*4+2, seed, velocity Angle*(1,.5)*speed and offset Angle*.5*radius,
/// then the effect's immediate Update. Categories 0..4 spawn Blood Particle
/// 1000.5 with the category as subtype and 5 spawns Poop Particle 1000.58.
pub(crate) fn burst(
    npc: &crate::npcs::Npc,
    random: &mut Mt19937,
    world: &mut crate::entities::NpcWorld<'_>,
) -> Result<(), String> {
    use crate::{
        actors::{ActorKind, ActorRef},
        familiars::Parent,
        motion::WorldVector,
        poop_effects::{Admission, ParticleBirth},
    };
    let config = crate::content::entity(npc.kind, npc.variant);
    // Challenge23 scales the amount by 8; this caller has no challenge owner.
    let counts = sample(
        config.gib_amount,
        config.gib_flags,
        AmountScale::Ordinary,
        random,
    )?;
    for (category, count) in counts.into_iter().enumerate() {
        for _ in 0..count {
            if category == 6 {
                return Err("Die category6 Effect1000.63 is not admitted".into());
            }
            let speed = random.uniform() * 4.0 + 2.0;
            let seed = random.next_u32();
            let direction = crate::npcs::direction(random);
            let velocity = WorldVector {
                x: direction.x,
                y: direction.y * 0.5,
            } * speed;
            let offset = (crate::npcs::direction(random) * 0.5) * npc.collision_radius;
            let birth = ParticleBirth {
                seed,
                position: npc.position + offset,
                velocity,
                spawner: Some(Parent {
                    actor: ActorRef {
                        kind: ActorKind::Npc,
                        id: npc.id,
                    },
                    entity_type: npc.kind,
                    variant: npc.variant,
                }),
            };
            let mut admission = Admission {
                game_update_count: world.effects.game_update_count,
                next_effect: world.effects.next_effect,
                effects: world.effects.effects,
                scheduler: world.effects.scheduler,
            };
            let id = if category == 5 {
                admission.admit_particle(birth)?
            } else {
                admission.admit_blood_particle(birth, category as u32)?
            };
            admission.advance_particle_initial(id, random)?;
        }
    }
    Ok(())
}
