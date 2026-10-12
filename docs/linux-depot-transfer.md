# Base Rebirth Linux depot transfer

Downloaded through the owner's logged-in Steam client on 2026-10-12:

- App: `250900`
- Linux depot: `250903`
- Manifest: `367036646469636831`
- Steam reported completion. Steam counted 17 entries; the depot contains 14 regular files, including x86_64 and i386
  Linux executables, Steam API libraries, launch scripts and packed resources.
- The installed Mac game was not modified.

This private repository carries the exact downloaded files as a gzip tar archive
split into parts below GitHub's per-file limit. Keep the repository private.
`references/isaac/linux/manifest.json` records each original file's size and SHA-256;
`transfer/parts.json` records checksums for the transfer parts.

In the cloud checkout, fetch the latest repository state, preserving any existing
cloud changes, then run with Python 3.12 or later:

```sh
python3 tools/restore-linux-depot.py
file references/isaac/linux/depot_250903/isaac.x64
```

The script verifies every part, extracts the depot, and verifies all 14 files.
The resulting directory is `references/isaac/linux/depot_250903`.
Launch from that directory using `./run-x64.sh` after checking Linux dependencies.

No Linux launch, dependency verification, or cloud ingestion is claimed by this
transfer. The cloud environment should perform those checks with its configured
display and preserve its network/proxy/CA policy. Base Rebirth remains separate
from Afterbirth and Afterbirth+.
