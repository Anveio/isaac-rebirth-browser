"""Decode the pinned base collectible/pool resources for production consumers."""
import argparse
import hashlib
import json
import math
from pathlib import Path
import struct
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]
RESOURCES = ROOT / 'references/isaac/rebirth/resources'
PINS = {'items.xml': '4fbad99276eebf0ed5dcba7e88591c03673bbae44c4cd8b5dd3ead36caad68cc',
        'itempools.xml': '8c3d8f04589992f1bac0c4f719c49068265d5fe134d29f414d90161e7bf95378',
        'costumes2.xml': '663f4653e1f753a1e16e5db6ee3ea4d9fedaca9df5c4f02f9a473e47b840c808'}
POOLS = ['treasure', 'shop', 'boss', 'devil', 'angel', 'secret', 'library', 'challenge',
         'goldenChest', 'redChest', 'beggar', 'demonBeggar', 'curse', 'keyMaster', 'bossrush', 'dungeon']
CACHE = dict(zip(['damage', 'firedelay', 'shotspeed', 'range', 'speed', 'tearflag',
                  'tearcolor', 'flying', 'weapon', 'familiars', 'luck'], [1 << i for i in range(11)]))
INVENTORY = ['maxhearts', 'hearts', 'soulhearts', 'blackhearts', 'bombs', 'keys', 'coins']


def float32(text):
    value = struct.unpack('<f', struct.pack('<f', float(text)))[0]
    if not math.isfinite(value) or value < 0:
        raise ValueError('Invalid authored pool scalar')
    return value


def catalogue():
    trees = {}
    for name, expected in PINS.items():
        data = (RESOURCES / name).read_bytes()
        if hashlib.sha256(data).hexdigest() != expected:
            raise ValueError(f'Unexpected base resource: {name}')
        trees[name] = ET.fromstring(data)
    definitions = []
    trinket_achievements = [None] * 62
    trinkets = []
    for item in trees['items.xml']:
        if item.tag == 'trinket':
            item_id = int(item.attrib['id'])
            achievement = int(item.get('achievement', '-1'))
            if not 0 < item_id < 62 or item_id == 47 or trinket_achievements[item_id] is not None:
                raise ValueError('Unexpected sparse base trinket identity')
            if not -1 <= achievement < 179:
                raise ValueError('Trinket references an undefined achievement byte')
            trinket_achievements[item_id] = achievement
            # Base ItemConfig Kind2 is a separate ID namespace. Metadata is
            # not an admission of registration, held effects or pickup350.
            if set(item.attrib) - {'id', 'name', 'description', 'gfx', 'achievement', 'cache'}:
                raise ValueError('Unexpected authored base trinket field')
            flags = 0
            for token in item.get('cache', '').split():
                flags |= 0xffffffff if token == 'all' else CACHE[token]
            graphic = 'gfx/items/trinkets/' + item.attrib['gfx'].lower() if item.get('gfx') else None
            if graphic and not (RESOURCES / graphic).is_file():
                raise ValueError(f'Missing authored trinket graphic: {graphic}')
            trinkets.append({'id': item_id, 'name': item.attrib['name'],
                'description': item.attrib['description'], 'graphic': graphic,
                'achievement': achievement, 'cacheFlags': flags,
                # Parser57E021 initializes config+60 empty; the pinned base
                # Kind2 nodes have no ANM field. Registration remains owned
                # by its caller rather than inferred from these definitions.
                'costumeAnm2': ''})
            continue
        kind = {'passive': 1, 'active': 3, 'familiar': 4}[item.tag]
        flags = 512 if kind == 4 else 0
        for token in item.get('cache', '').split():
            flags |= 0xffffffff if token == 'all' else CACHE[token]
        special = item.get('special', 'false')
        if special not in ('false', 'true', '0', '1'):
            raise ValueError('Invalid authored special flag')
        graphic = 'gfx/items/collectibles/' + item.attrib['gfx'].lower() if item.get('gfx') else None
        if graphic and not (RESOURCES / graphic).is_file():
            raise ValueError(f'Missing authored collectible graphic: {graphic}')
        definitions.append({'id': int(item.attrib['id']), 'kind': kind,
            'name': item.attrib['name'], 'description': item.attrib['description'], 'graphic': graphic,
            'achievement': int(item.get('achievement', '-1')), 'cacheFlags': flags,
            'inventory': [int(item.get(key, '0')) for key in INVENTORY],
            'maxCharges': int(item.get('maxcharges', '0')), 'cooldown': int(item.get('cooldown', '0')),
            'special': special in ('true', '1')})
    # Missing configs0/47 differ from a present config with no prerequisite.
    # Reset5820A0 treats both as eligible, then disables their availability.
    if [i for i, value in enumerate(trinket_achievements) if value is None] != [0, 47]:
        raise ValueError('Unexpected sparse base trinket catalogue')
    trinkets.sort(key=lambda item: item['id'])
    definitions.sort(key=lambda item: item['id'])
    ids = {item['id'] for item in definitions}
    if len(ids) != 343 or len(definitions) != 343 or min(ids) != 1 or max(ids) != 346:
        raise ValueError('Unexpected sparse base collectible catalogue')
    authored = {pool.attrib['Name']: pool for pool in trees['itempools.xml']}
    if set(authored) != set(POOLS) - {'challenge'}:
        raise ValueError('Unexpected authored pool identity')
    pools = []
    for index, name in enumerate(POOLS):
        entries = []
        for entry in authored.get(name, []):
            item_id = int(entry.attrib['Id'])
            if item_id not in ids:
                raise ValueError('Pool references an undefined base collectible')
            entries.append({'id': item_id, 'weight': float32(entry.attrib['Weight']),
                'decreaseBy': float32(entry.attrib['DecreaseBy']), 'removeOn': float32(entry.attrib['RemoveOn'])})
        pools.append({'id': index, 'name': name, 'entries': entries})
    # Collectible costume presence is configuration metadata, not evidence
    # that an Add call passed57BF00 or that any selected pose is implemented.
    costume_ids = sorted({int(node.attrib['id']) for node in trees['costumes2.xml']
                          if node.get('type') in ('passive', 'active', 'familiar')
                          and node.get('anm2path', '')})
    if not set(costume_ids) <= ids:
        raise ValueError('Costume references an undefined base collectible')
    return {'schema': 1, 'sourceHashes': PINS, 'collectibles': definitions,
            'pools': pools, 'costumeCollectibleIds': costume_ids,
            'trinketAchievements': trinket_achievements, 'trinkets': trinkets}


