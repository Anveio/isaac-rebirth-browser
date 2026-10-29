//! Tear.Update 542530 dead branch for the variant0 and blood 1/12 tears. The
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
    /// Tear+458 PositionOffset y (its height offset), copied to the poof.
    pub render_offset_y: f32,
    /// Entity+14, the tear's builder variant; None is an unqualified tear.
    pub variant: Option<u32>,
}

pub(crate) fn emit(state: &mut State, death: Death) -> Result<(), String> {
    let scale = death.scale;
    // 5425ba/543cb0: blood variants 1 and 12 take 544fa1 instead: SFX150 at
    // any height and the 1000.11 blood poof, then the shared scale/rotation
    // tail below. Only variant 0 reaches the Blood7 splashes (543980).
    let blood = matches!(death.variant, Some(1 | 12));
    // 5425d0..542640: scale <0.4 (66aa10) and <0.8 (669044) choose the small
    // poofs 0x50/0x4f; otherwise height <= -5 (66c194) is the wall PoofA
    // with SFX150 and the ground PoofB plays SFX258.
    let variant = if blood {
        11
    } else if scale < 0.4 {
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
            id: if variant == 12 || variant == 11 {
                150
            } else {
                258
            },
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
    // 54390c..54391a: the poof takes the tear's +458 PositionOffset, so a
    // tear dying in the air splashes where it was drawn, not on its shadow.
    state
        .sprite_effects
        .iter_mut()
        .find(|effect| effect.id == poof)
        .ok_or("tear poof was not admitted")?
        .render_offset = WorldVector {
        x: 0.0,
        y: death.render_offset_y,
    };
    // 458b9c: the 1000.11 blood poof's Init draws no rotation.
    if variant != 11 {
        let init_rotation = state.boot_lifecycle.shared.bounded(4) as f32 * 90.0;
        state
            .sprite_effects
            .iter_mut()
            .find(|effect| effect.id == poof)
            .ok_or("tear poof was not admitted")?
            .rotation_814 = init_rotation;
    }
    // 543940: except for the small 0x4f/0x50 poofs, sprite scale is 0.8 *
    // tear scale and sprite rotation 0x2cc is a second Random(4) * 90, which
    // the first Update's 45abe3 overwrites with -814.
    // SetColor copies the default white tear color.
    if variant == 12 || variant == 13 || variant == 11 {
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
    if death.variant.is_some_and(|variant| variant != 0) {
        return Ok(());
    }
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
