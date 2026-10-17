"""Independent source-only Shop35 law over an explicitly frozen plan document."""
import copy
import math
import struct
from types import SimpleNamespace


def next35(value):
    value ^= value >> 5
    value ^= (value << 9) & 0xffffffff
    value ^= value >> 7
    return value & 0xffffffff


def initialized(seed):
    table = list(range(6))
    draws = []

    def bounded(size):
        nonlocal seed
        seed = next35(seed)
        draws.append({'state': seed, 'bound': size, 'value': seed % size})
        return seed % size

    for index in range(5, 0, -1):
        other = bounded(index + 1)
        table[index], table[other] = table[other], table[index]
    table.reverse()
    table[bounded(3)] = 6
    sale = bounded(6) if bounded(3) else -1
    return {'table': table, 'saleSlot': sale, 'followingSeed': seed, 'draws': draws}


def create_law(plan_document, *, common_sku=False):
    if type(common_sku) is not bool:
        raise ValueError('Common SKU law is an explicit boolean opt-in')
    if not isinstance(plan_document, dict) or type(plan_document.get('schema')) is not int or plan_document.get('schema') != 1:
        raise ValueError('Unknown Shop source-plan schema')
    plans = plan_document.get('plans')
    if not isinstance(plans, list) or not plans:
        raise ValueError('Requires nonempty explicit Shop source plans')
    def integer(value, low, high):
        return type(value) is int and low <= value <= high
    by_key = {}
    for original in plans:
        plan = copy.deepcopy(original)
        if not isinstance(plan, dict) or not integer(plan.get('seedValue'), 1, 0xffffffff) or not integer(plan.get('stageSeed'), 1, 0xffffffff):
            raise ValueError('Source plan requires actual nonzero seed/stage words')
        key = (plan['seedValue'], plan['stageSeed'])
        if key in by_key:
            raise ValueError('Duplicate declared public seed/stage plan')
        allowed = ('common-sku',) if common_sku else ('historical-pwtj-refusal', 'paid-regular')
        if plan.get('guardKind') not in allowed:
            raise ValueError('Unknown explicit Shop reader scope')
        shops = plan.get('shops')
        if not isinstance(shops, list) or not shops or len({shop.get('index') for shop in shops}) != len(shops):
            raise ValueError('Requires unique declared Shop descriptors')
        for shop in shops:
            for field, low, high in [('index', 0, 127), ('gridIndex', 0, 168), ('assetStage', 0, 255), ('templateVariant', 0, 0xffffffff), ('decorationSeed', 1, 0xffffffff), ('spawnSeed', 1, 0xffffffff), ('initialFollowingSeed', 1, 0xffffffff), ('expectedCursor', 0, 5)]:
                if not integer(shop.get(field), low, high):
                    raise ValueError('Invalid declared Shop field: ' + field)
            selectors = shop.get('selectors')
            if not isinstance(selectors, list) or not selectors or len(selectors) > 512 or shop['expectedCursor'] != len(selectors) % 6:
                raise ValueError('Declared fresh150 cursor/count mismatch')
            seeds = set()
            for position, row in enumerate(selectors):
                if not isinstance(row, dict) or row.get('category') != 'entity' or not integer(row.get('type'), 5, 5) or not integer(row.get('variant'), 150, 150) or not integer(row.get('subtype'), 0, 0):
                    raise ValueError('Expected actual authored fresh150 selector')
                if not integer(row.get('spawnSeed'), 1, 0xffffffff) or row['spawnSeed'] in seeds or not integer(row.get('shopSlotAc4'), 0, 5) or row.get('shopSlotAc4') != position % 6:
                    raise ValueError('Invalid selector spawn identity or slot')
                seeds.add(row['spawnSeed'])
                point = row.get('position')
                if not isinstance(point, dict) or any(type(point.get(word)) not in (int, float) or not math.isfinite(point[word]) for word in ('x', 'y')):
                    raise ValueError('Invalid authored selector position')
        by_key[key] = plan

    def declared_shop(context, index):
        log = context.get('publicLogState', {})
        plan = by_key.get((log.get('seedValue'), log.get('stageSeed')))
        if plan is None or log.get('stage') != 1 or log.get('stageSeed') != plan['stageSeed']:
            raise ValueError('Actual public log is outside declared stage1 Shop source plans')
        shops = [shop for shop in plan['shops'] if shop['index'] == index]
        if len(shops) != 1:
            raise ValueError('Actual Shop descriptor is not uniquely declared')
        return shops[0]


    def validate(level, at, index, template, initial_following, following, context, is_current):
        if template[1] != 2:
            if following != initial_following:
                raise ValueError('Non-Shop descriptor violates unchanged initial RNG12 following seed')
            return None
        plan = declared_shop(context, index)
        if tuple(template) != (plan['assetStage'], 2, plan['templateVariant']):
            raise ValueError('Actual Shop template identity differs from declared source floor')
        decoration, spawn = struct.unpack_from('<II', level, at + 0x48)
        grid_index = struct.unpack_from('<i', level, at)[0]
        if (decoration, spawn, initial_following, grid_index) != (
                plan['decorationSeed'], plan['spawnSeed'], plan['initialFollowingSeed'], plan['gridIndex']):
            raise ValueError('Actual Shop initial seed/topology differs from declared source floor')
        if context.get('shopObservedAchievements151to154') != [0, 0, 0, 0]:
            raise ValueError('Requires actual fresh-profile ShopLevel1 achievements')
        cursor, sale = struct.unpack_from('<ii', level, at + 0xe8)
        if cursor == -1:
            if following != initial_following or sale != -1 or (is_current and plan['selectors']):
                raise ValueError('Uninitialized Shop must retain initial following/sale; entered150 must select')
            return {'initialized': False, 'cursor': cursor, 'saleSlot': sale,
                    'table': None, 'initialFollowingSeed': initial_following,
                    'currentFollowingSeed': following,
                    'tableScope': 'Uninitialized storage is not interpreted'}
        expected = initialized(initial_following)
        table = list(struct.unpack_from('<6i', level, at + 0xd0))
        if (table, sale, following) != (expected['table'], expected['saleSlot'], expected['followingSeed']):
            raise ValueError('Shop table/sale/current following seed violates independent source35 replay')
        if not plan['selectors'] or cursor != plan['expectedCursor']:
            raise ValueError('Shop cursor differs from declared fresh150 selections; saved reentry must retain it')
        return {'initialized': True, 'cursor': cursor, 'saleSlot': sale, 'table': table,
                'initialFollowingSeed': initial_following, 'currentFollowingSeed': following,
                'expected': expected, 'authoredSelections': len(plan['selectors']),
                'scope': 'Declared fresh normal source150 selections; concrete saved reentry has no initializer/cursor draws'}

    return SimpleNamespace(declared_shop=declared_shop, validate=validate)
