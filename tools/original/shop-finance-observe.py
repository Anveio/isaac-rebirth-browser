"""Bounded Shop reference composition over an explicit pre-capture public plan.

The returned observer supplies observations only. Its plan never enters WASM.
Default paired/item readers and historical capture modules remain unchanged.
"""
import copy
import importlib.util
import os
from pathlib import Path
import signal
import struct
import time
from types import SimpleNamespace

ROOT = Path(__file__).resolve().parents[2]


def load(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


def pickup_slots(pid, context, rooms):
    """Actual manager-member AC4; Init4fff8b/Save5ce5a1/readmit5c5937."""
    if not context.get('gateBlocked'):
        raise PermissionError('Shop slots require the blocked original child')
    parent = int(next(line.split(':', 1)[1] for line in Path(f'/proc/{pid}/status').read_text().splitlines()
                      if line.startswith('PPid:')))
    if parent != os.getpid():
        raise PermissionError('Shop slots require direct parent ownership')
    if rooms.contents.native_identity.native_binary_sha256(pid) != rooms.contents.NATIVE_SHA:
        raise ValueError('Shop slots require the pinned original binary')
    os.kill(pid, signal.SIGSTOP)
    try:
        deadline = time.monotonic() + 2
        while Path(f'/proc/{pid}/status').read_text().split('State:')[1].strip()[0] not in 'Tt':
            if time.monotonic() > deadline:
                raise TimeoutError('Original did not stop for pickup slot observation')
            time.sleep(.001)
        maps = rooms.contents.reader.writable_maps(Path(f'/proc/{pid}/maps').read_text())
        fd = os.open(f'/proc/{pid}/mem', os.O_RDONLY | os.O_CLOEXEC)
        consumed = 0
        try:
            def read(address, count):
                nonlocal consumed
                if count < 0 or consumed + count > 16384:
                    raise ValueError('Shop slot bounded read budget exceeded')
                if not any(m['start'] <= address and address + count <= m['end'] for m in maps):
                    raise PermissionError('Shop slot read left writable source fields')
                raw = os.pread(fd, count, address)
                if len(raw) != count:
                    raise OSError('Short shop slot read')
                consumed += count
                return raw
            u32 = lambda address: struct.unpack('<I', read(address, 4))[0]
            u64 = lambda address: struct.unpack('<Q', read(address, 8))[0]
            game = u64(0x8ecd30)
            room = u64(game + 0x8180)
            manager = room + 0x15d0
            count = u32(manager + 0x74)
            if count > 512:
                raise ValueError('Unexpected actual entity manager size')
            pointers = struct.unpack('<' + 'Q' * count, read(u64(manager + 0x68), count * 8)) if count else []
            slots = []
            for pointer in pointers:
                if pointer and u32(pointer + 0x10) == 5:
                    slots.append({'sequenceId': u32(pointer + 8), 'spawnSeed': u32(pointer + 0x7d0),
                                  'shopSlotAc4': u32(pointer + 0xac4)})
            return slots
        finally:
            os.close(fd)
    finally:
        os.kill(pid, signal.SIGCONT)


def keyed(entries):
    result = {}
    for entry in entries:
        key = (entry['sequenceId'], entry['spawnSeed'])
        if key in result:
            raise ValueError('Duplicate actual actor sequence/spawn identity')
        result[key] = entry
    return result


def create_observer(plan_document, *, common_sku_owners=False):
    if type(common_sku_owners) is not bool:
        raise ValueError('Common SKU owners require explicit boolean opt-in')
    document = copy.deepcopy(plan_document)
    law = load('shop_finance_explicit_law', 'shop-finance-law.py').create_law(
        document, common_sku=common_sku_owners)
    layout_observe = load('shop_finance_bound_layout', 'shop-finance-layout-observe.py').create_observer(law)
    paired = load('shop_finance_existing_paired', 'foundation-paired-observer.py')
    contents = load('shop_finance_existing_contents', 'foundation-pickup-observe.py')
    items = load('shop_finance_existing_items', 'item-observer.py')
    presentation = load('shop_finance_owned_presentation', 'shop-finance-presentation-observe.py')
    rooms = SimpleNamespace(contents=contents)
    reader_kinds = {(p['seedValue'], p['stageSeed']): p['guardKind'] for p in document['plans']}

    def base_observe(pid, context):
        result = paired.observe(pid, context)
        if context.get('publicLogState', {}).get('stage') != 1:
            result['shopFinance'] = {'status': 'skip', 'scope': 'Explicit stage1-only Shop finance reader'}
            return result
        label = context['label']
        routed = dict(context, label='gameplay-start' if label == 'gameplay-start' else 'topology')
        floor = layout_observe(pid, routed)
        actors = contents.observe(pid, context)
        finance = {'schema': 1, 'status': 'pass' if floor.get('status') == actors.get('status') == 'pass' else 'fail',
                   'checkpoint': label, 'memoryWritten': False, 'floor': floor, 'contents': actors}
        result['shopFinance'] = finance
        if finance['status'] == 'pass':
            try:
                slots = {(row['sequenceId'], row['spawnSeed']): row['shopSlotAc4'] for row in pickup_slots(pid, context, rooms)}
                pickups = [row for row in actors['entities'] if row['type'] == 5]
                if len(slots) != len(pickups):
                    raise ValueError('Actual manager-member pickup slot accounting changed')
                for row in pickups:
                    row['shopSlotAc4'] = slots[(row['sequenceId'], row['spawnSeed'])]
                finance['pickupSlotScope'] = 'Actual manager-member AC4; no inferred stock order or sold body'
            except (OSError, ValueError, PermissionError, TimeoutError, struct.error, KeyError) as error:
                finance.update(status='fail', slotError=str(error))
        if finance['status'] != 'pass':
            result['status'] = 'fail'
        result['scope'] = 'Existing paired owners, declared source floor, complete actual pickups and direct Player inventories; observations only.'
        return result

    def observe_finance(pid, context):
        if context.get('publicLogState', {}).get('stage') != 1:
            return base_observe(pid, context)
        acquisition = items.observe(pid, context)
        routed = dict(context, shopObservedAchievements151to154=acquisition.get('achievements', [])[151:155])
        result = base_observe(pid, routed)
        if common_sku_owners:
            fields = ('entitySpawnSeed', 'activeId', 'activeCharge',
                      'activeSubordinateCharge', 'reserveCharge', 'activeStateD80',
                      'activeStateD84', 'ownedCounts', 'inventory', 'health',
                      'pickupAnimationGuard', 'mainSprite', 'pickupSprite',
                      'queuedId', 'queuedCharge', 'queuedTouched', 'queuedTouchedRaw',
                      'controlsEnabled', 'inputCooldown')
            actual = {key: acquisition.get(key) for key in
                      ('status', 'nativeBinarySha256', 'memoryWritten',
                       'executableMappingsRead', 'systemCounter', 'gamePresent',
                       'rawGameA1D8', 'collectionSounds', 'configs')}
            actual['players'] = [{key: player[key] for key in fields}
                                 for player in acquisition.get('players', [])]
            result['commonSkuItemOwners'] = actual
        presentation = {key: acquisition.get(key) for key in
                        ('schema', 'status', 'nativeBinarySha256', 'memoryWritten',
                         'executableMappingsRead', 'bytesRead', 'systemCounter', 'gamePresent', 'error')}
        presentation['achievements151to154'] = routed['shopObservedAchievements151to154']
        presentation['players'] = [{key: player[key] for key in
                                   ('entitySpawnSeed', 'health', 'inventory', 'pickupAnimationGuard',
                                    'mainSprite', 'pickupSprite')}
                                  for player in acquisition.get('players', [])]
        presentation['scope'] = ('Unchanged bounded item-observer actual Player7E0 Sprite reader; '
                                 'observed clip/frame/playing, no inferred paid copy or lifecycle')
        result['shopPlayerPresentation'] = presentation
        finance = result.get('shopFinance', {})
        try:
            if acquisition.get('status') != 'pass' or not acquisition.get('gamePresent'):
                raise ValueError('Existing acquisition/Player Sprite observation did not pass')
            if finance.get('status') != 'pass':
                raise ValueError('Shop-compatible topology and financial observation did not pass')
            if len(presentation['players']) != 1 or len(finance['contents']['players']) != 1:
                raise ValueError('Declared Isaac route requires exactly one actual Player')
            carried = presentation['players'][0]
            player = finance['contents']['players'][0]
            if carried['entitySpawnSeed'] != player['spawnSeed'] or carried['health'][:2] != [player['maxRedHp'], player['redHp']] or carried['inventory'] != {
                    'bombs': player['bombInventory'], 'keys': player['keyInventory'], 'coins': player['coinInventory']}:
                raise ValueError('Existing Player presentation and financial readers disagree')
            actual = keyed(result['gameplay']['entities'])
            pickups = [entry for entry in finance['contents']['entities'] if entry['type'] == 5]
            pickup_keys = set(keyed(pickups))
            if pickup_keys != {key for key, entry in actual.items() if entry['type'] == 5}:
                raise ValueError('Existing combat/pickup manager membership differs')
            for entry in pickups:
                owner = actual[(entry['sequenceId'], entry['spawnSeed'])]
                if [entry['type'], entry['variant'], entry['subtype'], entry['removed']] != [
                        owner['type'], owner['variant'], owner['subtype'], owner['removed']]:
                    raise ValueError('Pickup lifecycle join disagrees with existing actor owner')
                entry['active'] = owner['active']
                entry['dead31f'] = owner['removed']
            finance['lifecycleScope'] = ('Existing combat owner active31E/dead31F, joined by complete '
                                         'actual sequence/spawn identity; membership erasure remains distinct')
            if finance['contents']['room']['type'] == 2:
                index = finance['floor']['currentDescriptorIndex']
                descriptor = finance['floor']['descriptors'][index]
                plan = law.declared_shop(routed, index)
                selectors = {row['spawnSeed']: row for row in plan['selectors']}
                if len(selectors) != len(plan['selectors']):
                    raise ValueError('Duplicate declared stock spawn identity')
                for entry in pickups:
                    selected = selectors.get(entry['spawnSeed'])
                    if selected is None:
                        # Other public producers (e.g. destroyed fire) remain real
                        # observations; they cannot be invented priced stock.
                        if entry['pickupPrice'] != 0:
                            raise ValueError('Undeclared priced stock in actual Shop manager')
                        continue
                    if entry['shopSlotAc4'] != selected['shopSlotAc4']:
                        raise ValueError('Actual declared stock AC4 differs from source cursor')
                finance['declaredStock'] = plan['selectors']
                finance['stockScope'] = ('Retain all actual members and lifecycle. Missing sold members '
                                         'are permitted observations, never accepted purchases. Route '
                                         'guards must independently require complete paid stock before contact.')
        except (ValueError, KeyError, TypeError) as error:
            result['status'] = 'fail'
            finance.update(status='fail', sourceLawError=str(error))
        result['scope'] += (' Declared source Shop descriptor mutation independently checked; '
                            'existing Player copied Sprite and actual pickup active/dead readers composed. '
                            'No paid/refusal acceptance is inferred from capture success.')
        return result

    def observe(pid, context):
        result = observe_finance(pid, context)
        if context.get('publicLogState', {}).get('stage') != 1:
            result['shopPresentation'] = {'status': 'skip', 'scope': 'Explicit gameplay-only presentation reader'}
            return result
        key = (context.get('publicLogState', {}).get('seedValue'), context.get('publicLogState', {}).get('stageSeed'))
        if reader_kinds.get(key) not in ('paid-regular', 'common-sku'):
            return result
        extra = presentation.observe(pid, context)
        result['shopPresentation'] = extra
        try:
            if extra.get('status') != 'pass' or result.get('status') != 'pass':
                raise ValueError('Existing financial or opted-in presentation observation failed')
            counter = extra.get('systemCounter')
            if type(counter) is not int or counter != result['shopPlayerPresentation'].get('systemCounter'):
                raise ValueError('Opted-in presentation and existing item reader observed different turns')
            finance = result['shopFinance']['contents']
            expected_players = keyed(finance['players'])
            actual_players = keyed(extra['players'])
            if set(expected_players) != set(actual_players):
                raise ValueError('Copied Sprite Player identity coverage differs from existing reader')
            for key, player in actual_players.items():
                old = next(row for row in result['shopPlayerPresentation']['players'] if row['entitySpawnSeed'] == key[1])['pickupSprite']
                current = player['pickupSprite']
                if any(old[field] != current[field] for field in ('animation', 'animationFrame', 'animationPlaying', 'overlay', 'overlayFrame')):
                    raise ValueError('Copied Sprite active fields disagree with existing reader')
            expected = keyed([row for row in finance['entities'] if row['type'] == 5]) if extra['roomType'] == 2 else {}
            actual = keyed(extra['pickups'])
            if set(expected) != set(actual) or extra['roomType'] != finance['room']['type']:
                raise ValueError('Opted-in Shop pickup membership differs from existing reader')
            for key, pickup in actual.items():
                old, current = expected[key], pickup['bodySprite']
                if old['pickupPrice'] != pickup['priceAc0'] or any(old[field] != current[field] for field in ('animation', 'animationFrame', 'animationPlaying')):
                    raise ValueError('Shop actual price/body active fields disagree with existing reader')
            extra['joinScope'] = 'Complete actual sequence/spawn identity; clip-only fields agree with unchanged readers, missing manager members remain missing'
        except (ValueError, KeyError, TypeError, StopIteration) as error:
            result['status'] = 'fail'
            extra.update(status='fail', joinError=str(error))
        return result

    return observe
