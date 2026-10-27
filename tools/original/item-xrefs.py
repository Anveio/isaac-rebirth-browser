#!/usr/bin/env python3
"""Map where the pinned original reads each collectible and trinket.

Disassembles the local pinned isaac.x64 (never committed) with objdump and
records, per function, every item query with a literal id:

  HasCollectible   5090C0(player, id)            count at Player+0xE1C+4*id > 0
  CountRead        [reg+0xE1C+4*id]              inline permanent count read
  TempCount        5E3ED0(effects, id)           TemporaryEffects collectible count
  TempHas          5E3D70(effects, id)           TemporaryEffects collectible present
  HasTrinket       508FE0(player, id)
  CheckFamiliar    508D10(player, variant, ...)  Familiar reconciliation (cache 0x200)

EvaluateItems (511F10) is split by cache flag, following its dispatch chain,
so each stat block lists the items it reads in source order. The output is
derived addresses and ids only, no original bytes. Inline count reads outside
EvaluateItems assume a Player base and are labelled as such.

    python3 tools/original/item-xrefs.py [--binary PATH] [--out references/isaac/foundation/items/item-xrefs.json]
"""
import argparse
import bisect
import hashlib
import json
import re
import subprocess
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
# The pin lives in tools/e2e/lib/original.mjs only.
PINNED_SHA = re.search(r"ORIGINAL_SHA = '([0-9a-f]{64})'", (ROOT / 'tools/e2e/lib/original.mjs').read_text()).group(1)
COUNT_BASE, MAX_COLLECTIBLE, MAX_TRINKET = 0xE1C, 346, 61
HAS, HAS_TRINKET, TEMP_COUNT, TEMP_HAS, CHECK_FAMILIAR = 0x5090C0, 0x508FE0, 0x5E3ED0, 0x5E3D70, 0x508D10
EVALUATE = 0x511F10
# 511F10 dispatch: flag -> first block instruction; the chain's own test
# instructions end every block walk.
CACHE_BLOCKS = {
    'weapon 0x100': 0x513AD0, 'libra 0x1b': 0x5122C0, 'firedelay 0x2': 0x5122D3,
    'shotspeed 0x4': 0x511F68, 'damage 0x1': 0x5134B0, 'range 0x8': 0x5127C8,
    'speed 0x10': 0x512C48, 'tearflag 0x20': 0x5130CA, 'tearcolor 0x40': 0x513355,
    'flying 0x80': 0x512166, 'familiars 0x200': 0x513E80, 'luck 0x400': 0x513C80,
    'clamps and final libra': 0x5121EF,
}
DISPATCH = {0x511F4A, 0x511F54, 0x511F5E, 0x51212E, 0x512138, 0x512142, 0x51214C,
            0x512156, 0x512160, 0x5121D0, 0x5121E3, 0x5121EF,
            # Fall-through tests that skip to the next flag after a block.
            0x5122C3, 0x512C3E, 0x5130C0, 0x51334B}

INSN = re.compile(r'^\s+([0-9a-f]+):\t(\S+)\s*(.*)$')


def disassemble(binary):
    text = subprocess.run(['objdump', '-d', '--no-show-raw-insn', '-M', 'intel', str(binary)],
                          check=True, capture_output=True, text=True).stdout
    out = []
    for line in text.split('\n'):
        match = INSN.match(line)
        if match:
            operands = re.sub(r'\s*<[^>]*>', '', match.group(3)).split('#')[0].strip()
            out.append((int(match.group(1), 16), match.group(2), operands))
    return out


def immediate(insns, index, register, window=16):
    """Literal loaded into `register` within the few instructions before a call."""
    for back in range(index - 1, max(index - window, 0) - 1, -1):
        mnemonic, operands = insns[back][1], insns[back][2]
        if mnemonic.startswith('call') or mnemonic.startswith('j'):
            return None
        if mnemonic == 'mov' and operands.startswith(register + ','):
            value = operands.split(',', 1)[1]
            if value.startswith('0x'):
                return int(value, 16)
            return None
    return None


