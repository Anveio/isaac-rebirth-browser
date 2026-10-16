//! Tear.Update 542530 dead branch for Isaac's normal variant0 tear. The
//! update after the dead flag is set plays the impact sound, spawns the
//! sized TearPoof and the faint Blood7 splashes, then removes the tear.

use crate::audio::SoundRequest;
use crate::motion::WorldVector;
use crate::{blood_effects, critters, familiars, State};

pub(crate) struct Death {
    pub position: WorldVector,
    pub height: f32,
    /// Tear+7e4 scale from the cached tear parameters.
    pub scale: f32,
}

pub(crate) fn emit(state: &mut State, death: Death) -> Result<(), String> {
    let scale = death.scale;
    // 5425d0..542640: scale <0.4 (66aa10) and <0.8 (669044) choose the small
    // poofs 0x50/0x4f; otherwise height <= -5 (66c194) is the wall PoofA
    // with SFX150 and the ground PoofB plays SFX258.
    let variant = if scale < 0.4 {
        0x50
    } else if scale < 0.8 {
        0x4f
    } else if death.height <= -5.0 {
        12
    } else {
        13
    };
    state.audio.admit(
        &mut state.boot_lifecycle.shared,
        state.system_clock.system_counter.wrapping_sub(1),
        SoundRequest {
            id: if variant == 12 { 150 } else { 258 },
            volume: 1.0,
            delay: 2,
            looping: false,
            pitch: 1.0,
        },
    );
    // 5438f0: seeded Game.Spawn 1000.variant at the tear position. Effect
    // Init 458b9c draws Random(4) for 12/13/0x4f/0x50 into its rotation 0x814.
    let seed = state.boot_lifecycle.shared.next_u32();
    let poof = critters::admit_plain(state, variant, seed, death.position)?;
    let _init_rotation = state.boot_lifecycle.shared.bounded(4) as f32 * 90.0;
    // 543940: except for the small 0x4f/0x50 poofs, sprite scale is 0.8 *
    // tear scale and sprite rotation 0x2cc is a second Random(4) * 90.
    // SetColor copies the default white tear color.
    if variant == 12 || variant == 13 {
        let rotation = state.boot_lifecycle.shared.bounded(4) as f32 * 90.0;
        let effect = state
            .sprite_effects
            .iter_mut()
            .find(|effect| effect.id == poof)
            .ok_or("tear poof was not admitted")?;
        let sprite_scale = scale * 0.8;
        effect.sprite.scale = WorldVector {
            x: sprite_scale,
            y: sprite_scale,
        };
        effect.sprite.rotation = rotation;
    }
    // 5439f0: variant0 tears add Random(5) Blood7 splashes, each seeded at
    // tear + Direction * (U * 24), scale 0.3 and an immediate Update. Alpha
    // is 0.08 (3da3d70a); item 310 draws U * 0.3 + 0.2, and otherwise item 6
    // copies an opaque color whose alpha the Blood7 Update then randomizes.
    let owned = |state: &State, item: usize| {
        state
            .player_items
            .as_ref()
            .is_some_and(|items| items.owned_counts.get(item).is_some_and(|count| *count > 0))
    };
    let splashes = state.boot_lifecycle.shared.bounded(5);
    for _ in 0..splashes {
        let seed = state.boot_lifecycle.shared.next_u32();
        let distance = state.boot_lifecycle.shared.uniform() * 24.0;
        let direction = blood_effects::direction(&mut state.boot_lifecycle.shared);
        let position = death.position + direction * distance;
        let alpha = if owned(state, 310) {
            state.boot_lifecycle.shared.uniform() * 0.3 + 0.2
        } else if owned(state, 6) {
            1.0
        } else {
            0.08
        };
        let blood =
            familiars::admit_blood_splat(state, seed, position, [1.0, 1.0, 1.0, alpha], [0.0; 3])?;
        familiars::set_blood_scale(state, blood, 0.3)?;
        familiars::advance_blood_initial(state, blood)?;
    }
    Ok(())
}
