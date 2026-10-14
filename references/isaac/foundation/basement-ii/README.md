The runtime generator now accepts normal Basement I and II requests through
`floor::generate_normal_basement`. The existing first-floor game entry calls
the same generator with fresh Isaac inputs. Basement II uses the continuing
run's retained stage seed, game flags, red/Soul health and coins; generation
does not initialize a new run or supply native descriptors.

The second floor uses difficulty 5–10, eleven or twelve additional rooms and
at least six dead ends. Its shape eligibility comes from the authored Basement
catalog. The even-floor boss branch reads the family flags saved on the first
floor without repeating the odd-floor family draw. Optional rooms use the
actual continuing health and coin predicates. Template weights, both local
streams and game flags survive failed placement attempts.

This is a source-mapped generator prerequisite. Actual second-floor admission
remains pending the concrete Grid17 exit callback, stage/Scene controller,
Player callbacks and a fresh original/browser public progression gate. There
is no new component harness or Stage command. Existing first-floor live
evidence belongs to its recorded producer; it does not establish Basement II
parity or acceptance of this composition.

The bounded path excludes alternate Cellar, Labyrinth, challenges, equipped
modifiers, later stages and profile unlock changes. Generated larger room
shapes retain their authored metadata; their unimplemented actor or lifecycle
behavior remains a required failure in the game-level gates.
