#!/usr/bin/env python3
"""Export base Rebirth pickup selectors and item art from authored XML."""
import argparse
import hashlib
import json
from pathlib import Path
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]


def extract(resources):
    entities_path = resources / 'entities2.xml'
    items_path = resources / 'items.xml'
    pockets_path = resources / 'pocketitems.xml'
    entities = ET.parse(entities_path).getroot()
    pickups = []
    for entity in entities.findall('entity'):
        if entity.get('id') != '5':
            continue
        pickups.append({
            'variant': int(entity.get('variant', '0')),
            'subtype': int(entity.get('subtype', '0')),
            'name': entity.get('name'),
            'actor': 'gfx/' + entity.attrib['anm2path'].replace('\\', '/').lower(),
            'collisionRadius': float(entity.attrib['collisionRadius']),
            'shadowSize': float(entity.attrib['shadowSize']),
        })
    items = []
    for item in ET.parse(items_path).getroot():
        if 'id' not in item.attrib:
            continue
        sprite = item.get('gfx')
        directory = 'trinkets' if item.tag == 'trinket' else 'collectibles'
        entry = {
            'id': int(item.attrib['id']), 'kind': item.tag,
            'name': item.get('name', ''), 'description': item.get('description', ''),
            'sprite': f'gfx/items/{directory}/{sprite.lower()}' if sprite else None,
        }
        if item.tag == 'active':
            # ItemConfig+0x44 MaxCharges, read by HUD::Render's charge bar.
            entry['maxCharges'] = int(item.get('maxcharges', '0'))
        items.append(entry)
    # Card config +8 description, +10 name, +18 HUD CardFronts animation
    # (empty for runes), read by HUD::Render's pocket slot.
    cards = [{'id': int(card.attrib['id']), 'name': card.get('name', ''),
              'description': card.get('description', ''), 'hud': card.get('hud', '')}
             for card in ET.parse(pockets_path).getroot() if card.tag in ('card', 'rune')]
    return {
        'schema': 1,
        'sources': {path.name: hashlib.sha256(path.read_bytes()).hexdigest()
                    for path in [entities_path, items_path, pockets_path]},
        'pickups': pickups, 'items': items, 'cards': cards,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--resources', type=Path, default=ROOT / 'references/isaac/rebirth/resources')
    parser.add_argument('--output', type=Path, default=ROOT / 'web/pickup-data.json')
    args = parser.parse_args()
    catalog = extract(args.resources)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(catalog, separators=(',', ':')) + '\n')
    print(json.dumps({'pickups': len(catalog['pickups']), 'items': len(catalog['items']),
                      'output': str(args.output)}))


if __name__ == '__main__':
    main()
