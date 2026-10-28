//! The tear property getter 50D090 (kinds 1 and 5), ported whole.
//!
//! 50D090(out, Player, kind) copies the Player's C88 damage, C8C height, C98
//! flags and CA0 color, derives the size from the damage, then runs one
//! address-ordered chain of item tests. Each test may draw the shared Random
//! (42E8A0), recolor (42F480 tint, 42F4A0 byte tint, 42F510 byte offsets),
//! set flags, change the variant or scale the size. Kind 1 (FireTear 5177F0)
//! and kind 5 then apply their own tails.
//!
//! Every query the chain can reach is read before the first draw. An
//! unknown fact returns `None` without touching the shared stream, so the
//! caller keeps its documented unknown-variant frontier.
use crate::{
    attack_color::AttackColor, attack_flags::AttackFlags, item_queries::AppearanceQueries,
    mt::Mt19937, weapon_plan::PrimaryWeaponC08,
};

/// The getter's output record (50D090 out+0..+40).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TearProperties {
    /// +0: the Tear variant.
    pub variant: u32,
    /// +4..+2C: the copied CA0 color after the item recolors.
    pub color: AttackColor,
    /// +30: the copied C98 flags with the item bits.
    pub flags: AttackFlags,
    /// +38: the copied C88 damage with the item additions.
    pub damage: f32,
    /// +3C: the size, before the 541FF0 bound.
    pub scale: f32,
    /// +40: the copied C8C height with the item offset.
    pub height: f32,
}

/// The Player fields 50D090 reads besides its item queries.
#[derive(Clone, Copy)]
pub(crate) struct PlayerFields {
    pub damage_c88: f32,
    pub height_c8c: f32,
    pub flags_c98: AttackFlags,
    pub color_ca0: AttackColor,
    pub luck_d64: f32,
    /// Signed byte D70, the eye the shot leaves from (-1 or 1).
    pub eye_d70: i8,
    pub player_type_bfc: u32,
    pub primary_c08: PrimaryWeaponC08,
}

/// How the chain queries an id: 5090C0 only, 5E3D70 (CF8) only, or 5090C0
/// then 5E3D70 when the first is false.
#[derive(Clone, Copy, PartialEq)]
enum Query {
    Have,
    Temporary,
    Either,
}
use Query::{Either, Have as Owned, Temporary as Record};

/// Collectible queries in reach order.
const QUERIES: &[(u32, Query)] = &[
    (110, Either),
    (89, Owned),
    (231, Owned),
    (103, Owned),
    (200, Owned),
    (201, Either),
    (221, Owned),
    (330, Owned),
    (336, Owned),
    (228, Owned),
    (230, Owned),
    (229, Owned),
    (7, Owned),
    (138, Owned),
    (80, Owned),
    (183, Owned),
    (34, Record),
    (90, Owned),
    (122, Record),
    (126, Owned),
    (154, Owned),
    (254, Owned),
    (261, Owned),
    (309, Owned),
    (48, Owned),
    (306, Owned),
    (257, Either),
    (259, Either),
    (317, Either),
    (237, Either),
    (315, Either),
    (213, Either),
    (305, Owned),
    (150, Either),
    (125, Owned),
    (140, Owned),
];

/// Every fact the chain reads, resolved before it starts.
struct Facts {
    have: [bool; 347],
    temporary: [bool; 347],
    trinket30: bool,
    trinket31: bool,
}

impl Facts {
    fn read(queries: AppearanceQueries<'_>) -> Option<Self> {
        let mut facts = Self {
            have: [false; 347],
            temporary: [false; 347],
            trinket30: queries.has_trinket(30)?,
            trinket31: queries.has_trinket(31)?,
        };
        for &(id, query) in QUERIES {
            if query != Record {
                facts.have[id as usize] = queries.have(id as usize)?;
            }
            if query != Owned {
                facts.temporary[id as usize] = queries.temporary_present(id)?;
            }
        }
        Some(facts)
    }
    /// 5090C0 alone.
    fn has(&self, id: u32) -> bool {
        self.have[id as usize]
    }
    /// 5E3D70 alone.
    fn temporary(&self, id: u32) -> bool {
        self.temporary[id as usize]
    }
    /// 5090C0, then 5E3D70 on CF8 when it is false.
    fn has_or_temporary(&self, id: u32) -> bool {
        self.have[id as usize] || self.temporary[id as usize]
    }
}

/// 42F480: tint only.
fn tint(color: &mut AttackColor, tint: [f32; 4]) {
    color.tint = tint;
}

/// 42F4A0: tint from bytes times the stored binary32 1/255 (668980).
fn byte_tint(color: &mut AttackColor, bytes: [i32; 4]) {
    let scale = 1.0_f32 / 255.0_f32;
    color.tint = bytes.map(|byte| byte as f32 * scale);
}

/// 42F510: offsets from bytes times the same constant.
fn byte_offsets(color: &mut AttackColor, bytes: [i32; 3]) {
    let scale = 1.0_f32 / 255.0_f32;
    color.offsets = bytes.map(|byte| byte as f32 * scale);
}

