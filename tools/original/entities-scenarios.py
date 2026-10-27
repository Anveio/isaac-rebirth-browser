#!/usr/bin/env python3
"""Public-input tapes for every entity spec in tools/e2e/entities/, shared.

Room specs keep their own room selection (entity-scenarios.py admitting_slots:
their type, shape and the all-NPCs-admitted filter) but share one OS-random
seed stream and one floor generation per seed. A (seed, door slot, route kind)
that admits several specs still needing rooms is one case graded for each of
them; random counts are met only by OS-random rooms. Identical contrived
routes are deduplicated. Boss specs keep boss-scenarios.py and shop specs
shop-scenarios.py, each with its own discovery and attempt budget. A spec
whose discovery fails is reported in `failures` and every other spec's cases
still run. Each case names its
graded specs in `entity.specs`.
"""
import argparse
import importlib.util
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]


def load(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


entity = load('entities_entity', 'entity-scenarios.py')
codec = entity.codec


def specs():
    script = ("import { entitySpecs } from './tools/e2e/entities.mjs';"
              "console.log(JSON.stringify((await entitySpecs()).map(({ name, type, variant, shape, roomType, gridType, count, contrived, fight, boss, shop, hard }) =>"
              " ({ name, type, variant, shape, roomType, gridType, count, contrived, fight, boss: Boolean(boss), shop: Boolean(shop), hard: Boolean(hard) }))));")
    return json.loads(subprocess.run(['node', '--input-type=module', '-e', script], cwd=ROOT, capture_output=True, text=True, check=True).stdout)


def identity(spec):
    return (spec.get('type'), spec.get('variant', 0) if spec.get('type') is not None else None)


def shape(spec):
    return tuple(spec.get('shape') or (13, 7))


def fight(specs):
    return max(spec.get('fight') or 0 for spec in specs)


def tagged(case, names, identifier):
    case['scenario']['id'] = identifier
    case['entity']['specs'] = names
    return case


def room_cases(room_specs, attempts):
    cases, failures = [], []
    # Contrived routes: one case per distinct (code, slot, shape), graded for
    # every spec that names it.
    routes = {}
    for spec in room_specs:
        for item in spec['contrived']:
            code, slot = item.split(':')
            routes.setdefault((code, int(slot), shape(spec), spec['hard']), []).append(spec)
    for (code, slot, room_shape, hard), named in routes.items():
        floor = entity.floor(codec.decode_seed(code), hard)
        for spec in named:
            if slot not in dict(entity.admitting_slots(floor, identity(spec), room_shape, spec.get('roomType'), spec.get('gridType'))):
                failures.append({'spec': spec['name'], 'error': f'Contrived {code} slot {slot} does not admit {identity(spec)} in a {room_shape[0]}x{room_shape[1]} room'})
        admitted = [spec for spec in named if slot in dict(entity.admitting_slots(floor, identity(spec), room_shape, spec.get('roomType'), spec.get('gridType')))]
        if admitted:
            names = [spec['name'] for spec in admitted]
            cases.append(tagged(entity.case('', code, slot, identity(admitted[0]), {'entropy': 'contrived spec seed'}, False, room_shape,
                                            fight(admitted), hard),
                                names, f"{'+'.join(names)}-contrived-{len(cases)}"))
    # OS-random rooms from one seed stream.
    needed = {spec['name']: spec['count'] for spec in room_specs if spec['count']}
    by_name = {spec['name']: spec for spec in room_specs}
    tried, attempt = set(), 0
    while any(needed.values()) and attempt < attempts:
        attempt += 1
        vector = codec.random_vector()
        if vector['seedValue'] in tried:
            continue
        tried.add(vector['seedValue'])
        floor, code = entity.floor(vector['seedValue']), codec.encode_seed(vector['seedValue'])
        # Each spec still needing a room takes at most one per seed; rooms that
        # admit the most of them are chosen first.
        groups = {}
        for name, remaining in needed.items():
            if not remaining:
                continue
            spec = by_name[name]
            for slot, descriptor in entity.admitting_slots(floor, identity(spec), shape(spec), spec.get('roomType'), spec.get('gridType')):
                groups.setdefault((slot, shape(spec)), (descriptor, []))[1].append(spec)
        chosen, taken = {}, set()
        for key, (descriptor, admitted) in sorted(groups.items(), key=lambda item: (-len(item[1][1]), item[0][0])):
            admitted = [spec for spec in admitted if spec['name'] not in taken]
            if admitted:
                chosen[key] = (descriptor, admitted)
                taken.update(spec['name'] for spec in admitted)
        groups = chosen
        for (slot, room_shape), (descriptor, admitted) in groups.items():
            names = [spec['name'] for spec in admitted]
            for name in names:
                needed[name] -= 1
            provenance = {'entropy': 'OS random', 'discoveryAttempts': attempt,
                          'room': descriptor['template']['name'], 'catalogIndex': descriptor['catalogIndex']}
            cases.append(tagged(entity.case('', code, slot, identity(admitted[0]), provenance, True, room_shape, fight(admitted)),
                                names, f"{'+'.join(names)}-random-{len(cases)}"))
    for name, remaining in needed.items():
        if remaining:
            failures.append({'spec': name, 'error': f'Only {by_name[name]["count"] - remaining} of {by_name[name]["count"]} rooms admitting '
                                                    f'{identity(by_name[name])} in {attempt} OS-random seeds'})
    return cases, failures, {'seedsTried': len(tried)}


def routed_cases(spec, helper, attempts):
    args = ['python3', str(Path(__file__).with_name(helper)), '--type', str(spec['type']), '--variant', str(spec.get('variant', 0)),
            '--count', str(spec['count']), '--attempts', str(attempts), *[arg for code in spec['contrived'] for arg in ('--contrived', code)]]
    result = subprocess.run(args, cwd=ROOT, capture_output=True, text=True)
    if result.returncode:
        return [], [{'spec': spec['name'], 'error': (result.stderr.strip().splitlines() or [f'{helper} discovery failed'])[-1]}], None
    document = json.loads(result.stdout)
    cases = [tagged(case, [spec['name']], f"{spec['name']}-{case['scenario']['id']}") for case in document['cases']]
    return cases, [], document.get('discovery')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--only', action='append', default=[], help='spec names (repeatable or comma-separated); default every spec')
    parser.add_argument('--attempts', type=int, default=4000, help='shared room-spec seed budget')
    parser.add_argument('--boss-attempts', type=int, default=6000, help='seed budget per boss spec')
    parser.add_argument('--shop-attempts', type=int, default=40000, help='seed budget per shop spec')
    args = parser.parse_args()
    selected = specs()
    only = {name for item in args.only for name in item.split(',') if name}
    unknown = only - {spec['name'] for spec in selected}
    if unknown:
        raise SystemExit(f'Unknown entity spec {", ".join(sorted(unknown))}')
    if only:
        selected = [spec for spec in selected if spec['name'] in only]
    cases, failures, discovery = room_cases([spec for spec in selected if not spec['boss'] and not spec['shop']], args.attempts)
    discovery = {'rooms': discovery}
    for spec in selected:
        if spec['boss'] or spec['shop']:
            found, failed, summary = (routed_cases(spec, 'boss-scenarios.py', args.boss_attempts) if spec['boss']
                                      else routed_cases(spec, 'shop-scenarios.py', args.shop_attempts))
            cases += found
            failures += failed
            discovery[spec['name']] = summary
    print(json.dumps({'schema': 1, 'specs': [spec['name'] for spec in selected], 'discovery': discovery, 'failures': failures, 'cases': cases}))


if __name__ == '__main__':
    main()
