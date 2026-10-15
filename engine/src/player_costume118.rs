//! Fresh normal Isaac's consumed 118 costume. General wardrobe composition,
//! flying/base-sprite transforms and guarded main clips keep separate owners.
use crate::{motion::WorldVector, phase::Phase, State};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::OnceLock};

#[derive(Deserialize)]
struct Actor {
    #[serde(rename = "Animations")]
    animations: Vec<Animation>,
}
#[derive(Deserialize)]
struct Animation {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "FrameNum")]
    frames: i32,
    #[serde(rename = "Loop")]
    looping: bool,
}
fn animation(name: &str) -> &'static Animation {
    static ACTOR: OnceLock<Actor> = OnceLock::new();
    let actor = ACTOR.get_or_init(|| {
        let mut actors: BTreeMap<String, Actor> =
            serde_json::from_str(include_str!("../../web/costume118-data.json"))
                .expect("authored 118 animation data");
        actors
            .remove("gfx/characters/118_brimstone.anm2")
            .expect("118 actor")
    });
    actor
        .animations
        .iter()
        .find(|clip| clip.name == name)
        .expect("118 clip")
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CostumeSprite {
    pub clip: String,
    pub frame: f32,
    pub playing: bool,
    ///Entry+2EC suppresses normal pose synchronization, not visibility.
    pub override_active: bool,
}
impl CostumeSprite {
    fn stopped(clip: &str) -> Self {
        Self {
            clip: clip.into(),
            frame: 0.0,
            playing: false,
            override_active: false,
        }
    }
    ///40FEF0/40EB30 selects the named clip and clamps to its final frame.
    fn set_frame(&mut self, name: &str, frame: i32) {
        self.clip = name.into();
        self.frame = frame.min(animation(name).frames - 1) as f32;
        self.playing = false;
    }
    ///40FD80 force=false does not restart an existing nonloop clip, including
    ///its stopped final pose. A stopped loop or a changed name starts at0.
    fn play(&mut self, name: &str) {
        if self.clip != name || (animation(name).looping && !self.playing) {
            self.clip = name.into();
            self.frame = 0.0;
            self.playing = true;
        }
    }
    fn advance_full(&mut self) {
        if !self.playing {
            return;
        }
        let clip = animation(&self.clip);
        self.frame += 1.0; //Fresh Sprite+2D8 is1; no C0C-derived clock.
        if self.frame.floor() >= clip.frames as f32 {
            if clip.looping {
                self.frame = 0.0;
            } else {
                self.frame = (clip.frames - 1) as f32;
                self.playing = false;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CostumeSlot {
    ///Relative indices in this admitted pair, not guessed native addresses.
    pub entry: usize,
    pub layer: u32,
    pub body_group: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Costume118 {
    pub slots: [Option<CostumeSlot>; 13],
    pub sprites: [CostumeSprite; 2],
    pub priority: i32,
    pub skin_color: i32,
    pub history_frontier: Option<String>,
    pub body_history_admitted: bool,
    pub main_clip_frontier: bool,
    ///Port coverage metadata for an actual additional selected costume layer;
    ///it never invalidates the independently owned118 Mouth/pose history.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additional_layer_frontier: Option<u32>,
}
impl Costume118 {
    fn fresh() -> Self {
        let mut slots = [None; 13];
        //511060 splits the two slot groups;50EDC0 matches base layer names.
        slots[1] = Some(CostumeSlot {
            entry: 0,
            layer: 1,
            body_group: true,
        });
        slots[4] = Some(CostumeSlot {
            entry: 1,
            layer: 0,
            body_group: false,
        });
        Self {
            slots,
            sprites: [
                CostumeSprite::stopped("WalkDown"),
                CostumeSprite::stopped("HeadDown"),
            ],
            priority: 23,
            skin_color: 1,
            history_frontier: None,
            body_history_admitted: true,
            main_clip_frontier: false,
            additional_layer_frontier: None,
        }
    }
}

///Actual52D196 costume registration, before the grant's cache callback. Only
///known118 registration and the source-closed first330 Null-preserving change
///are admitted; other grants retain their own slot/Null/removal frontier.
pub(crate) fn on_collectible_added(state: &mut State, id: u32) {
    let normal_nonflying =
        state.weapon_admission.source_player_type == Some(0) && state.player.flying == Some(false);
    let Some(items) = state.player_items.as_mut() else {
        return;
    };
    //Actual330 registration selects distinct head1 slot6, leaving118's
    //body1/head4 mappings. The authored330 actor has no Nulls in any clip,
    //so it cannot replace118's Mouth; its additional render layer is pending.
    //50EDC0 still stops each selected body/head at its Down0 pose.
    //Preserve only this real post-Add330 history, never restore an unknown
    //wardrobe from counts or clear its separate body/main render frontiers.
    let null_preserving_soy_registration = id == 330
        && normal_nonflying
        && items.trinket_slots == [0, 0]
        && items.temporary_collectibles.is_empty()
        && items.owned_counts[330] == 1
        && items.owned_counts[118] > 0;
    if null_preserving_soy_registration {
        if let Some(costume) = items
            .costume118
            .as_mut()
            .filter(|costume| costume.history_frontier.is_none())
        {
            costume.sprites[0].set_frame("WalkDown", 0);
            costume.sprites[1].set_frame("HeadDown", 0);
            costume.additional_layer_frontier = Some(330);
            return;
        }
    }
    if id != 118 {
        if let Some(costume) = &mut items.costume118 {
            costume
                .history_frontier
                .get_or_insert("otherCostumeGrant".into());
        }
        return;
    }
    let fresh_context = state.weapon_admission.source_player_type == Some(0)
        && items.temporary_collectibles.is_empty()
        && !items
            .owned_counts
            .iter()
            .enumerate()
            .any(|(id, count)| id != 118 && *count != 0);
    let actual_count = items.owned_counts[118];
    if let Some(previous) = &mut items.costume118 {
        if previous.history_frontier.is_some() {
            return;
        }
        if !fresh_context || actual_count <= 1 {
            previous.history_frontier = Some("untracked118RegistrationHistory".into());
            return;
        }
        //511400→5357F0 removes each matching entry, shifts remaining entries,
        //destroys the old tail, then resumes5110C4. Both known118 entries are
        //replaced;5110D1/51133A construct the pair and50EDC0 reassigns slots,
        //priority/skin and stopped WalkDown0/HeadDown0. No old pose survives.
        let mut rebuilt = Costume118::fresh();
        //Re-registration resets costume sprites, not the independently
        //unimplemented movement/base-sprite or guarded-main presentation.
        rebuilt.body_history_admitted = previous.body_history_admitted;
        rebuilt.main_clip_frontier = previous.main_clip_frontier;
        items.costume118 = Some(rebuilt);
        return;
    }
    let mut costume = Costume118::fresh();
    if !fresh_context {
        costume.history_frontier = Some("priorCostumeHistory".into());
    } else if actual_count != 1 {
        //A count restored without this real registration owner is not proof
        //that this grant started from the fresh constructor's zero count.
        costume.history_frontier = Some("missing118RegistrationHistory".into());
    }
    items.costume118 = Some(costume);
}

fn selected(state: &mut State) -> Option<&mut Costume118> {
    state
        .player_items
        .as_mut()?
        .costume118
        .as_mut()
        .filter(|c| c.history_frontier.is_none())
}
fn direction_name(direction: i32) -> Result<&'static str, String> {
    match direction {
        0 => Ok("Left"),
        1 => Ok("Up"),
        2 => Ok("Right"),
        3 => Ok("Down"),
        _ => Err("118 costume requires actual DA0 in0..3".into()),
    }
}
fn aim_direction(aim: WorldVector) -> Result<&'static str, String> {
    if !aim.is_finite() {
        return Err("118 costume aim is nonfinite".into());
    }
    //50B0D0 strict axis comparison; zero and absolute ties select Y/down.
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

///508820 group0 consumes the genuine caller vector. No DA0/facing fallback.
pub(crate) fn charge(
    state: &mut State,
    aim: WorldVector,
    prior: i32,
    maximum: i32,
) -> Result<bool, String> {
    let direction = aim_direction(aim)?;
    if maximum <= 0 {
        return Err("118 charge requires positive C78".into());
    }
    let Some(costume) = selected(state) else {
        return Ok(false);
    };
    let head = &mut costume.sprites[1];
    head.override_active = true;
    if prior < maximum {
        head.set_frame(
            &format!("Head{direction}Charge"),
            ((prior as f32 / maximum as f32) * 18.0) as i32,
        );
    } else {
        head.play(&format!("Head{direction}ChargeFull"));
    }
    //50E730 returns immediately for this normal Player variant0.
    Ok(true)
}
pub(crate) fn shoot(state: &mut State, aim: WorldVector) -> Result<bool, String> {
    let direction = aim_direction(aim)?;
    let Some(costume) = selected(state) else {
        return Ok(false);
    };
    costume.sprites[1].override_active = true;
    costume.sprites[1].play(&format!("Head{direction}Shoot"));
    Ok(true)
}
pub(crate) fn hide(state: &mut State) -> bool {
    let Some(costume) = selected(state) else {
        return false;
    };
    for sprite in &mut costume.sprites {
        sprite.override_active = false;
    }
    true
}

///Actual52F713 normal head synchronization follows519FC0/idleDA0, on both
///callbacks. The base head's frame request is0/2 before BB0 decrement.
pub(crate) fn sync_head(state: &mut State, frame: i32) -> Result<(), String> {
    let direction = state.attack_runtime.head_direction_da0;
    let Some(costume) = selected(state) else {
        return Ok(());
    };
    if !costume.sprites[1].override_active {
        if let Some(direction) = direction {
            costume.sprites[1].set_frame(&format!("Head{}", direction_name(direction)?), frame);
        } else {
            costume.history_frontier = Some("missingDA0History".into());
        }
    }
    Ok(())
}

///531E37→532090: full only, after copied-item clock. Advance each assigned
///entry once, despite its potentially repeated slot mappings. The118 actor
///has no WalkDown_Loop, so its stationary body normal-sync stops WalkDown0.
pub(crate) fn advance_full(state: &mut State, phase: Phase) {
    let moving = state
        .pressed
        .iter()
        .any(|key| matches!(key.as_str(), "KeyA" | "KeyD" | "KeyW" | "KeyS"));
    let main_clip = state.player_animation.current.is_some()
        || state
            .player_items
            .as_ref()
            .is_some_and(|items| items.pickup_animation_guard);
    let Some(costume) = selected(state) else {
        return;
    };
    costume.body_history_admitted &= !moving;
    costume.main_clip_frontier = main_clip;
    if phase == Phase::Update {
        if costume.body_history_admitted && !costume.sprites[0].override_active {
            costume.sprites[0].set_frame("WalkDown", 0);
        }
        let mut advanced = [false; 2];
        for slot in costume.slots.iter().flatten() {
            if !advanced[slot.entry] {
                costume.sprites[slot.entry].advance_full();
                advanced[slot.entry] = true;
            }
        }
    }
    //A consumed head pose never turns an unimplemented body/main render path
    //into complete wardrobe admission. Preserve this diagnostic and all prior
    //controller/frontier failures, including on an intermediate callback.
    let frontier = if costume.main_clip_frontier {
        Some("ordinaryBrimstoneMainClipCostumeRender")
    } else if !costume.body_history_admitted {
        Some("ordinaryBrimstoneBodyCostumeTransform")
    } else if costume.additional_layer_frontier.is_some() {
        Some("ordinaryBrimstoneAdditionalCostumeLayer")
    } else {
        None
    };
    if let Some(channel) = frontier {
        state.weapon_admission.unadmitted_controller.get_or_insert(
            crate::weapon_plan::UnadmittedController {
                primary: state.weapon_cache.primary.numeric(),
                channel: channel.into(),
            },
        );
    }
}
