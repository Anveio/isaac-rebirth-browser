#!/usr/bin/env python3
"""Public-input tapes that collect natural secret-room keys and enter a Basement shop.

Basement I shop doors are key-locked in the original (Door variant1/state1),
and free keys in ordinary rooms are walled in. Each route therefore bombs the
secret room below the start with the starting bomb (the secret-door prefix),
walks over its authored keys, returns and pushes the shop door open. Discovery
draws OS-random run seeds and keeps floors whose secret room below the start
holds keys and no hostiles, and whose shop borders a room reachable through
quiet ordinary rooms. The production build records the route closed-loop from
`secret-entry` (tools/e2e/boss-route-record.mjs --from --collect); both games
then replay the same public keys, and every recorded route is validated
against a fresh original process that must stand in the shop at `shop-arrival`.
"""
import argparse
from collections import Counter
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('shop_boss_routes', Path(__file__).with_name('boss-scenarios.py'))
routes = importlib.util.module_from_spec(spec)
spec.loader.exec_module(routes)
codec = routes.codec
SHOP = 2


spec = importlib.util.spec_from_file_location('shop_secret_routes', Path(__file__).with_name('secret-door-scenarios.py'))
secret = importlib.util.module_from_spec(spec)
spec.loader.exec_module(secret)


def plan(generated):
    """Room chain secret → start → … → shop, or a rejection reason."""
    descriptors, spawns = generated['descriptors'], generated['spawns']
    start = generated['startingDescriptor']
    below = descriptors[start]['neighbors'][3]
    if below is None or descriptors[below]['template']['type'] != 7:
        return None, 'no-secret-below'
    selectors = (spawns[below] or {}).get('selectors', [])
    if not any(row['category'] == 'entity' and row['type'] == 5 and row['variant'] == 30 for row in selectors):
        return None, 'no-secret-key'
    if any(row['category'] == 'entity' and row['type'] not in (5, 17) for row in selectors):
        return None, 'secret-hostile'
    shops = [index for index, descriptor in enumerate(descriptors) if descriptor['template']['type'] == SHOP]
    if not shops:
        return None, 'no-shop'
    shop = shops[0]
    previous, queue = {start: None}, [start]
    while queue:
        current = queue.pop(0)
        for neighbor in descriptors[current]['neighbors']:
            if neighbor is None or neighbor in previous:
                continue
            template = descriptors[neighbor]['template']
            if (template['type'], template['width'], template['height']) != (1, 13, 7) \
                    or not routes.quiet(spawns[neighbor]) or generated['unsupportedNpcs'][neighbor]:
                continue
            previous[neighbor] = current
            queue.append(neighbor)
    beside = [room for room in previous if shop in descriptors[room]['neighbors']]
    if not beside:
        return None, 'no-quiet-shop-door'
    def chain(room):
        rooms = [room]
        while previous[rooms[0]] is not None:
            rooms.insert(0, previous[rooms[0]])
        return rooms
    door = min(beside, key=lambda room: len(chain(room)))
    entry = descriptors[door]['neighbors'].index(shop) % 4
    return {'rooms': [below] + chain(door) + [shop], 'keyRoom': below, 'entrySlot': entry, 'shop': shop}, None


def record(base, route):
    with tempfile.NamedTemporaryFile('w', suffix='.json') as handle:
        json.dump(base, handle); handle.flush()
        result = subprocess.run(['node', str(routes.RECORDER), '--case', handle.name, '--rooms', ','.join(map(str, route['rooms'])),
                                 '--collect', str(route['keyRoom']), '--arrival', 'shop-arrival', '--from', 'secret-entry'],
                                cwd=ROOT, capture_output=True, text=True, timeout=600)
    lines = result.stdout.strip().splitlines()
    if not lines:
        raise RuntimeError(f'Route recorder failed: {result.stderr.strip()[-2000:]}')
    return json.loads(lines[-1])


def case(identifier, code, route, provenance, randomized):
    result = routes.paired.build('paired-bomb-moving-even', code)
    # The secret-door prefix: starting bomb below the start, wait, walk in.
    prefix = secret.secret_case(code, 'shop route secret prefix')['actions']
    prefix = prefix[:next(index for index, action in enumerate(prefix) if action.get('label') == 'secret-entry') + 1]
    for name in ('tape', 'pairedTape'):
        end = next(index for index, action in enumerate(result[name]) if action.get('label') == 'gameplay-start')
        result[name] = result[name][:end + 1] + prefix
    recorded = record(result, route)
    if recorded['status'] != 'pass':
        return None, f"port-route: {recorded['reason']}", None
    tail = list(recorded['actions'])
    routes.dense(tail, 120, 'shop-idle', 2)
    for name in ('tape', 'pairedTape'):
        result[name] = result[name] + tail
    routes.runner.validate_tape(result['tape'])
    result['scenario'] = {'id': identifier, 'description': f"Bomb into secret room {route['keyRoom']}, collect its keys and enter shop {route['shop']}",
                          'systems': ['room-connectivity', 'enemies-bosses', 'pickup-physics'], 'randomized': randomized}
    result['entity'] = {'type': 17, 'variant': 0, 'shape': [13, 7], 'slot': route['entrySlot'], 'shop': True,
                        'rooms': route['rooms'], 'keyRoom': route['keyRoom'], 'routeFrames': recorded['frames'], 'selection': provenance}
    return result, None, recorded['checkpoints']


