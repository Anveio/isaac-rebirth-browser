#!/usr/bin/env python3
"""Public-input tapes that exit a run mid-room and Continue it.

Game.Save (54F2C8..54F3C3) writes the ItemPool (581ED0), Room.Save (5CE2D0)
of the current room, the whole Level (58CD90: all descriptors with their
seeds, visited/cleared flags, persistent spawn sets and saved entity records,
the current/previous room 8188/818C and the entry door 8198) and the Player.
Room.Save keeps only the persistent families of 5C57C0 (bombs, pickups,
slots, shopkeepers, fireplaces and listed effects); NPCs are not saved.

Continue loads that Level with every reached descriptor visited, so the
current room takes Room.Init's revisit branch: saved records come back and
5CE6B0 respawns the layout's other entities unless the room is cleared. An
uncleared fight restarts with fresh layout NPCs; the player stands at the
entry door 8198, 40 units inside. Nothing re-rolls: the item pools, their
RNGs and every descriptor seed are restored, so a pedestal holds the same
collectible whether the run was quit before or after entering its room.

Contrived cases:
  * KRAYFRWA: quit after the left room's flies moved; they return to the
    layout cells at full health.
  * 3YSTBQPG (enter-then-quit): quit in the free treasure room, Continue,
    take its collectible, walk back to the start room and quit/Continue
    again (the start room's right door holds the player).
  * 3YSTBQPG (quit-then-enter): quit in the start room before ever entering
    the treasure room; Continue, then enter it. The pedestal matches the
    other case's.
OS-random cases enter a random side neighbour (never a secret room), idle a
random time and quit, Continue and walk back toward the door.
"""
import argparse
import importlib.util
import json
from pathlib import Path
import random
import subprocess

ROOT = Path(__file__).resolve().parents[2]


