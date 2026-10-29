//! Per-variant Familiar data, each table read from the original's own
//! dispatch: the Update jump table 0x66C818, Init's follower test 4666B0
//! and orbit-layer table 0x66C470, the grid class 46EF36, the Update tail's
//! hit-set interval 47064C, the collision callback's projectile blockers
//! 46FE40 and Fire 467170's per-variant tear. Behaviour lives in
//! familiar_ai.rs; nothing here is per-item code.

/// One case of Update 4704A0's switch (jump table 0x66C818).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UpdateCase {
    /// 47064C: no body (unused variants).
    None,
    /// 4708A9: Follow.
    Follower,
    /// 4708E6: Follow, Shooter.
    Shooter,
    /// 470910: Abel's mirror, Shooter.
    ShooterMirror,
    /// 4708BB: Mongo Baby.
    ShooterMimic,
    /// 470E5B: own motion, Shooter.
    ShooterFree,
    /// 471759: Orbit at the Player.
    Orbital,
    /// 471178: Orbit, Shooter.
    OrbitalShooter,
    /// 471168: Chase 469A00.
    Chaser,
    /// 470ADF: Chase, Shooter.
    ChaserShooter,
    /// 470CEE: Follow, bag spawn.
    Spawner,
    /// 470CDE: bum 46B060.
    Bum,
    /// Dry Baby 471401: Follow, the Hit-finished Necronomicon proc.
    DryBaby,
    /// Blue Fly 468AD0: target or orbit layer 1, steering.
    BlueFly,
    /// Any other case with its own Update, by address; not ported yet.
    Unique(u32),
}

pub(crate) fn update_case(variant: u32) -> UpdateCase {
    use UpdateCase::*;
    match variant {
        26 | 27 | 40 | 41 => Follower,
        1 | 4 | 5 | 6 | 7 | 9 | 10 | 11 | 12 | 54 => Shooter,
        8 => ShooterMirror,
        74 => ShooterMimic,
        13 | 76 => ShooterFree,
        30 | 31 | 32 | 33 | 35 | 44 | 60 | 69 => Orbital,
        45 | 70 => OrbitalShooter,
        46 | 47 | 56 => Chaser,
        71 | 72 => ChaserShooter,
        20 | 21 | 22 | 23 | 57 => Spawner,
        24 | 64 => Bum,
        51 => DryBaby,
        2 => Unique(0x46E760),
        3 | 59 => Unique(0x46A050),
        14 | 15 | 63 => Unique(0x46BE30),
        16 => Unique(0x46C2F0),
        17 => Unique(0x470DD0),
        25 => Unique(0x470B97),
        28 => Unique(0x46DBC0),
        42 => Unique(0x4714FF),
        43 => BlueFly,
        48 => Unique(0x469700),
        50 => Unique(0x46C8B0),
        52 => Unique(0x4712F7),
        53 => Unique(0x46A780),
        55 => Unique(0x4711E7),
        58 => Unique(0x47103A),
        61 => Unique(0x46EA60),
        62 => Unique(0x470F9D),
        65 => Unique(0x470F4F),
        66 => Unique(0x46CA90),
        67 => Unique(0x46D280),
        68 => Unique(0x470AF7),
        73 => Unique(0x468CB0),
        75 => Unique(0x46F4A0),
        77 => Unique(0x46ADD0),
        78 => Unique(0x470A30),
        79 => Unique(0x46D790),
        _ => None,
    }
}

/// One case of the Familiar FastUpdate 46F850 (jump table 0x66C5B8 for
/// variants up to 0x4B), run on the 60 Hz frames between full updates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InterpolateCase {
    /// 4446A0: the Entity half step.
    Base,
    /// 46F8BC: Follow(fast).
    Follow,
    /// 46F8E8: Follow(fast) while F58 is 0 (Little Chubby, Bob's Brain).
    FollowIdle,
    /// 46F880: Follow(fast) while the Float clip plays (the Full Key).
    FollowFloat,
    /// 46F900: the position takes the half-frame orbit point.
    Orbit,
    /// 46F948 (Robo-Baby 2.0) and 46F8D0 (Samson's Chains): a second
    /// update body; not ported.
    Unique(u32),
}