def rust(data):
    def scalar(value):
        return f'f32::from_bits(0x{struct.unpack("<I", struct.pack("<f", value))[0]:08x})'
    lines = [f"pub const COSTUME_COLLECTIBLE_IDS: &[u32] = &{data['costumeCollectibleIds']!r};",
             'pub const COLLECTIBLES: &[CollectibleDefinition] = &[']
    for item in data['collectibles']:
        strings = ', '.join(f'{key}: {json.dumps(item[key], ensure_ascii=False)}' for key in ['name', 'description'])
        graphic = f'Some({json.dumps(item["graphic"], ensure_ascii=False)})' if item['graphic'] else 'None'
        lines.append(f"CollectibleDefinition {{ id: {item['id']}, kind: {item['kind']}, {strings}, graphic: {graphic}, achievement: {item['achievement']}, cache_flags: {item['cacheFlags']}, inventory: {item['inventory']!r}, max_charges: {item['maxCharges']}, cooldown: {item['cooldown']}, special: {str(item['special']).lower()} }},")
    lines.append('];\npub const POOL_DEFINITIONS: &[PoolDefinition] = &[')
    for pool in data['pools']:
        lines.append(f"PoolDefinition {{ id: {pool['id']}, name: {json.dumps(pool['name'])}, entries: &[")
        lines.extend(f"PoolEntryDefinition {{ id: {entry['id']}, weight: {scalar(entry['weight'])}, decrease_by: {scalar(entry['decreaseBy'])}, remove_on: {scalar(entry['removeOn'])} }}," for entry in pool['entries'])
        lines.append('] },')
    lines.append('];')
    prerequisites = ', '.join('None' if value is None else f'Some({value})'
                              for value in data['trinketAchievements'])
    lines.append(f'pub const TRINKET_ACHIEVEMENTS: [Option<i32>; 62] = [{prerequisites}];')
    lines.append('pub const TRINKETS: &[TrinketDefinition] = &[')
    for item in data['trinkets']:
        strings = ', '.join(f'{key}: {json.dumps(item[key], ensure_ascii=False)}' for key in ['name', 'description'])
        graphic = f'Some({json.dumps(item["graphic"], ensure_ascii=False)})' if item['graphic'] else 'None'
        lines.append(f"TrinketDefinition {{ id: {item['id']}, {strings}, graphic: {graphic}, achievement: {item['achievement']}, cache_flags: {item['cacheFlags']}, costume_anm2: {json.dumps(item['costumeAnm2'])} }},")
    lines.append('];')
    return '\n'.join(lines) + '\n'


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--rust', action='store_true', help='Emit the production Rust catalogue')
    parser.add_argument('--check', action='store_true', help='Validate all pinned authored records')
    args = parser.parse_args()
    data = catalogue()
    if args.check:
        print(f"Validated {len(data['collectibles'])} collectibles and {len(data['pools'])} native pool slots")
    else:
        print(rust(data) if args.rust else json.dumps(data, separators=(',', ':')), end='\n')