def load(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


paired = load('continue_paired', 'foundation-paired-scenarios.py')
runner = paired.runner
GENERATOR = ROOT / 'engine/target/debug/floor'
# Per start-room door slot (left, up, right, down): the setup that lines Isaac
# up with the doorway from his spawn (320, 400), then the key held into it
# and its frames, and the key that walks back toward the entered door.
ROUTES = {0: ([('w', 27)], 'a', 72, 'd'), 1: ([], 'w', 70, 's'),
          2: ([('w', 27)], 'd', 72, 'a'), 3: ([], 's', 6, 'w')}
# Neither secret room type is reachable without a bomb.
UNREACHABLE = (7, 8)


class Tape:
    def __init__(self):
        self.actions = []

    def key(self, name, down):
        self.actions.append({'type': 'key', 'key': name, 'down': down})

    def step(self, count):
        self.actions.append({'type': 'step', 'frames': count})

    def capture(self, label):
        self.actions.append({'type': 'capture', 'label': label})

    def hold(self, name, frames):
        self.key(name, True); self.step(frames); self.key(name, False)

    def press(self, name, frames, settle):
        self.key(name, True); self.step(frames); self.key(name, False); self.step(settle)

    def walk(self, slot):
        setup, towards, frames, _ = ROUTES[slot]
        for name, count in setup:
            self.hold(name, count); self.step(30)
        self.hold(towards, frames)

    def quit_continue(self, prefix):
        # Pause, select Exit, then Main's Continue (selected by default once a
        # saved run exists). No capture while a menu owns the screen.
        self.capture(f'{prefix}pre-pause')
        self.press('Escape', 1, 29)
        self.press('Down', 1, 3)
        self.capture(f'{prefix}before-quit')
        self.press('Return', 3, 80)
        self.press('Return', 3, 57)
        self.capture(f'{prefix}cont-60')
        self.step(240); self.capture(f'{prefix}cont-300')
        self.step(72); self.capture(f'{prefix}cont-372')


def floor(run_seed):
    result = subprocess.run([str(GENERATOR)], input=json.dumps({'stageSeed': runner.codec.initial_stage_seed(run_seed)}),
                            capture_output=True, text=True, timeout=20, check=True)
    return json.loads(result.stdout)


def neighbors(run_seed):
    generated = floor(run_seed)
    start = generated['descriptors'][generated['startingDescriptor']]
    return [(slot, generated['descriptors'][index]['template']['type'])
            for slot, index in enumerate(start['neighbors'][:4])
            if index is not None and generated['descriptors'][index]['template']['type'] not in UNREACHABLE]


def uncleared(slot, idle):
    tape = Tape()
    tape.step(60); tape.capture('run-ready')
    tape.walk(slot); tape.step(40); tape.capture('in-room')
    tape.step(idle)
    tape.quit_continue('')
    back = ROUTES[slot][3]
    tape.key(back, True); tape.step(8); tape.capture('cont-move'); tape.key(back, False)
    tape.step(24); tape.capture('cont-coast')
    return tape.actions


def treasure_enter_then_quit():
    tape = Tape()
    tape.step(60); tape.capture('run-ready')
    tape.walk(2); tape.step(40); tape.capture('in-room')
    tape.quit_continue('')
    # From the left door inset (80, 280) to the pedestal at (320, 280).
    tape.hold('d', 60); tape.step(90); tape.capture('collected')
    tape.hold('a', 72); tape.step(40); tape.capture('returned')
    tape.quit_continue('second-')
    return tape.actions


def treasure_quit_then_enter():
    tape = Tape()
    tape.step(60); tape.capture('run-ready')
    tape.hold('w', 27); tape.step(30)
    tape.quit_continue('')
    # Continue places Isaac at a door of the fresh start room (a shared-MT
    # draw); the same keys then cross the right door from the bottom one.
    tape.hold('w', 27); tape.step(30); tape.hold('d', 72); tape.step(40)
    tape.capture('in-room')
    return tape.actions


def case(code, kind, actions, randomized, provenance, extra=None):
    result = paired.build('paired-bomb-moving-even', code)
    for name in ('tape', 'pairedTape'):
        start = next(index for index, action in enumerate(result[name]) if action.get('label') == 'gameplay-start')
        result[name] = result[name][:start + 1] + actions
    runner.validate_tape(result['tape'])
    result['scenario'] = {'id': f'continue-{kind}-{code}', 'description': f'Exit mid-run and Continue: {kind} ({provenance})',
                          'systems': ['save-continue', 'saved-run', 'room-connectivity', 'item-pools', 'input-determinism'],
                          'randomized': randomized}
    result['continueRun'] = {'kind': kind, 'provenance': provenance, **(extra or {})}
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--count', type=int, default=1, help='OS-random seeds')
    args = parser.parse_args()
    cases = [
        case('KRAYFRWA', 'uncleared', uncleared(0, 60), False, 'flies that moved before Exit', {'slot': 0}),
        case('3YSTBQPG', 'treasure-enter-then-quit', treasure_enter_then_quit(), False,
             'pedestal rolled before Exit, collected after Continue, second Exit in the start room',
             {'pedestalGroup': '3YSTBQPG'}),
        case('3YSTBQPG', 'treasure-quit-then-enter', treasure_quit_then_enter(), False,
             'Exit before the pedestal is rolled', {'pedestalGroup': '3YSTBQPG'}),
    ]
    entropy = random.SystemRandom()
    while len(cases) < 3 + args.count:
        vector = runner.codec.random_vector()
        slots = neighbors(vector['seedValue'])
        if not slots:
            continue
        slot, room_type = entropy.choice(slots)
        idle = entropy.randrange(0, 121)
        cases.append(case(runner.codec.encode_seed(vector['seedValue']), 'neighbour', uncleared(slot, idle), True,
                          'recorded OS-random run seed, neighbour and idle',
                          {'slot': slot, 'roomType': room_type, 'idle': idle}))
    print(json.dumps({'schema': 1, 'failures': [], 'cases': cases}))


if __name__ == '__main__':
    main()