def reads_at(insns, index):
    address, mnemonic, operands = insns[index]
    found = []
    if mnemonic.startswith('call'):
        target = operands.strip()
        try:
            target = int(target, 16)
        except ValueError:
            return found
        for function, kind, register in ((HAS, 'HasCollectible', 'esi'), (HAS_TRINKET, 'HasTrinket', 'esi'),
                                         (TEMP_COUNT, 'TempCount', 'esi'), (TEMP_HAS, 'TempHas', 'esi'), (CHECK_FAMILIAR, 'CheckFamiliar', 'esi')):
            if target == function:
                value = immediate(insns, index, register)
                if value is not None:
                    found.append((kind, value, address))
        return found
    for match in re.finditer(r'\[r[a-z0-9]+\+0x([0-9a-f]+)\]', operands):
        offset = int(match.group(1), 16)
        if COUNT_BASE + 4 <= offset <= COUNT_BASE + 4 * MAX_COLLECTIBLE and (offset - COUNT_BASE) % 4 == 0:
            found.append(('CountRead', (offset - COUNT_BASE) // 4, address))
    return found


def functions(insns):
    starts = {EVALUATE}
    for _, mnemonic, operands in insns:
        if mnemonic.startswith('call') and re.fullmatch(r'[0-9a-f]+', operands):
            starts.add(int(operands, 16))
    return sorted(starts)


def walk(insns, addresses, entry, end, stops):
    """Addresses reachable from entry inside [start, end) without crossing stops."""
    seen, stack = set(), [entry]
    while stack:
        address = stack.pop()
        if address in seen or address in stops and address != entry:
            continue
        index = bisect.bisect_left(addresses, address)
        while index < len(insns) and insns[index][0] < end:
            here, mnemonic, operands = insns[index]
            if here in seen or (here in stops and here != entry):
                break
            seen.add(here)
            if mnemonic.startswith('ret'):
                break
            if mnemonic.startswith('j'):
                try:
                    stack.append(int(operands, 16))
                except ValueError:
                    pass
                if mnemonic == 'jmp':
                    break
            index += 1
    return seen


def summarize(entries):
    collectibles, trinkets, familiars = [], [], []
    for kind, value, _ in entries:
        if kind == 'HasTrinket':
            if 0 < value <= MAX_TRINKET and value not in trinkets:
                trinkets.append(value)
        elif kind == 'CheckFamiliar':
            if value not in familiars:
                familiars.append(value)
        elif 0 < value <= MAX_COLLECTIBLE and value not in collectibles:
            collectibles.append(value)
    return dict(collectibles=collectibles, trinkets=trinkets, familiarVariants=familiars)


def main():
    parser = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    parser.add_argument('--binary', default=str(ROOT / '.cache/original-runtime/offline-depot/isaac.x64'))
    parser.add_argument('--out', default=str(ROOT / 'references/isaac/foundation/items/item-xrefs.json'))
    args = parser.parse_args()
    binary = Path(args.binary)
    if hashlib.sha256(binary.read_bytes()).hexdigest() != PINNED_SHA:
        raise SystemExit('binary is not the pinned base Rebirth isaac.x64')
    insns = disassemble(binary)
    addresses = [insn[0] for insn in insns]
    starts = functions(insns)
    evaluate_end = starts[bisect.bisect_right(starts, EVALUATE)]

    per_function = defaultdict(list)
    for index, (address, _, _) in enumerate(insns):
        for entry in reads_at(insns, index):
            function = starts[bisect.bisect_right(starts, address) - 1]
            per_function[function].append(entry)

    blocks = {}
    for name, entry in CACHE_BLOCKS.items():
        reach = walk(insns, addresses, entry, evaluate_end, DISPATCH)
        blocks[name] = dict(entry=f'{entry:06X}',
                            **summarize([read for read in per_function[EVALUATE] if read[2] in reach]))

    by_item = defaultdict(lambda: defaultdict(set))
    report = []
    for function in sorted(per_function):
        entries = per_function[function]
        has_query = any(kind != 'CountRead' for kind, _, _ in entries)
        # Inline count offsets outside the player-owning functions are too
        # ambiguous to report; keep them only beside a real item query.
        if function != EVALUATE and not has_query:
            continue
        summary = summarize(entries)
        report.append(dict(function=f'{function:06X}', **summary))
        for item in summary['collectibles']:
            by_item['collectibles'][item].add(f'{function:06X}')
        for item in summary['trinkets']:
            by_item['trinkets'][item].add(f'{function:06X}')

    # items.xml `cache` names the flags an Add requests; compare them with the
    # blocks that actually read each item. Differences are leads, not errors.
    flag_names = {'damage': 'damage 0x1', 'firedelay': 'firedelay 0x2', 'shotspeed': 'shotspeed 0x4',
                  'range': 'range 0x8', 'speed': 'speed 0x10', 'tearflag': 'tearflag 0x20',
                  'tearcolor': 'tearcolor 0x40', 'flying': 'flying 0x80', 'weapon': 'weapon 0x100',
                  'familiars': 'familiars 0x200', 'luck': 'luck 0x400'}
    xml = (ROOT / 'references/isaac/rebirth/resources/items.xml').read_text()
    declared_unread, read_undeclared = [], []
    for match in re.finditer(r'<(passive|active|familiar) ([^>]*)/>', xml):
        values = dict(re.findall(r'(\w+)="([^"]*)"', match.group(2)))
        item, flags = int(values['id']), set(values.get('cache', '').split())
        if not flags or 'all' in flags:
            continue
        for flag in sorted(flags):
            if flag in flag_names and item not in blocks[flag_names[flag]]['collectibles']:
                declared_unread.append(dict(item=item, flag=flag))
        for flag, block in flag_names.items():
            if flag not in flags and item in blocks[block]['collectibles']:
                read_undeclared.append(dict(item=item, flag=flag))

    result = dict(
        schema=1, binarySha256=PINNED_SHA, tool='tools/original/item-xrefs.py',
        note='Derived call sites only. CountRead outside 511F10 assumes a Player base.',
        evaluateItems=dict(function=f'{EVALUATE:06X}', end=f'{evaluate_end:06X}', blocks=blocks),
        cacheCrossCheck=dict(declaredButNotReadByLiteral=declared_unread, readButNotDeclared=read_undeclared),
        functions=report,
        collectibleReaders={str(item): sorted(functions) for item, functions in sorted(by_item['collectibles'].items())},
        trinketReaders={str(item): sorted(functions) for item, functions in sorted(by_item['trinkets'].items())},
    )
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(result, indent=1) + '\n')
    print(json.dumps(dict(functions=len(report), collectibles=len(by_item['collectibles']),
                          trinkets=len(by_item['trinkets']), out=str(out))))


if __name__ == '__main__':
    main()
