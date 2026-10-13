# Recovered Basement backdrop assembly

The port reproduces the original Basement starting-room backdrop when given
the room's decoration seed. This contract covers patch selection, every
submitted source/destination vertex, and mirroring. Run-to-room seed routing,
floor topology, and FX generation remain unresolved.

The evidence uses Linux depot `250903`, manifest `367036646469636831`.
`tools/original/compat/unwrap.py` restores the wrapper payload into a private
working copy; it checks the licensed source hash, payload round trip, entry
point, and allowed changed ranges. The original source remains unchanged.

| Evidence | SHA-256 |
| --- | --- |
| Licensed Linux `isaac.x64` | `c1b2ca9375d60d4354f9086871913901428af41b82a5892a0d235c5d8ae2f768` |
| Restored Linux executable used for static and live observations | `724b9c59ad4dc7964852a51728c0f76d2031a3dd3d0d6ba2dcade1faa7ea7d13` |
| Authored `backdrops.xml` | `9f2cfa5d3e5fbfe1617d3334b34eb213d41ee7b2ec2889d51fdd6848ad2d990b` |

The Linux executable is stripped. The following routine roles were identified
from configuration references, call relationships, the public RNG contracts,
and independent runtime output. Addresses refer to the restored Linux image.
The separate Mach-O symbol inventory names these subsystems, but its wrapped
code bytes were not used to derive these formulas.

| Routine role | Linux entry | Recovered behavior |
| --- | --- | --- |
| RNG seed / next / integer random | `0x5c2d40` / `0x5c2d70` / `0x5c2dd0` | Unsigned xorshift; integer random advances once then applies modulo. |
| Backdrop seed | `0x4271b0` | Selects shift index 35: right 5, left 9, right 7. |
| Backdrop graphics initialization | `0x42a170` | Calls floor assembly before wall assembly on the same decoration stream. |
| Floor assembly | `0x428020` | Consumes `Random(floors)` even when `floors=1`; constructs six authored floor variants. |
| Floor catalogue repetition / shuffle | `0x428a50` / `0x428ae0` | Doubles the catalogue until it has at least 32 entries, giving 48 Basement entries. Fisher–Yates visits indices 47 through 1 and swaps with `Random(index+1)`. |
| Wall assembly | `0x424440` | Consumes `Random(walls)` even when `walls=1`. Each corner/edge draws `Random(3)`; zero consumes another `Random(3)` for the variant, otherwise the plain variant is used. |

The four overlapping floor quads use shuffled catalogue entries `0, 1, 2, 2`.
Wall submission order is four corners (top left, top right, bottom left, bottom
right), thirteen pairs of top/bottom cells, and seven pairs of left/right cells.
Corners also permit nonzero variants. The wall rule has nominal weights 7:1:1,
but replacing its conditional draws with a weighted picker changes subsequent
stream state. The previous independent floor draws, fixed plain corners, and
empirical 8:1:1 picker therefore produced different original sequences.

Four independent fresh-profile runs passed complete 4-floor/44-wall comparisons:

| Entered run seed | Observed room decoration seed |
| --- | --- |
| `N4Z8 GWV3` | `4271600350` |
| `P01M XJW0` | `1985222680` |
| `B29Z RLP1` | `2463833516` |
| `PNWJ RQD3` | `1082003586` |

The first two runs verified the static derivation; the latter two were newly
selected after implementation. The bounded read-only descriptor observer checks
each decoration input against the actual original spawn log and stage seed
chain. The public GL observer records the original cache submissions. No
observed patch map or run-seed offset is used by production code.

The maximum source-coordinate difference was `0.00003072` pixels; destination
coordinates matched exactly. Submitted UVs differ from authored coordinates by
at most one float32 spacing in these observations, and the observer serializes
them with `%.9g`. The verifier expresses that bound in the normalized UV domain,
adds half the least printed decimal, then scales by the observed 512-pixel
texture extent. It also checks individual vertices, so equal rectangle bounds
cannot conceal a skewed quad.

The conditional backdrop verifier and its standalone CLI are now ignored
research. The production generator and original-only observations remain;
these earlier cache-assembly checks do not establish run-to-room routing.
`npm run test:e2e` captures both live games and retains complete image
differences, while selected functional contracts gate runtime progress.

Production `generate_room(run_seed)` still substitutes the Stage 1 seed for the
unrecovered room decoration seed and labels this route provisional. Observed
initial-room stage-stream offsets varied, and the level placement routines have
conditional RNG consumption. Recovering those general rules is required before
claiming arbitrary run-seed room parity.

Independent left, right, up, and down wall observations also recover Isaac's
center limits: native world X `70..570`, Y `150..410`, or logical X
`77.5..402.5`, Y `51.5..220.5`. The implementation derives these from the floor
quad extents and room origin, subtracts the actor-to-sprite ground offset, then
adds circle clearance. `entities2.xml` supplies the player's radius of 10 world
units, giving 6.5 logical pixels at the independently observed 0.65 projection.
This corrects all four center limits without a seed-specific boundary rule.
Native contact-velocity resolution and door traversal remain provisional.
`projectileBounds` preserves the separate projected tear limits; a player's
circle clearance is not evidence for a tear's collision radius or height.