pub(crate) fn interpolate_case(variant: u32) -> InterpolateCase {
    use InterpolateCase::*;
    match variant {
        1
        | 2
        | 4..=7
        | 9..=12
        | 20..=23
        | 25..=27
        | 40
        | 41
        | 51
        | 52
        | 54
        | 55
        | 57
        | 61
        | 62
        | 74 => Follow,
        3 | 59 => FollowIdle,
        28 => FollowFloat,
        30..=33 | 35 | 44 | 45 | 60 | 65 | 69 | 70 => Orbit,
        53 => Unique(0x46F948),
        75 => Unique(0x46F8D0),
        _ => Base,
    }
}

/// 4666B0: the variants Init 46EFD7 links into the follower train.
pub(crate) fn is_follower(variant: u32) -> bool {
    matches!(
        variant,
        1..=7 | 9..=12 | 20..=23 | 25..=28 | 40 | 41 | 51 | 52 | 54 | 55 | 57 | 59 | 61 | 74
    )
}

/// Init's orbit layer F7C through 0x66C470 (variants 30..70).
pub(crate) fn orbit_layer(variant: u32) -> Option<usize> {
    match variant {
        32 | 33 | 35 | 44 | 45 | 50 | 69 | 70 => Some(0),
        43 | 65 | 68 => Some(1),
        31 => Some(2),
        30 | 60 => Some(3),
        _ => None,
    }
}

/// 0x8ECB00 (built by 40B040): each orbit layer's radius vector.
pub(crate) const ORBIT_RADIUS: [[f32; 2]; 4] =
    [[25.0, 20.0], [35.0, 30.0], [60.0, 45.0], [110.0, 90.0]];
/// 0x66CAA0: each layer's angle offset.
pub(crate) const ORBIT_OFFSET: [f32; 4] = [1.5, 0.5, 2.0, 1.2];
/// 0x66CAC0 with Guardian Angel (112) held, 0x66CAD0 otherwise.
pub(crate) const ORBIT_SPEED_FAST: [f32; 4] = [0.06, 0.045, -0.03, 0.02];
pub(crate) const ORBIT_SPEED: [f32; 4] = [0.03, 0.03, -0.02, 0.013];

/// Init 46EF36..46F1A0: the grid collision class (+0x340).
pub(crate) fn grid_class(variant: u32) -> u32 {
    match variant {
        46 | 47 | 48 | 56 | 71 | 72 | 73 => 5,
        75 => 6,
        _ => 0,
    }
}

/// Update tail 47064C: the interval of 444890's hit-set clear.
pub(crate) fn hit_interval(variant: u32) -> u32 {
    match variant {
        35 => 6,
        14 | 15 | 16 | 17 | 42 | 43 | 46 | 47 | 56 | 63 | 75 | 79 => 14,
        _ => 2,
    }
}

/// Update tail 4706BB: BFFS! (247) scales these sprites by 1.25.
pub(crate) fn bffs_scaled(variant: u32) -> bool {
    !matches!(variant, 43 | 66 | 73 | 79)
}

/// Collision callback 46FAEA/46FE40: variants that remove a projectile
/// they touch. Smart Fly 50 blocks only with Entity+18 = 0 (470188).
pub(crate) fn blocks_projectiles(variant: u32) -> bool {
    matches!(
        variant,
        17 | 32 | 33 | 35 | 42 | 44 | 45 | 50 | 51 | 65 | 67 | 68 | 75
    )
}

