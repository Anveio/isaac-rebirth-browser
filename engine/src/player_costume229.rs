//! Real Add229 registration and the head poses consumed by 519FC0 C08=7.
//! Unknown wardrobe/restore histories stay explicit. Pose gaps do not stop the
//! independently admitted charge/burst controller or invent missing nulls.
use crate::{
    motion::WorldVector,
    phase::Phase,
    player_costume118::{CostumeSlot, CostumeSprite},
    State,
};
use serde::{Deserialize, Serialize};

fn animation(name: &str) -> crate::content::CostumeClip {
    crate::content::costume_clip("gfx/characters/229_monstros lung.anm2", name).expect("229 clip")
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Costume229 {
    pub slots: [Option<CostumeSlot>; 13],
    pub sprite: CostumeSprite,
    pub priority: i32,
    pub history_frontier: Option<String>,
    pub main_clip_frontier: bool,
    ///Only 51BBD2's actual release writer is represented here. This is a
    ///historical write, not a fabricated current main-Sprite playback owner.
    pub last_shoot_main_clip_ba8: Option<String>,
}
impl Costume229 {
    fn fresh() -> Self {
        let mut slots = [None; 13];
        //50EDC0 matches this actor's sole "head" layer to actual slot4.
        slots[4] = Some(CostumeSlot {
            entry: 0,
            layer: 0,
            body_group: false,
        });
        Self {
            slots,
            sprite: CostumeSprite::stopped("HeadDown"),
            priority: 22,
            history_frontier: None,
            main_clip_frontier: false,
            last_shoot_main_clip_ba8: None,
        }
    }
}

///52D196 actually registers before cache evaluation. Counts/restores alone
///never create this owner. Other costumes need their real ordering/slot proof.
pub(crate) fn on_collectible_added(state: &mut State, id: u32) {
    let Some(items) = state.player_items.as_mut() else {
        return;
    };
    // Actual Add4 registers only its lower-priority sole head layer. A
    // known selected229 entry stays the same object, but50EDC0 still resets
    // that entry's pose through40FEF0. No counts create missing ownership.
    let first_cricket_after_known229 = id == 4
        && state.weapon_admission.source_player_type == Some(0)
        && items.trinket_slots == [0, 0]
        && items.temporary_collectibles.is_empty()
        && items.owned_counts.get(4) == Some(&1)
        && items.owned_counts.get(229).is_some_and(|count| *count > 0)
        && items
            .costume_registrations
            .as_ref()
            .is_some_and(|history| history.configuration_ids == [229, 4])
        && !items
            .owned_counts
            .iter()
            .enumerate()
            .any(|(id, count)| id != 4 && id != 229 && *count != 0);
    if first_cricket_after_known229 {
        if let Some(costume) = &mut items.costume229 {
            if costume.history_frontier.is_none() {
                // 229priority22 beats4priority0 in the same head slot4.
                // FullAdd52CF25 clears selected entry2EC before511060;
                // then50EDC0 resets the pose. BA8/main/history stay owned.
                costume.sprite.override_active = false;
                costume
                    .sprite
                    .set_frame_with_frames("HeadDown", 0, animation("HeadDown").frames);
                return;
            }
        }
    }
    if id != 229 {
        if let Some(costume) = &mut items.costume229 {
            costume
                .history_frontier
                .get_or_insert("otherCostumeGrant".into());
        }
        return;
    }
    let actual_registration = items.costume_registrations.as_ref().is_some_and(|history| {
        history.configuration_ids == [229]
            || (history.configuration_ids == [4, 229]
                && items.owned_counts.get(4) == Some(&1)
                && items.owned_counts.get(229) == Some(&1))
    });
    let fresh_context = actual_registration
        && state.weapon_admission.source_player_type == Some(0)
        && items.trinket_slots == [0, 0]
        && items.temporary_collectibles.is_empty()
        && !items
            .owned_counts
            .iter()
            .enumerate()
            .any(|(id, count)| id != 4 && id != 229 && *count != 0);
    let actual_count = items.owned_counts[229];
    if let Some(previous) = &mut items.costume229 {
        if previous.history_frontier.is_some() {
            return;
        }
        if !fresh_context || actual_count <= 1 {
            previous.history_frontier = Some("untracked229RegistrationHistory".into());
            return;
        }
        //511400->5357F0 removes the matching real entry, then51133A rebuilds
        //the selected head and50EDC0 resets its stopped HeadDown0 pose.
        let mut rebuilt = Costume229::fresh();
        rebuilt.main_clip_frontier = previous.main_clip_frontier;
        //Re-registration does not claim a new write to the independent BA8.
        rebuilt.last_shoot_main_clip_ba8 = previous.last_shoot_main_clip_ba8.clone();
        items.costume229 = Some(rebuilt);
        return;
    }
    let mut costume = Costume229::fresh();
    if !fresh_context {
        costume.history_frontier = Some("priorCostumeHistory".into());
    } else if actual_count != 1 {
        costume.history_frontier = Some("missing229RegistrationHistory".into());
    }
    items.costume229 = Some(costume);
}

fn selected(state: &mut State) -> Option<&mut Costume229> {
    state
        .player_items
        .as_mut()?
        .costume229
        .as_mut()
        .filter(|c| c.history_frontier.is_none())
}
fn direction_name(direction: i32) -> Result<&'static str, String> {
    match direction {
        0 => Ok("Left"),
        1 => Ok("Up"),
        2 => Ok("Right"),
        3 => Ok("Down"),
        _ => Err("229 costume requires actual DA0 in0..3".into()),
    }
}
fn aim_direction(aim: WorldVector) -> Result<&'static str, String> {
    if !aim.is_finite() {
        return Err("229 costume aim is nonfinite".into());
    }
    //50B0D0 strict |x|>|y|; exact ties/zero use the Y/down branch.
    Ok(if aim.x.abs() > aim.y.abs() {
        if aim.x < 0.0 {
            "Left"
        } else {
            "Right"
        }
    } else if aim.y < 0.0 {
        "Up"
    } else {
        "Down"
    })
}

