#!/usr/bin/env python3
"""Seeded black-box native debug run. Wall-clock input is explicitly not lockstep.

Add cases in debug-config.json; expose original state only through observable
logs/UI and documented host APIs. Never inspect gameplay machine instructions.
Each run gets a private home, local options, cloud save dir, and launch cwd.
"""
import argparse
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]
DEFAULT_CONFIG = Path(__file__).with_name('debug-config.json')


def observed_state(text):
    # Original-produced log fields: useful seed/room state, not a full serializer.
    patterns = {
        'seed': r'RNG Start Seed: ([A-Z0-9]{4} [A-Z0-9]{4})',
        'seedValue': r'RNG Start Seed: [A-Z0-9]{4} [A-Z0-9]{4} \((\d+)\)',
        'playerVariant': r'Initialized player with Variant (\d+)',
        'playerSubtype': r'Initialized player with Variant \d+ and Subtype (\d+)',
        'stage': r'Level::Init m_Stage (\d+)',
        'altStage': r'Level::Init m_Stage \d+, m_AltStage (\d+)',
        'stageSeed': r'Level::Init m_Stage \d+, m_AltStage \d+ Seed (\d+)',
        'rooms': r'\n(\d+) rooms in \d+ loops',
        'room': r'\nRoom ([^\r\n]+)',
        'spawnSeed': r'SpawnRNG seed: (\d+)'
    }
    state = {}
    for key, pattern in patterns.items():
        matches = re.findall(pattern, text)
        if matches:
            state[key] = matches[-1] if key in ('seed', 'room') else int(matches[-1])
    return state


def pixel_diff(a, b):
    from PIL import Image, ImageChops
    with Image.open(a).convert('RGB') as first, Image.open(b).convert('RGB') as second:
        if first.size != second.size:
            raise RuntimeError('Capture dimensions differ')
        diff = ImageChops.difference(first, second)
        raw = diff.tobytes()
        changed = sum(any(raw[index:index + 3]) for index in range(0, len(raw), 3))
        return {'changedPixels': changed, 'totalPixels': first.width * first.height,
                'equal': changed == 0,
                'scope': 'whole game window; animation also changes pixels'}


def tutorial_pixels(path):
    # Black-box image observations calibrated for the known-seed Isaac tutorial
    # room only: skin centroid and blue projectile pixels. This is not native
    # x/y/health state; other rooms/characters require separate pixel observers.
    from PIL import Image
    with Image.open(path).convert('RGB') as image:
        skin = []
        blue = 0
        for y in range(120, min(450, image.height)):
            for x in range(140, min(810, image.width)):
                r, g, b = image.getpixel((x, y))
                if min(r, g, b) > 140 and max(r, g, b) - min(r, g, b) < 40:
                    skin.append((x, y))
                if b - r > 25 and g - r > 15 and b > 100:
                    blue += 1
        return {'skinPixels': len(skin),
                'skinCentroid': [sum(p[i] for p in skin) / len(skin) for i in range(2)] if skin else None,
                'bluePixels': blue,
                'scope': 'known-seed tutorial room rendered pixels, not internal state'}


def process_performance(pid, started):
    stat = Path(f'/proc/{pid}/stat').read_text().split(')', 1)[1].split()
    lines = Path(f'/proc/{pid}/status').read_text().splitlines()
    hwm = next(int(line.split()[1]) * 1024 for line in lines if line.startswith('VmHWM:'))
    hz = os.sysconf('SC_CLK_TCK')
    return {'wallSeconds': time.monotonic() - started,
            'cpuUserSeconds': int(stat[11]) / hz,
            'cpuSystemSeconds': int(stat[12]) / hz,
            'rssBytes': int(stat[21]) * os.sysconf('SC_PAGE_SIZE'),
            'peakRssBytes': hwm,
            'scope': 'native process lifetime including menu/capture waits; helper CPU excluded',
            'perSimulationTickMs': None}


