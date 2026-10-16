#!/usr/bin/env python3
"""Import the original base-Rebirth pre-boss presentation sheets and catalogue.

Only paths present in the selected base graphics archive are imported. The
original PNG bytes are preserved; animation/name/pivot data is not synthesized.
"""
import argparse
import importlib.util
import hashlib
import json
from pathlib import Path
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path, default=ROOT / 'references/isaac/linux/depot_250903/resources/packed/graphics.a')
    parser.add_argument('--filenames', type=Path, default=ROOT / 'references/isaac/resource-filenames.txt')
    parser.add_argument('--portraits', type=Path, default=ROOT / 'references/isaac/rebirth/resources/bossportraits.xml')
    parser.add_argument('--manifest', type=Path, default=ROOT / 'references/isaac/manifest.json')
    parser.add_argument('--animations', type=Path, default=ROOT / 'web/npc-data.json')
    parser.add_argument('--extractor', type=Path, default=ROOT / 'tools/foundation/extract-animations.py')
    parser.add_argument('--output', type=Path, default=ROOT)
    args = parser.parse_args()
    spec = importlib.util.spec_from_file_location('base_rebirth_original_assets', args.extractor)
    assets = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(assets)
    _data, _compression, entries = assets.archive_entries(args.archive)
    available = {(entry[0], entry[1]) for entry in entries}
    del _data
    prefix = 'resources/gfx/ui/boss/'
    candidates = {line.lower() for line in args.filenames.read_text().splitlines()
                  if line.lower().startswith(prefix) and line.lower().endswith('.png')}
    actor_path = 'gfx/ui/boss/versusscreen.anm2'
    actor = json.loads(args.animations.read_text())[actor_path]
    required_sheets = {prefix + sheet['Path'].lower() for sheet in actor['Spritesheets']}
    for path in required_sheets:
        if (assets.path_hash(path), assets.path_hash_b(path)) not in available:
            raise ValueError('Original versus-screen sheet is absent from the selected archive: ' + path)
    candidates.update(required_sheets)
    bosses = []
    for row in ET.parse(args.portraits).getroot():
        if row.tag != 'boss':
            continue
        portrait = row.attrib['portrait']
        path = prefix + portrait.lower()
        candidates.add(path)
        if (assets.path_hash(path), assets.path_hash_b(path)) not in available:
            raise ValueError('Original boss portrait is absent from the selected archive: ' + portrait)
        bosses.append(dict(id=int(row.attrib['id']), name=row.attrib['name'],
            portrait=path.removeprefix('resources/'),
            pivotX=int(row.attrib['pivotX']), pivotY=int(row.attrib['pivotY'])))
    if not bosses or len({row['id'] for row in bosses}) != len(bosses):
        raise ValueError('A nonempty original boss catalogue with unique IDs is required')
    paths = sorted(path for path in candidates
        if (assets.path_hash(path), assets.path_hash_b(path)) in available)
    # Reuse the base-archive decoder. No image conversion/resizing/recompression.
    temporary_root = args.output / '.cache/boss-splash-import'
    imported = assets.extract_named_assets(args.archive, paths, temporary_root)
    destination = args.output / 'references/isaac/rebirth'
    for path in imported:
        target = destination / path
        target.parent.mkdir(parents=True, exist_ok=True)
        original = (temporary_root / path).read_bytes()
        # Older reference exports retain up to three archive alignment bytes.
        # Preserve those exports when the original PNG datastream is identical.
        existing = target.read_bytes() if target.exists() else b''
        aligned_original = (len(existing) % 4 == 0 and
            0 <= len(existing) - len(original) <= 3 and existing.startswith(original))
        if not aligned_original:
            target.write_bytes(original)
    manifest = json.loads(args.manifest.read_text())
    inventory = {row['path']: row for row in manifest['files']}
    added = []
    for path in imported:
        target = destination / path
        name = target.relative_to(args.output).as_posix()
        if name not in inventory:
            added.append(name)
        payload = target.read_bytes()
        inventory[name] = dict(path=name, bytes=len(payload), sha256=hashlib.sha256(payload).hexdigest())
    manifest['files'] = [inventory[row['path']] for row in manifest['files']] + [inventory[path] for path in sorted(added)]
    if added:
        records = manifest['source'].setdefault('supplementary_extractions', [])
        records.append(dict(kind='Missing base pre-boss presentation assets and inventory completion',
            extractor='tools/foundation/import-boss-splash.py',
            decoder_sha256=hashlib.sha256(args.extractor.read_bytes()).hexdigest(),
            archive='graphics.a', archive_sha256=hashlib.sha256(args.archive.read_bytes()).hexdigest(),
            output_paths=sorted(added)))
    output_manifest = args.output / 'references/isaac/manifest.json'
    output_manifest.parent.mkdir(parents=True, exist_ok=True)
    output_manifest.write_text(json.dumps(manifest, indent=2) + '\n')
    normalized = [path.removeprefix('resources/') for path in imported]
    catalog = dict(schema=1, actor=actor_path, assets=normalized,
        nameAtlas='gfx/ui/boss/vsnames.png', bosses=bosses,
        playerPortraits=[path for path in normalized if Path(path).name.startswith('playerportrait_')],
        playerSpots=[path for path in normalized if Path(path).name.startswith('playerspot')],
        bossSpots=[path for path in normalized if Path(path).name.startswith('bossspot')])
    output = args.output / 'web/boss-splash-data.json'
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(catalog, separators=(',', ':')) + '\n')
    print(json.dumps(dict(originalSheets=len(imported), bossPresentations=len(bosses),
        playerPortraits=len(catalog['playerPortraits']), playerSpots=len(catalog['playerSpots']),
        bossSpots=len(catalog['bossSpots']), catalogue=str(output))))


if __name__ == '__main__':
    main()