/// One tear of Fire 467170 for a variant: Spawn(2, `tear_variant`), the
/// falling speed Random*0.2 + `falling_base` (5417F0), damage (times BFFS!)
/// through vtable+60, flags 541810, scale 541FF0 and the color 42F480 /
/// 42F510 (`tint`, offsets in 1/255).
#[derive(Clone, Copy, Debug)]
pub(crate) struct FireParams {
    pub tear_variant: u32,
    pub damage: f32,
    pub scale: f32,
    pub flags: u64,
    pub falling_base: f32,
    pub tint: [f32; 4],
    pub offsets: [i32; 3],
    /// Harlequin 10 fires two tears at ±0.2 of the perpendicular (467490).
    pub double: bool,
    /// 467A81: Random(n) on the shared stream before the tear; 0 adds the
    /// charm flag 0x2000. Zero means no roll.
    pub charm_roll: u32,
    /// Rainbow Baby's row 8: the roll is still drawn, but the tear is
    /// always charmed (467A95 with R15 set).
    pub charm_always: bool,
    /// Rainbow Baby 11 (467622): Random(9) + 1 picks the row
    /// (`rainbow_row`) before anything else is drawn.
    pub rainbow: bool,
    /// Not ported here: Robo-Baby's laser 467010, Rotten Baby's flies
    /// 46E6B0, Rainbow Baby's Random(9) effect and Mongo Baby's mimic.
    pub ported: bool,
}

const fn tear(tear_variant: u32, damage: f32, scale: f32) -> FireParams {
    FireParams {
        tear_variant,
        damage,
        scale,
        flags: 0,
        falling_base: 0.0,
        tint: [1.0; 4],
        offsets: [0; 3],
        double: false,
        charm_roll: 0,
        charm_always: false,
        rainbow: false,
        ported: true,
    }
}

/// 4672B3..467B9A for a variant that is not Mongo Baby.
pub(crate) fn fire_params(variant: u32) -> FireParams {
    match variant {
        // 467642: Isaac's Head, flags 2.
        12 => FireParams {
            flags: 2,
            ..tear(0, 3.5, 0.8)
        },
        // 467350: Ghost Baby, spectral and tinted (1.5, 2, 2, 0.5).
        9 => FireParams {
            flags: 1,
            tint: [1.5, 2.0, 2.0, 0.5],
            ..tear(0, 3.5, 0.8)
        },
        // 467778: Harlequin Baby, blood tear 4.0 at 0.72, doubled.
        10 => FireParams {
            double: true,
            ..tear(1, 4.0, 0.72)
        },
        // 4678B8: Robo-Baby's tear row (its Shooter fires 467010 instead).
        6 => FireParams {
            ported: false,
            ..tear(1, 5.0, 0.848)
        },
        // 4678F0: Little Gish, slowing (0x100000008), tinted.
        4 => FireParams {
            flags: 0x1_0000_0008,
            tint: [0.95, 0.8, 0.6, 1.0],
            offsets: [-150, -150, -150],
            ..tear(0, 3.5, 0.8)
        },
        // 46796B: Little Steven, homing, falling base -3.
        5 => FireParams {
            flags: 4,
            falling_base: -3.0,
            tint: [0.4, 0.15, 0.38, 1.0],
            offsets: [90, 0, 180],
            ..tear(0, 3.5, 0.8)
        },
        // 4679EA: ???'s Soul, spectral homing, falling base -3.
        13 => FireParams {
            flags: 5,
            falling_base: -3.0,
            tint: [1.5, 2.0, 2.0, 0.5],
            ..tear(0, 3.5, 0.8)
        },
        // 467A4E: Cube of Meat L2, blood tear.
        45 => tear(1, 3.5, 0.8),
        // 467A81: Ball of Bandages L2..L4 roll Random(10) for charm; L4 72
        // also pierces (467B6B, 0x8000).
        70..=72 => FireParams {
            charm_roll: 10,
            flags: if variant == 72 { 0x8000 } else { 0 },
            tint: [1.0, 0.0, 1.0, 1.0],
            offsets: [50, 0, 0],
            ..tear(0, 3.5, 0.8)
        },
        // 467B14: Sister Maggy, tinted (0.9, 0, 0, 1).
        7 => FireParams {
            tint: [0.9, 0.0, 0.0, 1.0],
            ..tear(0, 3.5, 0.8)
        },
        // 4672C4: Rainbow Baby rolls its row in Fire.
        11 => FireParams {
            rainbow: true,
            ..tear(0, 3.5, 0.8)
        },
        // 4703EA: Rotten Baby; Mongo Baby's mimic.
        54 | 74 => FireParams {
            ported: false,
            ..tear(0, 3.5, 0.8)
        },
        // 46784E: Brother Bobby and every other shooter.
        _ => tear(0, 3.5, 0.8),
    }
}