def original_route(result, directory):
    directory.mkdir(parents=True, exist_ok=True)
    tape = directory / 'tape.json'
    end = next(index for index, action in enumerate(result['tape']) if action.get('label') == 'shop-arrival')
    tape.write_text(json.dumps(result['tape'][:end + 1]))
    subprocess.run(['bash', 'tools/original/with-display.sh', 'python3', 'tools/original/foundation-clock-run.py',
                    '--seed', result['seed'], '--tape', str(tape), '--state-observer', 'tools/original/foundation-combat-observe.py',
                    '--frame-timeout', '120', '--output', str(directory / 'original')],
                   cwd=ROOT, capture_output=True, text=True, timeout=1800)
    manifest = json.loads((directory / 'original/manifest.json').read_text())
    return {row['context']['label']: {'room': row['observation']['room'], 'player': row['observation']['players'][0]['position']}
            for row in manifest['debugObservations']
            if row['observation'].get('status') == 'pass' and row['observation'].get('players')}


def validate(result, checkpoints, scratch):
    """Reject a route the fresh original does not follow into the shop."""
    original = original_route(result, scratch)
    for label, port in checkpoints.items():
        source = original.get(label)
        if not source:
            return f'original-missing-{label}'
        room = source['room']['descriptorIndex']
        drift = max(abs(a - b) for a, b in zip(source['player'], port['player']))
        if room != port['room'] or drift > 40:
            return f"original-left-route at {label}: room {room} vs {port['room']}, drift {drift:.1f}"
    if original['shop-arrival']['room']['type'] != SHOP:
        return f"original-room-type-{original['shop-arrival']['room']['type']}"
    return None


def build(identifier, code, provenance, randomized, checked, scratch):
    route, reason = plan(routes.floor(codec.decode_seed(code)))
    if route is None:
        return None, reason
    result, reason, checkpoints = case(identifier, code, route, provenance, randomized)
    if result is None or not checked:
        return result, reason
    reason = validate(result, checkpoints, scratch / code.replace(' ', ''))
    return (None, reason) if reason else (result, None)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    # The generic entity spec passes its identity; only the shop Keeper is planned.
    parser.add_argument('--type', type=int, default=17)
    parser.add_argument('--variant', type=int, default=0)
    parser.add_argument('--shape')
    parser.add_argument('--contrived', action='append', default=[], metavar='CODE')
    parser.add_argument('--count', type=int, default=1, help='OS-random cases to discover')
    parser.add_argument('--attempts', type=int, default=40000)
    parser.add_argument('--no-validate', action='store_true', help='skip the fresh-original arrival check (research only)')
    args = parser.parse_args()
    if (args.type, args.variant) != (17, 0):
        parser.error('shop routes admit only the shop Keeper17/0')
    checked = not args.no_validate
    counts, rejected, cases = Counter(), [], []
    (ROOT / '.cache/shop-route').mkdir(parents=True, exist_ok=True)
    scratch = Path(tempfile.mkdtemp(prefix='validate-', dir=ROOT / '.cache/shop-route'))
    for index, code in enumerate(args.contrived):
        result, reason = build(f'contrived-{index}', code, {'entropy': 'contrived spec seed'}, False, checked, scratch)
        if result is None:
            raise ValueError(f'Contrived {code} does not reach a shop: {reason}')
        cases.append(result)
    found, tried = 0, set()
    for attempt in range(1, args.attempts + 1):
        if found == args.count:
            break
        vector = codec.random_vector()
        if vector['seedValue'] in tried:
            continue
        tried.add(vector['seedValue'])
        code = codec.encode_seed(vector['seedValue'])
        result, reason = build(f'random-{found}', code, {'entropy': 'OS random', 'discoveryAttempts': attempt}, True, checked, scratch)
        if result is None:
            counts[reason.split(':')[0].split(' ')[0]] += 1
            if reason not in ('no-secret-below', 'no-secret-key', 'secret-hostile', 'no-shop', 'no-quiet-shop-door'):
                rejected.append({'code': code, 'reason': reason})
            continue
        cases.append(result)
        found += 1
    if found < args.count:
        raise RuntimeError(f'Only {found} of {args.count} shop routes in {args.attempts} OS-random seeds: {dict(counts)} {rejected}')
    print(json.dumps({'schema': 1, 'entity': {'type': 17, 'variant': 0, 'shape': [13, 7], 'shop': True},
                      'discovery': {'seedsTried': len(tried), 'rejectedCounts': dict(counts), 'rejectedAfterPlanning': rejected,
                                    'originalValidated': checked},
                      'cases': cases}))


if __name__ == '__main__':
    main()