/// cvttss2si: i32::MIN for NaN and out-of-range values, as the hardware does.
fn truncate(value: f32) -> i32 {
    if value.is_finite() && value.abs() < 2_147_483_648.0 {
        value as i32
    } else {
        i32::MIN
    }
}

/// `max(minimum, base - trunc(luck * factor))`, the bound of each luck roll.
fn luck_bound(base: i32, luck: f32, factor: f32, minimum: i32) -> u32 {
    base.wrapping_sub(truncate(luck * factor)).max(minimum) as u32
}

/// 50D090 for `kind`, drawing on `shared`. `entity_variant` is Entity+14;
/// the co-op variant 1 reads 30E8, which the port does not model, so it
/// stays unknown.
pub(crate) fn evaluate(
    queries: AppearanceQueries<'_>,
    player: PlayerFields,
    kind: u32,
    shared: &mut Mt19937,
) -> Option<TearProperties> {
    if queries.entity_variant()? != 0 {
        return None;
    }
    let facts = Facts::read(queries)?;
    let luck = player.luck_d64;
    let mut out = TearProperties {
        variant: 0,
        color: player.color_ca0,
        flags: player.flags_c98,
        damage: player.damage_c88,
        scale: 0.0,
        height: player.height_c8c,
    };
    //50D12B..50D171: (C88*.04+.6) + sqrt(C88)*.15, the sum in binary64.
    let linear = out.damage * 0.04_f32 + 0.6_f32;
    out.scale = (f64::from(linear) + f64::from(out.damage).sqrt() * f64::from(0.15_f32)) as f32;
    let mut flags = out.flags.0;

    //50D176: trinket 30.
    if facts.trinket30 && shared.bounded(luck_bound(10, luck, 0.5, 1)) == 0 {
        flags |= 0x10;
        if kind != 5 {
            tint(&mut out.color, [0.5, 0.97, 0.5, 1.0]);
        }
        byte_offsets(&mut out.color, [0, 0, 0]);
    }
    //50D1C7: trinket 31.
    if facts.trinket31 && shared.bounded(luck_bound(10, luck, 0.5, 1)) == 0 {
        flags |= 0x3;
        tint(&mut out.color, [1.5, 2.0, 2.0, 0.5]);
        byte_offsets(&mut out.color, [0, 0, 0]);
        out.variant = 11;
    }
    //50D218: 110.
    if facts.has_or_temporary(110) && shared.bounded(luck_bound(5, luck, 0.15, 2)) == 0 {
        flags |= 0x20;
        tint(&mut out.color, [1.25, 0.05, 0.15, 1.0]);
        byte_offsets(&mut out.color, [0, 0, 0]);
    }
    //50D26E: 89.
    if facts.has(89) && shared.bounded(luck_bound(4, luck, 0.2, 1)) == 0 {
        tint(&mut out.color, [2.0, 2.0, 2.0, 1.0]);
        byte_offsets(&mut out.color, [50, 50, 50]);
        flags |= 0x8;
    }
    //50D2B1: 231.
    if facts.has(231) && shared.bounded(luck_bound(10, luck, 0.5, 1)) == 0 {
        flags |= 0x1_0000_0008;
        tint(&mut out.color, [0.15, 0.15, 0.15, 1.0]);
        byte_offsets(&mut out.color, [0, 0, 0]);
    }
    //50D2F4: 103.
    if facts.has(103) && shared.bounded(luck_bound(4, luck, 0.25, 1)) == 0 {
        flags |= 0x10;
        tint(&mut out.color, [0.4, 0.97, 0.5, 1.0]);
        byte_offsets(&mut out.color, [0, 0, 0]);
    }
    //50D337: 200.
    if facts.has(200) && shared.bounded(luck_bound(10, luck, 0.334, 1)) == 0 {
        tint(&mut out.color, [1.0, 0.0, 1.0, 1.0]);
        byte_offsets(&mut out.color, [50, 0, 0]);
        flags |= 0x2000;
    }
    //50D37A: 201.
    if facts.has_or_temporary(201) && shared.bounded(luck_bound(10, luck, 0.334, 1)) == 0 {
        tint(&mut out.color, [0.5, 0.5, 0.5, 1.0]);
        byte_offsets(&mut out.color, [0, 0, 0]);
        flags |= 0x4000;
    }
    //50D3C1: 221.
    if facts.has(221) {
        flags |= 0x80000;
    }
    //50D3DA / 50D3FD: 330 and 336 sizes.
    if facts.has(330) {
        out.scale *= 0.5_f32;
    }
    if facts.has(336) {
        out.scale *= 1.5_f32;
    }
    //50D420 / 50DB30: without 330, any blood source turns the variant red.
    if !facts.has(330)
        && (facts.has(230)
            || facts.has(229)
            || facts.has(7)
            || facts.has(138)
            || facts.has(80)
            || facts.has(183)
            || facts.temporary(34)
            || facts.has(90)
            || facts.temporary(122)
            || facts.has(126))
    {
        out.variant = if out.variant == 11 { 12 } else { 1 };
    }
    //50D42D: 228 / 230 luck roll.
    if facts.has(228) || facts.has(230) {
        let roll = shared.bounded(luck_bound(100, luck, 1.0, 1));
        if roll <= 14 {
            if facts.has(230) {
                byte_tint(&mut out.color, [100, 100, 100, 255]);
                byte_offsets(&mut out.color, [0, 0, 0]);
            } else if facts.has(228) {
                byte_tint(&mut out.color, [255, 255, 116, 255]);
                byte_offsets(&mut out.color, [43, 37, 0]);
            }
            flags |= 0x10_0000;
        }
    }
    //50D4BB: 154, the eye at D70 (-1 always; otherwise one in two unless
    //this is not Azazel and the primary is ordinary tears).
    if facts.has(154) {
        let applies = player.eye_d70 == -1
            || ((player.player_type_bfc == 2
                || player.primary_c08 != PrimaryWeaponC08::RegularTears)
                && shared.bounded(2) == 0);
        if applies {
            out.variant = 1;
            out.damage += 2.0_f32;
        }
    }
    //50D500: 254, the same eye test on the sign of D70.
    if facts.has(254) {
        let applies = player.eye_d70 < 0
            || ((player.player_type_bfc == 2
                || player.primary_c08 != PrimaryWeaponC08::RegularTears)
                && shared.bounded(2) == 0);
        if applies {
            tint(&mut out.color, [1.0, 0.0, 0.0, 1.0]);
            byte_offsets(&mut out.color, [0, 0, 0]);
            out.damage += 1.0_f32;
            out.scale *= 1.25_f32;
            out.height -= 10.0_f32;
        }
    }
    //50D548 / 50D567: 261 doubles, 309 grows the size.
    if facts.has(261) {
        out.scale += out.scale;
    }
    if facts.has(309) {
        out.scale *= 1.25_f32;
    }
    //50D58A: 48 or 306.
    if facts.has(48) || facts.has(306) {
        out.variant = match out.variant {
            0 => 11,
            1 => 12,
            other => other,
        };
    }
    //50D5B1: 257.
    if facts.has_or_temporary(257) {
        flags |= 0x40_0000;
        out.variant = 5;
    }
    //50D5D5: 259 sets its variant before its luck roll.
    if facts.has_or_temporary(259) {
        out.variant = 6;
        if shared.bounded(luck_bound(3, luck, 0.1, 1)) == 0 {
            flags |= 0x10_0000;
        }
    }
    //50D628..50D691: later variants override earlier ones.
    for (id, variant) in [(317, 7), (237, 8), (315, 3), (213, 10)] {
        if facts.has_or_temporary(id) {
            out.variant = variant;
        }
    }
    //50D698: 305 (no CF8 search).
    if facts.has(305) {
        byte_tint(&mut out.color, [50, 255, 50, 255]);
        byte_offsets(&mut out.color, [0, 0, 0]);
    }
    //50D6AD: 150, Random(max(0, 10 - trunc(luck))); Random(0) is 0.
    if facts.has_or_temporary(150) {
        let bound = 10_i32.wrapping_sub(truncate(luck)).max(0) as u32;
        if shared.bounded(bound) == 0 {
            out.damage = 3.2_f32 * player.damage_c88;
            out.variant = 2;
        }
    }
    out.flags = AttackFlags(flags);
    match kind {
        //50DA28: kind 5 adds 125 and 140 flags.
        5 => {
            if facts.has(125) {
                out.flags.0 |= 0x4;
            }
            if facts.has(140) {
                out.flags.0 |= 0x10;
            }
        }
        //50D958: kind 1 sizes by the flag pair.
        1 => {
            if out.flags.0 & 0x2_0000_1000 == 0x1000 {
                out.scale *= 0.4_f32;
            }
            if out.flags.0 & 0x2_0004_0000 == 0x40000 {
                out.scale *= 1.2_f32;
            }
        }
        _ => {}
    }
    Some(out)
}

