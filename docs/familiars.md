The Familiar resource catalogue contains the original base-Rebirth Leech3/56,
appearance Poof1000/15, and healing Heart1000/49 animations. The existing pinned
resource inventory owns all five PNG sheets; the browser now bundles them and
loads the extracted actor metadata through the same animation renderer.

Regenerate the metadata with `python3 tools/foundation/extract-animations.py
--select-file tools/foundation/familiar-animations.txt --out
web/familiar-data.json`. The recorded catalogue in
`references/isaac/foundation/familiars/authored-catalogue.json` pins the archive,
extractor, metadata, sheets, clip lengths and loop flags. Leech defaults to
Move Hori12 loop, Poof43 is nonlooping, and Heart8 is nonlooping.

This resource prerequisite does not accept Familiar admission, appearance,
movement, contact or healing parity. Those require actual native/browser
public-input evidence and the original cache, RNG and actor lifecycle owners.
