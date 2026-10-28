#!/usr/bin/env python3
"""Import every item's authored animation data at build time.

One pass over the owner's local Base animation cache selects the actor of every
costume in costumes2.xml and every Familiar (entity type 3) in entities2.xml,
and writes them with a catalogue (costumes, Familiars, and every collectible
and trinket with its name and pools) into the ignored web/generated tree.
The page, the sprite pack and the engine build read these files; nothing here
is committed. Items then gain behaviour as rows and mechanisms, not as new
per-item extractions.
"""
import argparse
import importlib.util
import json
from pathlib import Path
import sys
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]
RESOURCES = ROOT / 'references/isaac/rebirth/resources'
FLAGS = ['isFlying', 'overwriteColor', 'hasSkinAlt', 'hasOverlay']


def write(path, value):
    # Replace the generated file without modifying a possibly shared inode.
    temporary = path.with_suffix('.json.tmp')
    temporary.unlink(missing_ok=True)
    with temporary.open('x') as stream:
        stream.write(json.dumps(value, separators=(',', ':')) + '\n')
    temporary.replace(path)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path, default=ROOT / 'references/isaac/linux/depot_250903/resources/packed/animations.a')
    parser.add_argument('--filenames', type=Path, default=ROOT / 'references/isaac/resource-filenames.txt')
    parser.add_argument('--output', type=Path, default=ROOT / 'web/generated/item-assets')
    args = parser.parse_args()
    sys.dont_write_bytecode = True
    spec = importlib.util.spec_from_file_location('rebirth_animation_reader', ROOT / 'tools/foundation/extract-animations.py')
    decoder = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(decoder)

    names = {decoder.path_hash(path): path.removeprefix('resources/') for path in args.filenames.read_text().splitlines()
             if path.startswith('resources/') and path.endswith('.anm2')}
    binary = decoder.load_archive(args.archive)
    reader = decoder.Reader(binary)
    actors = {}
    for _ in range(reader.count(5000)):
        name, actor = reader.u32(), decoder.actor(reader)
        path = names.get(name)
        if path is not None:
            actors[path.lower()] = (path, actor)
    if reader.at != len(binary):
        raise ValueError('Incomplete Base animation cache')

    def find(path, owner):
        found = actors.get(path.replace('\\', '/').lower())
        if found is None:
            raise ValueError(f'{owner}: authored actor {path} is not in the animation cache')
        return found

    costumes_xml = ET.parse(RESOURCES / 'costumes2.xml').getroot()
    root = costumes_xml.attrib['anm2root'].removeprefix('resources/')
    costume_actors, costumes = {}, []
    for node in costumes_xml.iter('costume'):
        key, actor = find(root + node.attrib['anm2path'], f"costume {node.attrib['id']}")
        costume_actors[key] = actor
        row = {'id': int(node.attrib['id']), 'type': node.attrib['type'], 'actor': key}
        if 'priority' in node.attrib:
            row['priority'] = int(node.attrib['priority'])
        if 'skinColor' in node.attrib:
            row['skinColor'] = int(node.attrib['skinColor'])
        row.update({flag: True for flag in FLAGS if node.attrib.get(flag) == 'true'})
        costumes.append(row)

    familiar_actors, familiars = {}, []
    for node in ET.parse(RESOURCES / 'entities2.xml').getroot().iter('entity'):
        if node.attrib.get('id') != '3':
            continue
        key, actor = find('gfx/' + node.attrib['anm2path'], f"familiar {node.attrib['variant']}")
        familiar_actors[key] = actor
        familiars.append({'variant': int(node.attrib['variant']), 'name': node.attrib['name'], 'actor': key})

    pools = {}
    for pool in ET.parse(RESOURCES / 'itempools.xml').getroot().iter('Pool'):
        for item in pool.iter('Item'):
            pools.setdefault(int(item.attrib['Id']), []).append(pool.attrib['Name'])
    collectibles, trinkets = [], []
    for node in ET.parse(RESOURCES / 'items.xml').getroot():
        if node.tag == 'trinket':
            trinkets.append({'id': int(node.attrib['id']), 'name': node.attrib['name']})
        elif node.tag in ('passive', 'active', 'familiar'):
            item = int(node.attrib['id'])
            collectibles.append({'id': item, 'name': node.attrib['name'], 'kind': node.tag, 'pools': pools.get(item, [])})

    args.output.mkdir(parents=True, exist_ok=True)
    write(args.output / 'costume-data.json', costume_actors)
    write(args.output / 'familiar-data.json', familiar_actors)
    write(args.output / 'catalogue.json', {'schema': 1, 'costumes': costumes, 'familiars': familiars,
                                               'collectibles': collectibles, 'trinkets': trinkets})
    print(f'Imported {len(costumes)} costumes ({len(costume_actors)} actors), '
          f'{len(familiars)} Familiars ({len(familiar_actors)} actors), {len(collectibles)} collectibles '
          f'and {len(trinkets)} trinkets into {args.output.relative_to(ROOT)}')


if __name__ == '__main__':
    main()
