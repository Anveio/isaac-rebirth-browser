#!/usr/bin/env python3
"""Run an original input tape at an explicit virtual clock/presentation gate.

The measured frame count is GL presentation, not proven simulation ticks. Only
public time, X11 input and rendered pixels are observed. Fresh random seeds are
recorded; saved tapes can be replayed with the same seed.
"""
import argparse
import ctypes
import importlib.util
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[2]

def module(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result

codec = module('foundation_seeds', 'foundation-seeds.py')
debug = module('debug_run', 'debug-run.py')


def default_tape(seed):
    tape = [{'type': 'step', 'frames': 180}, {'type': 'capture', 'label': 'intro'}]
    for label in ('title', 'profile', 'main', 'character'):
        tape += [{'type': 'key', 'key': 'Return', 'down': True}, {'type': 'step', 'frames': 12},
                 {'type': 'key', 'key': 'Return', 'down': False}, {'type': 'step', 'frames': 72},
                 {'type': 'capture', 'label': label}]
    tape += [{'type': 'key', 'key': 'Tab', 'down': True}, {'type': 'step', 'frames': 12},
             {'type': 'key', 'key': 'Tab', 'down': False}, {'type': 'step', 'frames': 108}]
    for character in seed.replace(' ', '').lower():
        tape += [{'type': 'key', 'key': character, 'down': True}, {'type': 'step', 'frames': 12},
                 {'type': 'key', 'key': character, 'down': False}, {'type': 'step', 'frames': 6}]
    tape += [{'type': 'capture', 'label': 'seed-entry'}]
    for label, settle in [('character-seeded', 72), ('gameplay-start', 300)]:
        tape += [{'type': 'key', 'key': 'Return', 'down': True}, {'type': 'step', 'frames': 12},
                 {'type': 'key', 'key': 'Return', 'down': False}, {'type': 'step', 'frames': settle},
                 {'type': 'capture', 'label': label}]
    tape += [{'type': 'key', 'key': 'd', 'down': True}, {'type': 'step', 'frames': 27},
             {'type': 'key', 'key': 'd', 'down': False}, {'type': 'step', 'frames': 30},
             {'type': 'capture', 'label': 'movement'},
             {'type': 'key', 'key': 'Right', 'down': True}, {'type': 'step', 'frames': 54},
             {'type': 'key', 'key': 'Right', 'down': False},
             {'type': 'capture', 'label': 'shooting'}, {'type': 'step', 'frames': 30},
             {'type': 'capture', 'label': 'after-fire'}]
    return tape


def validate_tape(tape):
    if not isinstance(tape, list) or len(tape) > 36000:
        raise ValueError('Tape must be a list of at most 36000 public input/presentation commands')
    labels = set()
    total = 0
    for action in tape:
        kind = action.get('type')
        if kind == 'step':
            frames = action.get('frames')
            if type(frames) is not int or not 1 <= frames <= 36000:
                raise ValueError('step frames must be an integer 1..36000')
            total += frames
        elif kind == 'key':
            if type(action.get('down')) is not bool or not isinstance(action.get('key'), str) or not action['key'] or len(action['key']) > 64 or not action['key'][0].isalnum() or not all(c.isalnum() or c in '_-' for c in action['key']):
                raise ValueError('key requires an X11 keysym name and boolean down')
        elif kind == 'capture':
            label = action.get('label')
            if not isinstance(label, str) or not label or len(label) > 80 or not all(c.isalnum() or c in '-_' for c in label) or label in labels:
                raise ValueError('capture labels must be unique simple names')
            labels.add(label)
        else:
            raise ValueError(f'Unsupported tape action: {kind}')
    if total > 36000:
        raise ValueError('Tape has more than 36000 presented frames')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--seed', help='Valid seed; omitted means new OS-random seed')
    parser.add_argument('--tape', type=Path, help='List of key/step/capture commands; public X11 key names')
    parser.add_argument('--focus-handoff', action='append', default=[], help='At named capture, generate public X11 focus out/in and release one presentation before capture; diagnostic tape alteration is recorded')
    parser.add_argument('--state-observer', type=Path, help='Optional parent-only reference debug module exposing observe(pid, context); never production game input')
    parser.add_argument('--gl-trace', type=Path, help='Optional documented graphics API observer to chain; no game instructions/state')
    parser.add_argument('--trace-every-frame', action='store_true', help='Request graphics observer continuous draw/presentation tracing')
    parser.add_argument('--width', type=int, default=960, help='Original supported window width; larger values aid public subpixel diagnostics')
    parser.add_argument('--height', type=int, default=540, help='Original supported window height')
    parser.add_argument('--max-render-scale', type=int, default=2, help='Original supported MaxRenderScale option; normal comparisons use 2')
    parser.add_argument('--fps', type=int, default=60, help='Nominal presentation rate; not asserted native simulation Hz')
    parser.add_argument('--step-ns', type=int, default=16667000, help='Explicit virtual interval; default is 16667 microseconds, above native timer polling threshold')
    parser.add_argument('--output', type=Path, default=ROOT / '.cache/foundation' / f'clock-native-{time.time_ns()}')
    parser.add_argument('--frame-timeout', type=float, default=30)
    args = parser.parse_args()
    if not os.environ.get('DISPLAY'):
        raise SystemExit('Run through tools/original/with-display.sh')
    # X server autorepeat uses real host time; it must not invent input events
    # while the original virtual clock is gated or checkpoints are captured.
    x11 = ctypes.CDLL('libX11.so.6')
    x11.XOpenDisplay.argtypes = [ctypes.c_char_p]
    x11.XOpenDisplay.restype = ctypes.c_void_p
    x11.XStringToKeysym.argtypes = [ctypes.c_char_p]
    x11.XStringToKeysym.restype = ctypes.c_ulong
    x11.XGetInputFocus.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_ulong), ctypes.POINTER(ctypes.c_int)]
    x11.XGetInputFocus.restype = ctypes.c_int
    x11.XSetInputFocus.argtypes = [ctypes.c_void_p, ctypes.c_ulong, ctypes.c_int, ctypes.c_ulong]
    x11.XSetInputFocus.restype = ctypes.c_int
    x11.XAutoRepeatOff.argtypes = [ctypes.c_void_p]
    x11.XAutoRepeatOff.restype = ctypes.c_int
    x11.XSync.argtypes = [ctypes.c_void_p, ctypes.c_int]
    x11.XSync.restype = ctypes.c_int
    x11.XCloseDisplay.argtypes = [ctypes.c_void_p]
    x11.XCloseDisplay.restype = ctypes.c_int
    display = x11.XOpenDisplay(None)
    if not display:
        raise SystemExit('Could not open the private X11 display')
    try:
        x11.XAutoRepeatOff(display)
        x11.XSync(display, 0)
    finally:
        x11.XCloseDisplay(display)
    def x11_focus(window=None):
        display = x11.XOpenDisplay(None)
        if not display:
            raise RuntimeError('X11 focus diagnostic could not open display')
        try:
            if window is not None:
                x11.XSetInputFocus(display, 0, 2, 0)
                x11.XSync(display, 0)
                x11.XSetInputFocus(display, int(window), 2, 0)
                x11.XSync(display, 0)
            focused = ctypes.c_ulong()
            revert = ctypes.c_int()
            x11.XGetInputFocus(display, ctypes.byref(focused), ctypes.byref(revert))
            return {'window': focused.value, 'revertTo': revert.value}
        finally:
            x11.XCloseDisplay(display)
    if not 1 <= args.fps <= 1000 or not 1 <= args.step_ns <= 1000000000 or not 0 < args.frame_timeout <= 300:
        raise SystemExit('FPS must be 1..1000; step-ns 1..1000000000; frame timeout 0..300 seconds')
    if not 480 <= args.width <= 3840 or not 270 <= args.height <= 2160 or args.width*9 != args.height*16 or not 1 <= args.max_render_scale <= 8:
        raise SystemExit('Window must be 16:9, from 480x270 through 3840x2160')
    vector = {'code': args.seed, 'seedValue': codec.decode_seed(args.seed)} if args.seed else codec.random_vector()
    seed = codec.encode_seed(vector['seedValue'])
    tape = json.loads(args.tape.read_text()) if args.tape else default_tape(seed)
    validate_tape(tape)
    selected_labels = {a['label'] for a in tape if a['type'] == 'capture'}
    if not set(args.focus_handoff) <= selected_labels:
        raise SystemExit('Focus-handoff labels must exist in the selected tape')
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=True)
    if (out / 'profile').exists():
        raise SystemExit('Output already contains a profile; use a fresh directory')
    source = ROOT / '.cache/original-runtime/offline-depot'
    local = out / 'profile/data/binding of isaac rebirth'
    for directory in (local, out / 'profile/cloud', out / 'profile/config', out / 'game', out / 'frames'):
        directory.mkdir(parents=True, exist_ok=True)
    for name in ('isaac.x64', 'lib64', 'resources'):
        (out / 'game' / name).symlink_to(source / name)
    (local / 'options.ini').write_text(f'[Options]\nFullscreen=0\nFilter=0\nVSync=0\nPauseOnFocusLost=0\nSteamCloud=1\nMouseControl=0\nWindowWidth={args.width}\nWindowHeight={args.height}\nWindowPosX=0\nWindowPosY=0\nMaxRenderScale={args.max_render_scale}\nMusicVolume=0\nSFXVolume=0\n')
    gate = out / 'clock-gate.so'
    subprocess.run(['cc', '-std=c11', '-Wall', '-Wextra', '-Werror', '-fPIC', '-shared',
                    str(Path(__file__).with_name('foundation-clock-gate.c')), '-o', str(gate), '-ldl'], check=True)
    env = os.environ.copy()
    env.update({'HOME': str(out / 'profile'), 'XDG_DATA_HOME': str(out / 'profile/data'),
                'XDG_CONFIG_HOME': str(out / 'profile/config'), 'ISAAC_OFFLINE_CLOUD_DIR': str(out / 'profile/cloud'),
                'ALSOFT_DRIVERS': 'null', 'LD_PRELOAD': str(gate) + (' ' + env['LD_PRELOAD'] if env.get('LD_PRELOAD') else ''),
                'LD_LIBRARY_PATH': str(source / 'lib64') + (':' + env['LD_LIBRARY_PATH'] if env.get('LD_LIBRARY_PATH') else ''),
                'ISAAC_CLOCK_GATE_FPS': str(args.fps), 'ISAAC_CLOCK_GATE_STEP_NS': str(args.step_ns)})
    if args.gl_trace:
        trace = out / 'gl-trace'
        trace.mkdir(exist_ok=True)
        env['LD_PRELOAD'] = str(gate) + ' ' + str(args.gl_trace.resolve()) + (' ' + os.environ['LD_PRELOAD'] if os.environ.get('LD_PRELOAD') else '')
        env['ISAAC_GL_TRACE_DIR'] = str(trace)
        env['ISAAC_GL_TRACE_CONTROL'] = str(trace / 'request')
        env.pop('ISAAC_GL_TRACE_STOP_AFTER_FRAME', None)
        if args.trace_every_frame:
            env['ISAAC_GL_TRACE_EVERY_FRAME'] = '1'
    elif args.trace_every_frame:
        raise SystemExit('--trace-every-frame requires --gl-trace')
    observer = None
    if args.state_observer:
        spec = importlib.util.spec_from_file_location('original_reference_observer', args.state_observer.resolve())
        observer = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(observer)
        if not callable(getattr(observer, 'observe', None)):
            raise SystemExit('State observer must expose observe(pid, context)')
    frames, events, acknowledgements = [], [], []
    report = {'schema': 1, 'status': 'running', 'output': str(out),
              'captureClock': 'main-thread virtual Linux clocks and blocking public GL presentation gate; simulation ticks unknown', 'seed': seed, 'seedValue': vector['seedValue'],
              'requestedViewport': {'width': args.width, 'height': args.height, 'maxRenderScale': args.max_render_scale},
              'profile': {'fresh': True, 'isolated': True}, 'tape': tape, 'frames': frames,
              'inputs': events, 'presentations': acknowledgements,
              'clock': {'scope': 'main-thread wall-clock ABI; worker/CPU clocks unchanged', 'nominalStepsPerSecond': args.fps, 'stepNs': args.step_ns,
                        'mainThreadSleeps': 'recorded then returned without host delay', 'realPerformance': 'controller wall clock'},
              'inputDelivery': {'api': 'public X11 key events', 'serverAutoRepeat': False},
              'simulationTicks': False, 'fullInternalState': False, 'failures': []}
    if observer:
        report['debugObserver'] = {'module': str(args.state_observer.resolve()), 'scope': 'reference-only parent debug observations at blocked presentation checkpoints; no hidden-state parity claim'}
        report['debugObservations'] = []
    (out / 'tape.json').write_text(json.dumps(tape, indent=2) + '\n')
    proc = None
    started = time.monotonic()
    try:
        with tempfile.TemporaryDirectory(prefix='isaac-clock-') as socket_dir, (out / 'launch.log').open('w') as log:
            address = str(Path(socket_dir) / 'gate.sock')
            env['ISAAC_CLOCK_GATE_SOCKET'] = address
            proc = subprocess.Popen(['./isaac.x64'], cwd=out / 'game', env=env, stdout=log, stderr=log, start_new_session=True)
            connection = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            connection.settimeout(args.frame_timeout)
            deadline = time.monotonic() + args.frame_timeout
            while True:
                try:
                    connection.connect(address)
                    break
                except (FileNotFoundError, ConnectionRefusedError):
                    if proc.poll() is not None or time.monotonic() >= deadline:
                        raise RuntimeError('Original gate did not open; inspect launch.log')
                    time.sleep(.02)
            reader = connection.makefile('r', encoding='utf8')
            def acknowledgement():
                line = reader.readline()
                if not line:
                    raise RuntimeError('Original gate connection closed')
                status = json.loads(line)
                if 'error' in status:
                    raise RuntimeError(status['error'])
                if status.get('inputOverflow'):
                    raise RuntimeError('Public X11 observation buffer overflowed')
                status['controllerElapsedMs'] = (time.monotonic() - started) * 1000
                acknowledgements.append(status)
                return status
            current = acknowledgement()
            connection.sendall(b'step 30\n')
            current = acknowledgement()
            report['bootstrapPresentedFrames'] = 30
            windows = subprocess.check_output(['xdotool', 'search', '--onlyvisible', '--name', '(Isaac|Rebirth)'], text=True)
            window = windows.splitlines()[-1]
            subprocess.run(['xdotool', 'windowfocus', '--sync', window], check=True)
            for action in tape:
                kind = action['type']
                if kind == 'step':
                    before = current['presentedFrame']
                    cpu_before = debug.process_performance(proc.pid, started)
                    step_started = time.monotonic()
                    connection.sendall(f"step {action['frames']}\n".encode())
                    current = acknowledgement()
                    current['requestedPresentationFrames'] = action['frames']
                    current['advanceHostWallMs'] = (time.monotonic()-step_started)*1000
                    cpu_after = debug.process_performance(proc.pid, started)
                    user_ms = (cpu_after['cpuUserSeconds']-cpu_before['cpuUserSeconds'])*1000
                    system_ms = (cpu_after['cpuSystemSeconds']-cpu_before['cpuSystemSeconds'])*1000
                    current['advancePerformance'] = {'cpuUserMs': user_ms, 'cpuSystemMs': system_ms,
                        'nativeCpuMsPerPresentedFrame': (user_ms+system_ms)/action['frames'],
                        'hostWallMsPerPresentedFrame': current['advanceHostWallMs']/action['frames'],
                        'scope': 'native process CPU / controller wall time during released presentations; excludes checkpoint waits, includes observer cost'}
                    if current['presentedFrame'] != before + action['frames']:
                        raise RuntimeError('Presentation acknowledgement count diverged from requested tape')
                elif kind == 'key':
                    keysym = x11.XStringToKeysym(action['key'].encode('ascii'))
                    if not keysym:
                        raise RuntimeError('Unknown public X11 keysym: '+action['key'])
                    subprocess.run(['xdotool', 'keydown' if action['down'] else 'keyup', '--window', window, action['key']], check=True)
                    subprocess.run(['xdotool', 'getwindowfocus'], check=True, stdout=subprocess.DEVNULL)
                    events.append({**action, 'keysym': keysym, 'presentedFrame': current['presentedFrame'], 'virtualElapsedNs': current['virtualElapsedNs']})
                else:
                    focus_before = x11_focus()
                    if action['label'] in args.focus_handoff:
                        focus_after = x11_focus(window)
                        connection.sendall(b'step 1\n')
                        current = acknowledgement()
                        report.setdefault('focusHandoffs', []).append({'label': action['label'], 'before': focus_before, 'after': focus_after, 'releasedPresentations': 1, 'presentedFrame': current['presentedFrame']})
                    if action['label'] == 'gameplay-start':
                        report['initialState'] = debug.observed_state((local / 'log.txt').read_text())
                    if observer:
                        current_log = debug.observed_state((local / 'log.txt').read_text()) if (local / 'log.txt').exists() else {}
                        context = {'label': action['label'], 'seed': seed, 'seedValue': vector['seedValue'],
                            'presentedFrame': current['presentedFrame'], 'virtualElapsedNs': current['virtualElapsedNs'],
                            'output': str(out), 'publicLogState': current_log, 'gateBlocked': True,
                            'scope': 'reference debug data only; simulation ticks and full internal state not established'}
                        observed = observer.observe(proc.pid, context)
                        # Reject non-JSON results, errors, or a missing observation.
                        json.dumps(observed, allow_nan=False)
                        if not isinstance(observed, dict) or observed.get('status') == 'fail':
                            raise RuntimeError('Reference debug observer failed at '+action['label']+': '+str(observed))
                        report['debugObservations'].append({'context': context, 'observation': observed})
                    geometry = subprocess.check_output(['xdotool', 'getwindowgeometry', '--shell', window], text=True)
                    values = dict(line.split('=', 1) for line in geometry.splitlines() if '=' in line)
                    path = out / 'frames' / f"{action['label']}.png"
                    subprocess.run(['ffmpeg', '-hide_banner', '-loglevel', 'error', '-y', '-f', 'x11grab', '-draw_mouse', '0',
                                    '-video_size', f"{values['WIDTH']}x{values['HEIGHT']}", '-i',
                                    f"{os.environ['DISPLAY']}+{values['X']},{values['Y']}", '-frames:v', '1', str(path)], check=True, timeout=15)
                    frames.append({'label': action['label'], 'path': str(path), 'width': int(values['WIDTH']), 'height': int(values['HEIGHT']),
                                   'presentedFrame': current['presentedFrame'], 'virtualElapsedNs': current['virtualElapsedNs'],
                                   'simulationTick': None, 'gateBlocked': True, 'x11Focus': x11_focus(), 'targetWindow': int(window)})
            report['observedState'] = debug.observed_state((local / 'log.txt').read_text()) if (local / 'log.txt').exists() else {}
            if report['observedState'].get('seedValue') != vector['seedValue']:
                raise RuntimeError('Original did not read back the requested seed; deterministic time/input may be incomplete')
            report['seedReadbackMatchesCodec'] = True
            if args.gl_trace:
                report['graphicsTrace'] = {'path': str(trace / 'commands.jsonl'), 'continuous': args.trace_every_frame, 'scope': 'public graphics submitted data; hidden state unknown'}
            report['performance'] = debug.process_performance(proc.pid, started)
            measured = [p for p in acknowledgements if 'advanceHostWallMs' in p]
            report['performance']['controlledAdvances'] = {'presentedFrames': sum(p['requestedPresentationFrames'] for p in measured),
                'hostWallMs': sum(p['advanceHostWallMs'] for p in measured),
                'nativeCpuMs': sum(p['advancePerformance']['cpuUserMs']+p['advancePerformance']['cpuSystemMs'] for p in measured),
                'scope': 'released presentation intervals only; simulation ticks unknown; observer cost included'}
            report['status'] = 'pass'
            connection.sendall(b'quit\n')
            proc.wait(timeout=5)
            reader.close()
            connection.close()
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
        if (local / 'log.txt').exists():
            (out / 'original.log').write_text((local / 'log.txt').read_text())
        report['controllerElapsedMs'] = (time.monotonic() - started) * 1000
        (out / 'manifest.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({'status': report['status'], 'manifest': str(out / 'manifest.json'), 'failures': report['failures']}))
    return 0 if report['status'] == 'pass' else 1

if __name__ == '__main__':
    raise SystemExit(main())