/// Rainbow Baby's rows by Random(9) + 1 (4672D6..467848, the variant
/// tests failing for 11): 1 Ghost Baby's, 2 Harlequin's single tear, 3 a
/// blood tear (falling 1.2, 3.0 at 0.64, 467317), 4 Robo-Baby's tear row,
/// 5 Little Gish's, 6 Little Steven's, 7 ???'s Soul's, 8 the Ball of
/// Bandages row always charmed, 9 the plain tear doubled (R14, 467490).
pub(crate) fn rainbow_row(roll: u32) -> FireParams {
    match roll {
        1 => fire_params(9),
        2 => FireParams {
            double: false,
            ..fire_params(10)
        },
        3 => FireParams {
            falling_base: 1.2,
            ..tear(1, 3.0, 0.64)
        },
        4 => FireParams {
            ported: true,
            ..fire_params(6)
        },
        5 => fire_params(4),
        6 => fire_params(5),
        7 => fire_params(13),
        8 => FireParams {
            charm_always: true,
            ..fire_params(70)
        },
        _ => FireParams {
            double: true,
            ..tear(0, 3.5, 0.8)
        },
    }
}

/// One case of the Familiar room-clear callback 46DF10 (jump table
/// 0x66C340 over variants 20..57): after F90 rises, n is F90, or
/// trunc(F90 / divisor), optionally wrapped to signed n % 4; a pickup drops
/// when n is positive and even.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum RoomClearCase {
    None,
    Drop {
        divisor: Option<f32>,
        wrap4: bool,
        pickup: RoomClearPickup,
        /// 46E1CB: Little C.H.A.D. plays a sound on every clear.
        sound: Option<u32>,
    },
    /// 46E0F8: Juicy Sack's blue spiders; not ported.
    Unported(u32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum RoomClearPickup {
    Fixed { variant: u32, subtype: u32 },
    /// 46E07A: Random(4) picks heart, coin, key or bomb, and a second roll
    /// its subtype (Random(6/4/3/2) + 1), both before the n test.
    Mystery,
}

pub(crate) fn room_clear_case(variant: u32) -> RoomClearCase {
    use RoomClearPickup::*;
    let drop = |divisor, wrap4, variant, subtype, sound| RoomClearCase::Drop {
        divisor,
        wrap4,
        pickup: Fixed { variant, subtype },
        sound,
    };
    match variant {
        // 46DF48: Bomb Bag, a bomb every second n (F90 / 1.1).
        20 => drop(Some(1.1), false, 40, 0, None),
        // 46E0B0: Sack of Pennies, a coin every second room.
        21 => drop(None, false, 20, 0, None),
        // 46E190: Little C.H.A.D., sound 219 then a half heart (F90 / 1.1).
        22 => drop(Some(1.1), false, 10, 2, Some(219)),
        // 46E0C0: The Relic, a soul heart when (F90 / 1.11) % 4 is 2.
        23 => drop(Some(1.11), true, 10, 3, None),
        52 => RoomClearCase::Unported(0x46E0F8),
        // 46E050: Mystery Sack, (F90 / 1.11) % 4.
        57 => RoomClearCase::Drop {
            divisor: Some(1.11),
            wrap4: true,
            pickup: Mystery,
            sound: None,
        },
        _ => RoomClearCase::None,
    }
}