/// FireTear 5177F0's 50D090(kind1) call for an ordinary shot, on the
/// current Player and the shared stream. FireTear's own Have69/154 preludes
/// and its Have182/309 Tear writes are not ported, so either leaves the shot
/// unresolved (`None`, nothing drawn), as does any unknown fact.
pub(crate) fn fire_tear_kind1(state: &mut crate::State) -> Option<TearProperties> {
    let owner = state.player_items.as_ref()?;
    let queries = AppearanceQueries::new(
        owner,
        state.weapon_cache.primary,
        state.player.entity_source,
        None,
    );
    for id in [69, 154, 182, 309] {
        if queries.have(id)? {
            return None;
        }
    }
    let player = PlayerFields {
        damage_c88: state.player_stats.damage,
        height_c8c: state.player_stats.tear_height,
        flags_c98: state.weapon_cache.flags_c98,
        color_ca0: state.weapon_cache.color_ca0,
        luck_d64: state.player_stats.luck?,
        eye_d70: match state.player.eye {
            crate::projectiles::Eye::Negative => -1,
            crate::projectiles::Eye::Positive => 1,
        },
        player_type_bfc: state.weapon_admission.source_player_type?,
        primary_c08: state.weapon_cache.primary,
    };
    evaluate(queries, player, 1, &mut state.boot_lifecycle.shared)
}
