#!/usr/bin/env python3
"""Generate private common-SKU fixtures from locally available Base resources.

This source-only recipe ships no original room records or XML. Generated STBs
stay below the repository's ignored .cache; this tool never executes a game.
"""
import argparse
import copy
import importlib.util
import json
from pathlib import Path


MODES = ('free-no-active', 'free-full', 'free-undercharged',
         'paid-no-active', 'paid-full', 'paid-undercharged', 'paid100')


def group(x, y, variant, subtype):
    return dict(x=x, y=y, entries=[dict(type=5, variant=variant,
                                      subtype=subtype, weight=1.0)])


def additions(mode):
    start = [group(6, 2, 90, 0) if mode.startswith('free-') else
             group(6, 2, 20, 3 if mode == 'paid100' else 2)]
    if 'full' in mode or 'undercharged' in mode:
        start.append(group(7, 3, 100, 45))
    stock = [group(3, 4, 150, 0), group(9, 4, 150, 0)] if mode.startswith('paid-') else []
    return {80: start, 81: stock}


def prepare(root, output, selected):
    root, output = root.resolve(), output.resolve()
    cache = (root / '.cache').resolve()
    if cache not in output.parents:
        raise ValueError('Generated private fixtures must stay below ignored .cache')
    if output.exists():
        raise ValueError('Output must be fresh; existing evidence is never replaced')
    decoder_path = root / 'tools/foundation/decode-rooms.py'
    spec = importlib.util.spec_from_file_location('shop_sku_rooms', decoder_path)
    decoder = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(decoder)
    resources = root / 'references/isaac/rebirth/resources'
    source = resources / 'rooms/00.special rooms.stb'
    xml = resources / 'challenges.xml'
    original = decoder.decode(source)
    if decoder.encode_bytes(original) != source.read_bytes():
        raise ValueError('Base STB must round-trip through the existing decoder')
    if len(original) != 492 or (original[80]['type'], original[80]['variant']) != (1, 2):
        raise ValueError('This recipe requires the declared Base start-room owner')
    shop = original[81]
    if (shop['type'], shop['variant']) != (2, 0):
        raise ValueError('This recipe requires the declared ordinary Shop owner')
    selectors = [entry for spawn in shop['spawns'] for entry in spawn['entries']
                 if (entry['type'], entry['variant']) == (5, 150)]
    if len(selectors) != 2 or not xml.is_file():
        raise ValueError('Base ordinary Shop or unchanged challenge XML is unavailable')
    built, records = {}, []
    for mode in selected:
        rooms = copy.deepcopy(original)
        appended = additions(mode)
        for index, groups in appended.items():
            rooms[index]['spawns'].extend(groups)
            cells = [(spawn['x'], spawn['y']) for spawn in rooms[index]['spawns']]
            if len(cells) != len(set(cells)):
                raise ValueError('Recipe cannot overlap an authored spawn cell')
        for index, (before, after) in enumerate(zip(original, rooms)):
            if index not in appended and before != after:
                raise ValueError('Recipe changed an undeclared room')
            if index in appended:
                if {k: v for k, v in before.items() if k != 'spawns'} != {
                        k: v for k, v in after.items() if k != 'spawns'}:
                    raise ValueError('Recipe changed room metadata or doors')
                if after['spawns'] != before['spawns'] + appended[index]:
                    raise ValueError('Recipe changed original spawn order')
        encoded = decoder.encode_bytes(rooms)
        if decoder.decode_bytes(encoded) != rooms:
            raise ValueError('Generated private fixture failed structural round-trip')
        built[mode] = encoded
        records.append(dict(name='battery90-' + mode,
                            recipe=dict(schema='shop-common-sku-local-v1', mode=mode),
                            specialRoomsPath=str(output / (mode + '.stb')),
                            authoredResourcePath=str(xml), challengeId=0,
                            startingItems=[], challengeConfigurationUsed=False))
    output.mkdir(parents=True)
    for mode, encoded in built.items():
        (output / (mode + '.stb')).write_bytes(encoded)
    (output / 'local-fixtures.json').write_text(json.dumps(dict(
        schema=1, status='PREPARED_NOT_EXECUTED', cases=records,
        scope='Locally reconstructed private resources; source-only recipe, no game evidence.'), indent=2) + '\n')
    return dict(status='PREPARED_NOT_EXECUTED', cases=len(records), output=str(output))


def main():
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument('--repository-root', type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--case', choices=MODES, action='append', required=True)
    args = parser.parse_args()
    if len(args.case) != len(set(args.case)):
        parser.error('Each recipe can be selected only once')
    print(json.dumps(prepare(args.repository_root, args.output, args.case)))


if __name__ == '__main__':
    main()