///51C526..552 passes actual C0C after its Full-only increment. The frame
///request is CVTT(f32(C0C)/f32(C78)*18), clamped by the authored18-frame clip.
pub(crate) fn partial_charge(
    state: &mut State,
    aim: WorldVector,
    current: i32,
    maximum: i32,
) -> Result<bool, String> {
    let direction = aim_direction(aim)?;
    if current < 0 || maximum <= 0 || current > maximum {
        return Err("229 partial pose requires the actual nonnegative charging interval".into());
    }
    let Some(costume) = selected(state) else {
        return Ok(false);
    };
    let name = format!("Head{direction}Charge");
    costume.sprite.override_active = true;
    costume.sprite.set_frame_with_frames(
        &name,
        ((current as f32 / maximum as f32) * 18.0) as i32,
        animation(&name).frames,
    );
    Ok(true)
}

///51DE21 uses508820(frame=-1), hence Play(force=false); no frame0 restart
///when the same ChargeFull clip is already playing. Normal variant0's flash
///helper50E730 returns without a color/audio or random draw.
pub(crate) fn full_charge(state: &mut State, aim: WorldVector) -> Result<bool, String> {
    let direction = aim_direction(aim)?;
    let Some(costume) = selected(state) else {
        return Ok(false);
    };
    let name = format!("Head{direction}ChargeFull");
    costume.sprite.override_active = true;
    costume
        .sprite
        .play_with_looping(&name, animation(&name).looping);
    Ok(true)
}

///51BAA0 uses the caller's cardinal priorDB4, not current released input or
///DA0. 51BBD2 separately stores the selected Head* name into main BA8.
pub(crate) fn shoot(state: &mut State, prior_cardinal_aim: WorldVector) -> Result<bool, String> {
    let direction = aim_direction(prior_cardinal_aim)?;
    let Some(costume) = selected(state) else {
        return Ok(false);
    };
    let name = format!("Head{direction}Shoot");
    costume.sprite.override_active = true;
    costume
        .sprite
        .play_with_looping(&name, animation(&name).looping);
    costume.last_shoot_main_clip_ba8 = Some(format!("Head{direction}"));
    Ok(true)
}

///531C32/533850 invokes50B040("") for actual C08=7 before519FC0. A
///stopped nonloop assigned override makes531C53..C7D clear entry2EC. This
///uses the real Sprite playing/Loop owners, never an elapsed-charge estimate.
pub(crate) fn begin_full(state: &mut State, phase: Phase) {
    if phase != Phase::Update
        || state.weapon_cache.primary != crate::weapon_plan::PrimaryWeaponC08::MonstrosLung
    {
        return;
    }
    let Some(costume) = selected(state) else {
        return;
    };
    if costume.slots[4].is_some()
        && costume.sprite.override_active
        && !costume.sprite.playing
        && !animation(&costume.sprite.clip).looping
    {
        costume.sprite.override_active = false;
    }
}

///52F713's normal head synchronization cannot override an active508820 pose.
pub(crate) fn sync_head(state: &mut State, frame: i32) -> Result<(), String> {
    let direction = state.attack_runtime.head_direction_da0;
    let Some(costume) = selected(state) else {
        return Ok(());
    };
    if !costume.sprite.override_active {
        if let Some(direction) = direction {
            let name = format!("Head{}", direction_name(direction)?);
            costume
                .sprite
                .set_frame_with_frames(&name, frame, animation(&name).frames);
        } else {
            costume.history_frontier = Some("missingDA0History".into());
        }
    }
    Ok(())
}

///531E37..532090 advances the assigned head entry once on actual Full only.
///No body layer/null or complete guarded-main presentation is inferred.
pub(crate) fn advance_full(state: &mut State, phase: Phase) {
    let main_clip = state.player_animation.current.is_some()
        || state
            .player_items
            .as_ref()
            .is_some_and(|items| items.pickup_animation_guard);
    let Some(costume) = selected(state) else {
        return;
    };
    costume.main_clip_frontier = main_clip;
    if phase == Phase::Update && costume.slots[4].is_some() {
        let clip = animation(&costume.sprite.clip);
        costume
            .sprite
            .advance_full_with_clip(clip.frames, clip.looping);
    }
}