def run(args):
    config = json.loads(args.config.read_text())
    if config.get('schema') != 1:
        raise ValueError('Unsupported debug config schema')
    compact_seed = (args.seed or config['seed']).replace(' ', '').upper()
    if not re.fullmatch(r'[A-Z0-9]{8}', compact_seed):
        raise ValueError('Seed must contain eight letters/digits; original log must confirm acceptance')
    seed = compact_seed[:4] + ' ' + compact_seed[4:]
    if args.case not in config['cases']:
        raise ValueError(f'Unknown debug case: {args.case}')
    # Never silently drop unsupported logical actions or malformed scenario fields.
    catalog = json.loads((ROOT / 'tools/verification/input-catalog.json').read_text())
    actions = {action['id']: action for action in catalog['actions']}
    for item in config['cases'][args.case]:
        action = actions.get(item.get('action'))
        if action is None or not action.get('x11'):
            raise ValueError(f'Unsupported native action: {item}')
        item['key'] = action['x11']
        for name in ('holdMs', 'waitMs'):
            if not isinstance(item.get(name), int) or not 0 <= item[name] <= 10000:
                raise ValueError(f'Invalid {name}: {item}')
    if not os.environ.get('DISPLAY'):
        raise RuntimeError('Run through tools/original/with-display.sh')
    source = ROOT / '.cache/original-runtime/offline-depot'
    if not (source / 'isaac.x64').exists() or not (source / 'provenance.json').exists():
        raise RuntimeError('Run bash tools/original/prepare-offline.sh first')
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=True)
    profile = out / 'profile'
    if profile.exists():
        raise RuntimeError(f'Output profile already exists; use a fresh output path: {profile}')
    local = profile / 'data' / 'binding of isaac rebirth'
    cloud = profile / 'cloud'
    for directory in (local, cloud, profile / 'config', out / 'frames', out / 'game'):
        directory.mkdir(parents=True, exist_ok=True)
    game = out / 'game'
    for name in ('isaac.x64', 'lib64', 'resources', 'run-x64.sh'):
        (game / name).symlink_to(source / name)
    # Documented option keys observed in the original-created options.ini.
    # Modern Repentance debug-console switches do not exist in base Rebirth.
    options = {'Fullscreen': 0, 'Filter': 0, 'VSync': 1, 'PauseOnFocusLost': 1,
               'SteamCloud': 1, 'MouseControl': 0, 'WindowWidth': 960,
               'WindowHeight': 540, 'WindowPosX': 0, 'WindowPosY': 0,
               'MusicVolume': '0.0000', 'SFXVolume': '0.0000'}
    (local / 'options.ini').write_text('[Options]\n' + ''.join(f'{k}={v}\n' for k, v in options.items()))
    env = os.environ.copy()
    # This only configures the subprocess; caller environment and host saves are untouched.
    env.update({'HOME': str(profile), 'XDG_DATA_HOME': str(profile / 'data'),
                'XDG_CONFIG_HOME': str(profile / 'config'),
                'ISAAC_OFFLINE_CLOUD_DIR': str(cloud), 'ALSOFT_DRIVERS': 'null'})
    report = {'schema': 1, 'source': 'offline-derived-original', 'seed': seed,
              'case': args.case, 'status': 'running', 'output': str(out),
              'actualSeed': None,
              'profile': {'fresh': True, 'isolated': True, 'slot': 1, 'path': str(profile)},
              'capabilities': {'inputClock': 'wall-clock', 'simulationTicks': False,
                               'fullInternalState': False, 'pauseAtTick': False,
                               'originalLogState': True, 'seedReadback': False, 'renderedPixels': True,
                               'processPerformance': True},
              'inputs': [], 'frames': [], 'observations': {}, 'failures': [],
              'unavailableGuarantees': ['native simulation-tick input scheduling',
                                        'native full gameplay/hidden/metagame state serializer',
                                        'equal-input cross-game state and pixel parity',
                                        'comparable per-simulation-tick performance']}
    proc = None
    existing_windows = set(subprocess.run(
        ['xdotool', 'search', '--onlyvisible', '--name', '(Isaac|Rebirth)'],
        capture_output=True, text=True).stdout.splitlines())
    started = time.monotonic()
    timing = config['timing']
    window = None

    def alive():
        if proc.poll() is not None:
            raise RuntimeError(f'Original exited with {proc.returncode}; inspect launch.log')

    def wait(ms):
        time.sleep(ms / 1000)
        alive()

    def key(key_name, hold_ms, settle_ms, phase, action=None):
        now = (time.monotonic() - started) * 1000
        subprocess.run(['xdotool', 'keydown', key_name], check=True)
        wait(hold_ms)
        subprocess.run(['xdotool', 'keyup', key_name], check=True)
        report['inputs'].append({'phase': phase, 'action': action or key_name,
                                 'key': key_name, 'kind': 'hold', 'atMs': now,
                                 'requestedHoldMs': hold_ms,
                                 'actualHoldMs': (time.monotonic() - started) * 1000 - now})
        wait(settle_ms)

    def capture(label):
        geometry = subprocess.check_output(['xdotool', 'getwindowgeometry', '--shell', window], text=True)
        values = dict(line.split('=', 1) for line in geometry.splitlines() if '=' in line)
        path = out / 'frames' / f'{label}.png'
        subprocess.run(['ffmpeg', '-hide_banner', '-loglevel', 'error', '-y',
                        '-f', 'x11grab', '-draw_mouse', '0', '-video_size',
                        f"{values['WIDTH']}x{values['HEIGHT']}", '-i',
                        f"{os.environ['DISPLAY']}+{values['X']},{values['Y']}",
                        '-frames:v', '1', str(path)], check=True, timeout=15)
        report['frames'].append({'label': label, 'path': str(path),
                                 'width': int(values['WIDTH']), 'height': int(values['HEIGHT']),
                                 'atMs': (time.monotonic() - started) * 1000})
        return path

    try:
        with (out / 'launch.log').open('w') as log:
            proc = subprocess.Popen(['bash', './run-x64.sh'], cwd=game, env=env,
                                    stdout=log, stderr=log, start_new_session=True)
            for _ in range(200):
                alive()
                found = subprocess.run(['xdotool', 'search', '--onlyvisible', '--name', '(Isaac|Rebirth)'],
                                       capture_output=True, text=True)
                new_windows = [identifier for identifier in found.stdout.splitlines()
                               if identifier not in existing_windows]
                if found.returncode == 0 and new_windows:
                    window = new_windows[-1]
                    break
                wait(100)
            if not window:
                raise RuntimeError('No original game window within 20 seconds')
            report['observations']['window'] = True
            subprocess.run(['xdotool', 'windowfocus', '--sync', window], check=True)
            wait(timing['bootMs'])
            capture('boot')
            # Fresh profile only: skip intro, title, select profile 1, new run.
            for index in range(4):
                key('Return', timing['keyHoldMs'], timing['menuSettleMs'], 'setup')
            capture('character-selection')
            key('Tab', timing['keyHoldMs'], timing['seedSettleMs'], 'setup')
            # xdotool type may send presses shorter than original key-state polling;
            # hold each seed character long enough and capture the actual seed UI.
            for character in compact_seed:
                key(character.lower(), timing['keyHoldMs'], 70, 'seed-entry')
            capture('seed-entry')
            report['observations']['seedEntrySent'] = True
            key('Return', timing['keyHoldMs'], timing['menuSettleMs'], 'setup')
            # Release controls at the menu/run boundary to avoid residual key holds.
            subprocess.run(['xdotool', 'keyup', 'w', 'a', 's', 'd', 'Up', 'Down', 'Left', 'Right'], check=True)
            key('Return', timing['keyHoldMs'], timing['startSettleMs'], 'setup')
            first = capture('gameplay-start')
            log_path = local / 'log.txt'
            initial = observed_state(log_path.read_text())
            report['observations']['initialState'] = initial
            report['actualSeed'] = initial.get('seed')
            if initial.get('seed') != seed:
                raise RuntimeError(f"Requested seed {seed}, original accepted {initial.get('seed')}; see seed-entry.png")
            report['observations']['seedConfirmedByOriginalLog'] = True
            report['capabilities']['seedReadback'] = True
            expected = config.get('knownSeeds', {}).get(seed, {})
            mismatches = {name: {'expected': value, 'actual': initial.get(name)}
                          for name, value in expected.items() if initial.get(name) != value}
            if mismatches:
                raise RuntimeError(f'Known-seed state changed: {mismatches}')
            report['observations']['expectedInitialStateMatches'] = True if expected else None
            key('Escape', timing['keyHoldMs'], timing['menuSettleMs'], 'setup')
            capture('pause-seed')
            key('Escape', timing['keyHoldMs'], timing['menuSettleMs'], 'setup')
            previous = capture('before-input')
            initial_pixels = tutorial_pixels(previous)
            wait(500)
            idle_frame = capture('idle-control')
            idle_pixels = tutorial_pixels(idle_frame)
            report['observations']['idleControl'] = {
                'before': initial_pixels, 'after': idle_pixels, **pixel_diff(previous, idle_frame)}
            previous = idle_frame
            previous_pixels = idle_pixels
            responses = []
            for item in config['cases'][args.case]:
                key(item['key'], item['holdMs'], item['waitMs'], 'scenario', item['action'])
                current = capture(item['capture'])
                current_pixels = tutorial_pixels(current)
                responses.append({'action': item['action'], **pixel_diff(previous, current),
                                  'beforePixels': previous_pixels, 'afterPixels': current_pixels})
                previous = current
                previous_pixels = current_pixels
            report['observations']['inputResponse'] = responses
            if not responses or not any(item['changedPixels'] > 0 for item in responses):
                raise RuntimeError('No visible pixel response to scenario input')
            if args.case == 'movement-fire' and seed in config.get('knownSeeds', {config['seed']: {}}):
                movement, shooting = responses
                before_position = movement['beforePixels']['skinCentroid']
                after_position = movement['afterPixels']['skinCentroid']
                if before_position is None or after_position is None:
                    raise RuntimeError('Isaac tutorial-room sprite was not identified in input captures')
                displacement = after_position[0] - before_position[0]
                idle_before = initial_pixels['skinCentroid']
                idle_after = idle_pixels['skinCentroid']
                if idle_before is None or idle_after is None:
                    raise RuntimeError('Isaac tutorial-room sprite was not identified in idle control')
                idle_displacement = abs(idle_after[0] - idle_before[0])
                projectile_gain = shooting['afterPixels']['bluePixels'] - shooting['beforePixels']['bluePixels']
                report['observations']['actionResponse'] = {
                    'moveRightDisplacementPixels': displacement,
                    'idleDisplacementPixels': idle_displacement,
                    'shootRightBluePixelGain': projectile_gain,
                    'scope': 'known tutorial room rendered pixel observer; no gameplay-memory inspection'}
                if displacement <= 20 or idle_displacement >= 8 or projectile_gain <= 20:
                    raise RuntimeError('Known-room movement/projectile pixel response did not pass idle-controlled checks')
            report['observations']['finalLogState'] = observed_state(log_path.read_text())
            report['observations']['stateCoverage'] = 'seed/player/stage/map/room/spawn log fields only'
            report['performance'] = process_performance(proc.pid, started)
            report['status'] = 'pass'
    except Exception as error:
        report['status'] = 'fail'
        report['failures'].append(str(error))
    finally:
        if proc is not None and proc.poll() is None:
            os.killpg(proc.pid, signal.SIGTERM)
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                os.killpg(proc.pid, signal.SIGKILL)
                proc.wait()
        report['profile']['files'] = [str(p.relative_to(profile)) for location in (local, cloud) for p in sorted(location.rglob('*')) if p.is_file()]
        if (local / 'options.ini').exists():
            report['profile']['options'] = (local / 'options.ini').read_text()
        if (local / 'log.txt').exists():
            (out / 'original.log').write_text((local / 'log.txt').read_text())
        (out / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--config', type=Path, default=DEFAULT_CONFIG)
    parser.add_argument('--seed', help='Actual base Rebirth eight-character seed; confirmed from original log')
    parser.add_argument('--case', default='movement-fire')
    parser.add_argument('--output', type=Path, default=ROOT / '.cache/original-runtime' / f'debug-{time.time_ns()}')
    args = parser.parse_args()
    report = run(args)
    print(json.dumps({'status': report['status'], 'seed': report['seed'], 'case': report['case'],
                      'output': report['output'], 'failures': report['failures'],
                      'capabilities': report['capabilities']}))
    return 0 if report['status'] == 'pass' else 1


if __name__ == '__main__':
    raise SystemExit(main())
