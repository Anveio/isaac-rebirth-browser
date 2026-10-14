# Normal Isaac health damage

The existing contact, enemy-projectile, curse-door, spike and regular-bomb
callers now share the actual health-container branch of Player `52b080`.
They retain their own contact and immunity rules. A living Player with red
health zero can still take Soul damage; the actual dead flag controls rejection.

| Source branch | Implemented behavior |
| --- | --- |
| `52b7d0 → 52c0b6 → 52c0e0` | Normal primary Isaac requests SFX55, volume1/pitch1/delay2, before container mutation. Its three authored samples use the existing audio-owned shared draw and throttle. |
| `52bdf4 → 52c0f4 → 52c181` | Positive Soul health takes ordinary damage. Consumption is `min(Soul, damage)` with no red spill and no grant-side mask normalization. |
| `52be04..52be24` | Incoming flag32 chooses red with red>2, or red==2 and integer damage<=1; otherwise positive Soul still takes damage. |
| `52be2a..52be52` | The red path clears positive Eternal health first and subtracts it from damage, clamps the remainder to zero, then uses the existing signed red-heart helper. |
| `52c116..52c162`, `508290/508260` | Descending odd Soul positions clear the actual consumed Black slot before invoking forced35. An intact pair's first half does not trigger the effect. |
| `52c143`, `522250 → 523e5a` | Shared authored Book16 is requested before forced35's persistent temporary record, SFX33 and40HP/flags0/blank-source queued NPC damage. Entity318 mask0x10 targets are excluded; HP drains when the book host resumes the target's full callback. No main UseItem animation is requested. |
| `52c668..52c698` | The existing normal-death owner handles incoming NO_KILL and real Kill callbacks only after both red and Soul depletion. |
| `52c4f0 → 50a5c0 → 510510` | The ordinary branch force-plays authored Hit8 and clears active stateD80 after health and death dispatch. Fatal damage still requests Hit; the next actual Player callback selects Death. Controls and movement keep their own owners. |
| `5318a5`, `509540`, `5049ca → 50c880` | Bomb placement uses the actual dead flag; Chest opening consumes the sole collection predicate. Living red0/positive-Soul Players remain eligible. Hit and the item names permit generic collection through bf4; dead/Death reject it. |
| `504820 → 5054ca → 50c780` | Natural collectible selection separately rejects Pickup/LiftItem/HideItem/UseItem. Hit permits a new Pickup to take the main presentation; UseItem cannot admit a second collectible after retiring the previous Pickup guard. |

The forced35 target predicate reads the existing typed Fireplace, Keeper,
Monstro and Level2Fly flag owners. Unmodified ordinary Fly, AttackFly and
RoundWorm retain the initial flag4 contract; charm and other flag modifiers
remain separate source-qualified work. Their fields and subclass writes are
preserved. The temporary-record method is byte-identical to Active correction
`417f5ea5e5cc6f72a4ea9bd745b0049f7db3b248` and serves both actual45 and35.

Exact immutable original function and authored configuration hashes are in
[the source contract](../references/isaac/foundation/health-damage/source-contract.json).
All-target cargo check and strict Clippy pass with fresh private generated
metadata; a first shared-target generated-header failure is retained. The
frozen parent floor/WASM bytes are unchanged. This initial runtime slice has
no full production build or fresh original/browser acceptance yet.
The required `test:e2e:health-damage` gate declares public Soul72 and Black6
acquisition routes at both phase offsets, plus two OS seeds drawn and recorded
before grading without filtering. Contrived cases must acquire health and
damage it in both games, preserving red on Soul damage and executing guarded
Hit/SFX55. Black cases also require actual temporary35, queued40 targets,
Book16 freeze/completion and SFX33. It compares all selected functional
fields and keeps full original/browser evidence; browser manifests and traces
are lossless gzip archives with raw/archive hashes. Each case is attempted
separately so an abort cannot suppress later predeclared cases. Every mismatch
and aborted case remains a failure. No fresh gate execution is claimed;
source-only fixture predictions do not prove original execution.

The separate callback extension composes the shared Book16 host and Hit8
clock. Book16 uses GiantBook Shake35 and its actual BlackHeart sheet,
interrupts an existing book, and freezes room callbacks through state2. It
checks completion before advancing; the completion full callback resumes the
room. Hit completion uses the sole Player clock and actual current-clip guard.
The complete composed runtime `06e4093` passes its own all-target Rust
check (11.52s) and strict all-target Clippy (11.33s), using private generated
metadata. Earlier results remain scoped to the frozen initial container
slice. No production or paired acceptance is claimed.

Complete Hit/copy presentation, charged49 cancellation, complete HUD,
NO_KILL/flag32 public producers, alternate characters/co-op, hard-stage and
item/shield admission modifiers remain open. Bomb/Chest admission now uses
the mapped actual guard; fresh public living-red0/Soul-positive action
coverage is pending. The generic collection helper permits the source item
names without a blanket legacy Pickup veto. Collectible selection keeps its
separate item-clip rejection. Complete LiftItem/HideItem presentation and the
extra queued D80/overlay branches remain open.
No original state, time, RNG cursor or outcome enters the port.

The current source composition consumes reviewed Death `5afa7dd` and final
Heart `f782d64` APIs, preserving main98 shared pickup admission and the Root
name-based collection helper. Both main representations are now mutually
exclusive: Hit/UseItem/Death retire the old Pickup guard and clock; a new
Pickup replaces the concrete current main. The queued config/charge/touched
and copied item sprite remain independent. Current-main completion cannot
resume an interrupted Pickup. The gate additionally reads and compares
actual queued fields, copied clocks and exclusive ownership. All six tapes
and the two OS outcomes are retained unchanged; the composed private Rust
checks pass, while production and fresh paired acceptance remain pending.


Current source integration includes merged main Health115/Info/Hit and
Nightmare124, with actual room counts119, strict negative NPC death123,
Heart109 and the public keyboard122 preserved. Private Rust check9.51s and
strict Clippy10.36s qualify exact composition `defb1091`; the final narrow
skipped-music guard and Soul-only action restack have source/type/format/matrix
qualification separately. Production and fresh health execution remain pending.

The health gate requires all four sealed Soul72/Black6 phase tapes and two to
four recorded OS outcomes before either game starts. It rejects empty/missing
cohorts, duplicate IDs, altered fixed seeds/phase tapes and changed OS routes.
Mandatory coverage is chosen by the sealed ID even when replay requirements
are empty. Fourteen ignored no-game checks pass; all original six inputs are
byte-identical when rebuilt with their recorded words, with no new entropy.
NPC membership follows active31e and compares dead31f independently, preserving
dead active actors until retirement. Unsupported217, missing58/43 callbacks
and actual living-red0/Soul-positive actions remain required future coverage;
no source or preflight result is live acceptance.
