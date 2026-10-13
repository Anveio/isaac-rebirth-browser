#!/usr/bin/env python3
"""Capture original menu/gameplay pixels and timed motion through public X11 APIs.

SIGSTOP provides a stable displayed frame only; neither video frames nor process
stops are original simulation ticks. This script never inspects gameplay code.
"""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import signal
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('debug_run', Path(__file__).with_name('debug-run.py'))
debug = importlib.util.module_from_spec(spec)
spec.loader.exec_module(debug)
seed_spec = importlib.util.spec_from_file_location('foundation_seeds', Path(__file__).with_name('foundation-seeds.py'))
seed_codec = importlib.util.module_from_spec(seed_spec)
seed_spec.loader.exec_module(seed_codec)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--seed', default='T1PK 6Y0W')
    parser.add_argument('--probe-unvalidated-seed', action='store_true', help='Observe original acceptance/rejection of boundary or malformed seed text')
    parser.add_argument('--setup-only', action='store_true', help='Explicit menu/seed native probe; gameplay tape/video belongs to the full capture case')
    parser.add_argument('--move-key', choices=('w', 'a', 's', 'd'), default='d', help='Public keyboard movement direction for the full capture tape')
    parser.add_argument('--gl-trace', type=Path, help='Graphics-only observer shared library; never gameplay code')
    parser.add_argument('--output', type=Path, default=ROOT / '.cache/foundation' / f'native-{time.time_ns()}')
    args = parser.parse_args()
    try:
        seed_value = seed_codec.decode_seed(args.seed)
    except ValueError:
        if not args.probe_unvalidated_seed:
            raise
        seed_value = None
    if not os.environ.get('DISPLAY'):
        raise SystemExit('Run through tools/original/with-display.sh')
    out = args.output.resolve()
    if (out / 'profile').exists():
        raise SystemExit('Use a new output directory for a fresh profile')
    local = out / 'profile/data/binding of isaac rebirth'
    cloud = out / 'profile/cloud'
    for directory in (local, cloud, out / 'profile/config', out / 'game', out / 'frames'):
        directory.mkdir(parents=True, exist_ok=True)
    source = ROOT / '.cache/original-runtime/offline-depot'
    for name in ('isaac.x64', 'lib64', 'resources', 'run-x64.sh'):
        (out / 'game' / name).symlink_to(source / name)
    (local / 'options.ini').write_text('[Options]\nFullscreen=0\nFilter=0\nVSync=1\nPauseOnFocusLost=0\nSteamCloud=1\nMouseControl=0\nWindowWidth=960\nWindowHeight=540\nWindowPosX=0\nWindowPosY=0\nMusicVolume=0\nSFXVolume=0\n')
    env = os.environ.copy()
    env.update({'HOME': str(out / 'profile'), 'XDG_DATA_HOME': str(out / 'profile/data'),
                'XDG_CONFIG_HOME': str(out / 'profile/config'),
                'ISAAC_OFFLINE_CLOUD_DIR': str(cloud), 'ALSOFT_DRIVERS': 'null'})
    if args.gl_trace:
        trace_directory = out / 'gl-trace'
        trace_directory.mkdir(exist_ok=True)
        env['LD_PRELOAD'] = str(args.gl_trace.resolve()) + (' '+env['LD_PRELOAD'] if env.get('LD_PRELOAD') else '')
        env['ISAAC_GL_TRACE_DIR'] = str(trace_directory)
        env['ISAAC_GL_TRACE_CONTROL'] = str(trace_directory / 'request')
        env['ISAAC_GL_TRACE_STOP_AFTER_FRAME'] = '1'
    compact = args.seed.replace(' ', '').upper()
    started = time.monotonic()
    events, frames = [], []
    proc = None
    video = None
    window = None
    video_started = None
    frozen = False
    report = {'schema': 1, 'seed': compact[:4] + ' ' + compact[4:], 'status': 'running',
              'frames': frames, 'inputs': events, 'output': str(out),
              'captureClock': 'wall clock; SIGSTOP-stable framebuffer checkpoints',
              'simulationTicks': False, 'fullInternalState': False,
              'logicalViewport': {'width': 480, 'height': 270, 'scale': 2},
              'captureCase': 'menu-seed-probe' if args.setup_only else 'menu-movement-fire',
              'movementKey': args.move_key,
              'profile': {'fresh': True, 'isolated': True}, 'failures': []}

    def stamp():
        return (time.monotonic() - started) * 1000

    def key(name, duration=180, settle=1200, phase='menu'):
        at = stamp()
        wall_at = time.time()*1000
        subprocess.run(['xdotool', 'keydown', name], check=True)
        time.sleep(duration / 1000)
        subprocess.run(['xdotool', 'keyup', name], check=True)
        events.append({'key': name, 'phase': phase, 'atMs': at, 'wallStartedMs': wall_at,
                       'requestedHoldMs': duration, 'releasedAtMs': stamp(), 'wallReleasedMs': time.time()*1000,
                       'relativeVideoMs': None if video_started is None else at - video_started})
        time.sleep(settle / 1000)
        if proc.poll() is not None:
            raise RuntimeError(f'Original exited: {proc.returncode}')

    def capture(label):
        nonlocal frozen
        graphics_frame = None
        if args.gl_trace:
            trace_path = trace_directory / 'commands.jsonl'
            previous_ends = sum('\"op\":\"frameEnd\"' in line for line in trace_path.read_text().splitlines()) if trace_path.exists() else 0
            (trace_directory / 'request').write_text(label+'\n')
            deadline = time.monotonic()+10
            while time.monotonic()<deadline:
                if trace_path.exists():
                    ends = [json.loads(line) for line in trace_path.read_text().splitlines() if '\"op\":\"frameEnd\"' in line]
                    if len(ends)>previous_ends:
                        graphics_frame=ends[-1]
                        break
                if proc.poll() is not None:
                    raise RuntimeError('Original exited during graphics frame capture')
                time.sleep(.02)
            if graphics_frame is None:
                raise RuntimeError('Timed out waiting for a complete public graphics frame')
        frozen_at = stamp()
        os.killpg(proc.pid, signal.SIGSTOP)
        frozen = True
        time.sleep(.08)
        geometry = subprocess.check_output(['xdotool', 'getwindowgeometry', '--shell', window], text=True)
        values = dict(line.split('=', 1) for line in geometry.splitlines() if '=' in line)
        path = out / 'frames' / f'{label}.png'
        subprocess.run(['ffmpeg', '-hide_banner', '-loglevel', 'error', '-y', '-f', 'x11grab', '-draw_mouse', '0',
                        '-video_size', f"{values['WIDTH']}x{values['HEIGHT']}", '-i',
                        f"{os.environ['DISPLAY']}+{values['X']},{values['Y']}", '-frames:v', '1', str(path)],
                       check=True, timeout=15)
        frames.append({'label': label, 'path': str(path), 'atMs': stamp(),
                       'frozenAtMs': frozen_at, 'captureCompletedAtMs': stamp(),
                       'atMsMeaning': 'capture completed while process stopped',
                       'width': int(values['WIDTH']), 'height': int(values['HEIGHT']),
                       'processStopped': True, 'simulationTick': None,
                       'graphicsFrame': graphics_frame})
        os.killpg(proc.pid, signal.SIGCONT)
        frozen = False
        return path

    try:
        with (out / 'launch.log').open('w') as log:
            proc = subprocess.Popen(['bash', './run-x64.sh'], cwd=out / 'game', env=env,
                                    stdout=log, stderr=log, start_new_session=True)
            for _ in range(200):
                found = subprocess.run(['xdotool', 'search', '--onlyvisible', '--name', '(Isaac|Rebirth)'],
                                       capture_output=True, text=True)
                if found.returncode == 0 and found.stdout.strip():
                    window = found.stdout.splitlines()[-1]
                    break
                if proc.poll() is not None:
                    raise RuntimeError(f'Original exited before displaying: {proc.returncode}')
                time.sleep(.1)
            if window is None:
                raise RuntimeError('No visible original window')
            subprocess.run(['xdotool', 'windowfocus', '--sync', window], check=True)
            time.sleep(5)
            capture('intro')
            for label in ('title', 'profile', 'main', 'character'):
                key('Return')
                capture(label)
            key('Tab', settle=1800)
            for character in compact:
                key(character.lower(), settle=70, phase='seed-entry')
            capture('seed-entry')
            key('Return')
            capture('character-seeded')
            subprocess.run(['xdotool', 'keyup', 'w', 'a', 's', 'd', 'Up', 'Down', 'Left', 'Right'], check=True)
            key('Return', settle=5000)
            report['initialState'] = debug.observed_state((local / 'log.txt').read_text())
            if seed_value is not None and (report['initialState'].get('seed') != report['seed'] or report['initialState'].get('seedValue') != seed_value):
                raise RuntimeError(f"Original seed mismatch: {report['initialState'].get('seed')}")
            report['seedReadbackMatchesCodec'] = True if seed_value is not None else None
            if seed_value is None:
                report['unvalidatedSeedObservation'] = {'requested': report['seed'],
                    'actualCode': report['initialState'].get('seed'),
                    'actualValue': report['initialState'].get('seedValue'),
                    'acceptedRequestedCode': report['initialState'].get('seed') == report['seed']}
            model_seed = seed_value if seed_value is not None else report['initialState'].get('seedValue', 0)
            report['publicStage1Model'] = {
                'expected': seed_codec.initial_stage_seed(model_seed),
                'actual': report['initialState'].get('stageSeed'),
                'matches': report['initialState'].get('stageSeed') == seed_codec.initial_stage_seed(model_seed),
                'scope': 'public start RNG index27 secondNext; other stream routing unverified'}
            if args.gl_trace:
                report['graphicsTrace'] = {'path': str(trace_directory / 'commands.jsonl'),
                                           'scope': 'documented GL submitted data and textures; complete render frames, not gameplay state/ticks'}
            capture('gameplay-start')
            time.sleep(.5)
            capture('gameplay-repeat')
            key('Escape')
            capture('pause-seed')
            key('Escape')
            capture('gameplay-unpaused')
            if args.setup_only:
                report['video'] = None
            else:
                # Record a real X11 video. Its nominal 60fps is not native simulation Hz.
                video_path = out / 'movement-fire.mkv'
                video_started = stamp()
                video_geometry = subprocess.check_output(['xdotool', 'getwindowgeometry', '--shell', window], text=True)
                video_values = dict(line.split('=', 1) for line in video_geometry.splitlines() if '=' in line)
                video_input = f"{os.environ['DISPLAY']}+{video_values['X']},{video_values['Y']}"
                video = subprocess.Popen(['ffmpeg', '-hide_banner', '-loglevel', 'verbose', '-debug_ts', '-y', '-f', 'x11grab',
                                          '-draw_mouse', '0', '-framerate', '60', '-video_size', '960x540',
                                          '-i', video_input, '-c:v', 'ffv1',
                                          '-pix_fmt', 'bgr0', str(video_path)], stdin=subprocess.PIPE,
                                         stdout=subprocess.DEVNULL, stderr=(out / 'video.log').open('w'))
                time.sleep(.6)
                key(args.move_key, duration=450, settle=500, phase='gameplay')
                capture('movement')
                time.sleep(.2)
                key('Right', duration=900, settle=0, phase='gameplay')
                capture('shooting')
                time.sleep(.5)
                capture('after-fire')
                if video.stdin:
                    video.stdin.write(b'q\n')
                    video.stdin.flush()
                video.wait(timeout=10)
                if video.returncode:
                    raise RuntimeError(f'Video capture failed: {video.returncode}')
                report['video'] = {'path': str(video_path), 'nominalFps': 60,
                                   'startedAtMs': video_started, 'simulationTickRate': None,
                                   'windowOriginRender': [int(video_values['X']), int(video_values['Y'])],
                                   'captureMatchesWindowGeometry': True, 'timestampLog': str(out / 'video.log')}
            report['finalLogState'] = debug.observed_state((local / 'log.txt').read_text())
            report['performance'] = debug.process_performance(proc.pid, started)
            report['status'] = 'pass'
    except Exception as error:
        report['status'] = 'fail'
        report['failures'].append(str(error))
    finally:
        if proc is not None and proc.poll() is None:
            if frozen:
                os.killpg(proc.pid, signal.SIGCONT)
            os.killpg(proc.pid, signal.SIGTERM)
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                os.killpg(proc.pid, signal.SIGKILL)
                proc.wait()
        if video is not None and video.poll() is None:
            video.terminate()
            video.wait(timeout=5)
        if (local / 'log.txt').exists():
            (out / 'original.log').write_text((local / 'log.txt').read_text())
        (out / 'manifest.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({'status': report['status'], 'manifest': str(out / 'manifest.json'), 'failures': report['failures']}))
    return 0 if report['status'] == 'pass' else 1


if __name__ == '__main__':
    raise SystemExit(main())
