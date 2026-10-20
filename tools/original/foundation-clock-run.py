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
import struct
import subprocess
import tempfile
import time
import zlib

ROOT = Path(__file__).resolve().parents[2]

def module(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result

codec = module('foundation_seeds', 'foundation-seeds.py')
debug = module('debug_run', 'debug-run.py')
evidence = module('native_evidence', 'native-evidence.py')


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
    host_hold_seconds = 0
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
        elif kind == 'audioHold':
            seconds = action.get('seconds')
            if type(seconds) not in (int, float) or not 0 < seconds <= 30:
                raise ValueError('audioHold requires 0..30 explicit real host seconds')
            host_hold_seconds += seconds
        else:
            raise ValueError(f'Unsupported tape action: {kind}')
    # A whole tape may present twenty minutes at 60 Hz (a multi-floor descent);
    # each step and the command list keep their own 36000 bounds.
    if total > 72000:
        raise ValueError('Tape has more than 72000 presented frames')
    if host_hold_seconds > 120:
        raise ValueError('Tape has more than 120 explicit real audio-device hold seconds')


def checked_observation(observer, pid, context):
    """Observers may explicitly skip checkpoints outside their typed scope."""
    observed = observer.observe(pid, context)
    json.dumps(observed, allow_nan=False)
    if not isinstance(observed, dict) or observed.get('status') not in ('pass', 'skip'):
        raise RuntimeError('Reference debug observer failed at '+context['label']+': '+str(observed))
    return observed


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--seed', help='Valid seed; omitted means new OS-random seed')
    parser.add_argument('--ordinary-run', action='store_true', help='Observe a generated Character seed through an explicit no-dialog tape; no requested run seed')
    parser.add_argument('--challenge-run', action='store_true', help='Explicit Challenge menu tape; observes the generated run seed without manual seed entry')
    parser.add_argument('--boot-epoch-seed', type=int, default=1700000000, help='Public realtime seconds supplied to native time(NULL); independent of the requested run seed')
    parser.add_argument('--tape', type=Path, help='List of key/step/capture/audioHold commands; public X11 keys and explicit real audio-device holds')
    parser.add_argument('--focus-handoff', action='append', default=[], help='At named capture, generate public X11 focus out/in and release one presentation before capture; diagnostic tape alteration is recorded')
    parser.add_argument('--state-observer', type=Path, help='Optional parent-only reference debug module exposing observe(pid, context); never production game input')
    parser.add_argument('--observe-every-step', action='store_true', help='Observe at each tape step gate without screenshots; use one-frame steps for dense trajectories')
    parser.add_argument('--composition-lifecycle-events', action='store_true', help='Optional source-bound leader-thread Knife8/0/1 hardware event observation; requires canonical ISAAC_ORIGINAL_SHA, no game inputs changed')
    parser.add_argument('--trinket-start-events', action='store_true', help='Owned leader-thread Give41/Remove41 hardware entry/return observation; no actor/instruction writes')
    parser.add_argument('--archive-evidence', action='store_true', help='Lossless gzip manifest plus committed native progress journal; public observations and clock unchanged')
    parser.add_argument('--journal-codec', choices=('gzip', 'zstd'), default='gzip', help='Opt-in journal transport only; zstd requires Python zstandard/system libzstd and Node24 readers; default gzip unchanged')
    parser.add_argument('--manifest-codec', choices=('gzip', 'zstd'), default='gzip', help='Opt-in complete manifest transport; requires archived evidence and qualified Zstd readers; default gzip unchanged')
    parser.add_argument('--gl-trace', type=Path, help='Optional documented graphics API observer to chain; no game instructions/state')
    parser.add_argument('--audio-trace', type=Path, help='Optional forwarding OpenAL API observer to chain; no game instructions/state')
    parser.add_argument('--audio-sfx-owners', action='store_true', help='Opt-in actual configured-SFX table owner at forwarded backend starts; requires --audio-trace, preserves ambiguous aliases')
    parser.add_argument('--audio-options-owners', action='store_true', help='Opt-in actual Options237 request/query/gain/reset owners; requires --audio-trace and --audio-sfx-owners')
    parser.add_argument('--mt-trace', action='store_true', help='Chain the shared-MT draw observer; writes OUTPUT/mt-trace/trace-<pid>.bin')
    parser.add_argument('--sfx-volume', type=float, default=0, help='Owned public SFXVolume option, 0..1; admission still occurs when muted')
    parser.add_argument('--music-volume', type=float, default=0, help='Owned public MusicVolume option, 0..1')
    parser.add_argument('--trace-every-frame', action='store_true', help='Request graphics observer continuous draw/presentation tracing')
    parser.add_argument('--render-all', action='store_true', help='Draw and present every released frame (gate ISAAC_CLOCK_GATE_RENDER_ALL=1); implied by --gl-trace')
    parser.add_argument('--width', type=int, default=960, help='Original supported window width; larger values aid public subpixel diagnostics')
    parser.add_argument('--height', type=int, default=540, help='Original supported window height')
    parser.add_argument('--max-render-scale', type=int, default=2, help='Original supported MaxRenderScale option; normal comparisons use 2')
    parser.add_argument('--fps', type=int, default=60, help='Nominal presentation rate; not asserted native simulation Hz')
    parser.add_argument('--step-ns', type=int, default=16667000, help='Explicit virtual interval; default is 16667 microseconds, above native timer polling threshold')
    parser.add_argument('--output', type=Path, default=ROOT / '.cache/foundation' / f'clock-native-{time.time_ns()}')
    parser.add_argument('--frame-timeout', type=float, default=60,
                        help='Host-wall socket/gate wait limit in seconds, per 600 frames of a step; does not alter released presentation counts')
    args = parser.parse_args()
    if args.trinket_start_events and args.composition_lifecycle_events:
        parser.error('One exclusive hardware event owner may trace the Native leader')
    event_report_field = 'trinketStartEvents' if args.trinket_start_events else 'compositionLifecycle'
    if args.journal_codec != 'gzip' and not args.archive_evidence:
        raise SystemExit('--journal-codec requires --archive-evidence')
    if args.manifest_codec != 'gzip':
        if not args.archive_evidence:
            raise SystemExit('--manifest-codec requires --archive-evidence')
        evidence.require_zstd()
    if not 0 <= args.boot_epoch_seed <= 0xffffffff:
        raise SystemExit('--boot-epoch-seed must be a u32')
    if args.observe_every_step and not args.state_observer:
        raise SystemExit('--observe-every-step requires --state-observer')
    if not 0 <= args.sfx_volume <= 1 or not 0 <= args.music_volume <= 1:
        raise SystemExit('Owned public audio volumes must be 0..1')
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
    # Window captures read the X framebuffer in process: the same pixels the
    # former per-capture ffmpeg x11grab subprocess wrote, without ~0.3 s of
    # process start-up per capture.
    class XImage(ctypes.Structure):
        _fields_ = [('width', ctypes.c_int), ('height', ctypes.c_int), ('xoffset', ctypes.c_int), ('format', ctypes.c_int),
                    ('data', ctypes.POINTER(ctypes.c_ubyte)), ('byte_order', ctypes.c_int), ('bitmap_unit', ctypes.c_int),
                    ('bitmap_bit_order', ctypes.c_int), ('bitmap_pad', ctypes.c_int), ('depth', ctypes.c_int),
                    ('bytes_per_line', ctypes.c_int), ('bits_per_pixel', ctypes.c_int),
                    ('red_mask', ctypes.c_ulong), ('green_mask', ctypes.c_ulong), ('blue_mask', ctypes.c_ulong)]
    x11.XDefaultRootWindow.argtypes = [ctypes.c_void_p]
    x11.XDefaultRootWindow.restype = ctypes.c_ulong
    x11.XGetImage.argtypes = [ctypes.c_void_p, ctypes.c_ulong, ctypes.c_int, ctypes.c_int, ctypes.c_uint, ctypes.c_uint, ctypes.c_ulong, ctypes.c_int]
    x11.XGetImage.restype = ctypes.POINTER(XImage)
    x11.XGetWindowAttributes.argtypes = [ctypes.c_void_p, ctypes.c_ulong, ctypes.c_void_p]
    x11.XTranslateCoordinates.argtypes = [ctypes.c_void_p, ctypes.c_ulong, ctypes.c_ulong, ctypes.c_int, ctypes.c_int,
                                          ctypes.POINTER(ctypes.c_int), ctypes.POINTER(ctypes.c_int), ctypes.POINTER(ctypes.c_ulong)]
    x11.XFree.argtypes = [ctypes.c_void_p]

    def x11_grab(window, path):
        display = x11.XOpenDisplay(None)
        if not display:
            raise RuntimeError('X11 capture could not open display')
        try:
            # XWindowAttributes begins with x, y, width, height (ints).
            attributes = (ctypes.c_int * 64)()
            if not x11.XGetWindowAttributes(display, int(window), attributes):
                raise RuntimeError('X11 capture could not read window geometry')
            width, height = attributes[2], attributes[3]
            if width <= 0 or height <= 0:
                raise RuntimeError('X11 capture requires positive window dimensions')
            root = x11.XDefaultRootWindow(display)
            x, y, child = ctypes.c_int(), ctypes.c_int(), ctypes.c_ulong()
            if not x11.XTranslateCoordinates(display, int(window), root, 0, 0, ctypes.byref(x), ctypes.byref(y), ctypes.byref(child)):
                raise RuntimeError('X11 capture could not translate the window origin')
            image = x11.XGetImage(display, root, x.value, y.value, width, height, 0xFFFFFFFF, 2)
            if not image:
                raise RuntimeError('X11 capture XGetImage failed')
            try:
                frame = image.contents
                if (frame.width, frame.height) != (width, height) or frame.xoffset != 0:
                    raise RuntimeError('X11 capture image geometry differs from the requested window')
                if frame.bytes_per_line < width * 4 or not frame.data:
                    raise RuntimeError('X11 capture image has an invalid pixel buffer or row stride')
                if frame.bits_per_pixel != 32 or frame.byte_order != 0 or (frame.red_mask, frame.green_mask, frame.blue_mask) != (0xff0000, 0xff00, 0xff):
                    raise RuntimeError('X11 capture expects a little-endian 32-bit BGRX root visual')
                stride = frame.bytes_per_line
                raw = ctypes.string_at(frame.data, stride * height)
            finally:
                x11.XFree(image.contents.data)
                x11.XFree(image)
        finally:
            x11.XCloseDisplay(display)
        rows = []
        for row in range(height):
            line = raw[row * stride:row * stride + width * 4]
            rgb = bytearray(width * 3)
            rgb[0::3], rgb[1::3], rgb[2::3] = line[2::4], line[1::4], line[0::4]
            rows.append(b'\0' + bytes(rgb))
        chunk = lambda kind, data: struct.pack('>I', len(data)) + kind + data + struct.pack('>I', zlib.crc32(kind + data))
        path.write_bytes(b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', width, height, 8, 2, 0, 0, 0))
                         + chunk(b'IDAT', zlib.compress(b''.join(rows), 6)) + chunk(b'IEND', b''))
        return width, height

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
    if args.ordinary_run and (args.seed or not args.tape):
        raise SystemExit('--ordinary-run requires an explicit public tape and excludes --seed')
    if args.challenge_run and (args.ordinary_run or args.seed or not args.tape):
        raise SystemExit('--challenge-run requires an explicit public tape and excludes --seed/--ordinary-run')
    generated_run = args.ordinary_run or args.challenge_run
    vector = {'seedValue': None} if generated_run else ({'code': args.seed, 'seedValue': codec.decode_seed(args.seed)} if args.seed else codec.random_vector())
    seed = None if generated_run else codec.encode_seed(vector['seedValue'])
    tape = json.loads(args.tape.read_text()) if args.tape else default_tape(seed)
    validate_tape(tape)
    if args.ordinary_run and any(a['type'] == 'key' and a['key'] != 'Return' for a in tape):
        raise SystemExit('--ordinary-run admission tape permits Return confirmations only, without a seed dialog')
    if args.challenge_run and any(a['type'] == 'key' and a['key'] == 'Tab' for a in tape):
        raise SystemExit('--challenge-run enters the source-selected Challenge seed without a seed dialog')
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
    declared_options = {'Fullscreen': 0, 'Filter': 0, 'VSync': 0, 'PauseOnFocusLost': 0, 'SteamCloud': 1, 'MouseControl': 0,
                        'WindowWidth': args.width, 'WindowHeight': args.height, 'WindowPosX': 0, 'WindowPosY': 0,
                        'MaxRenderScale': args.max_render_scale, 'MusicVolume': args.music_volume, 'SFXVolume': args.sfx_volume}
    (local / 'options.ini').write_text('[Options]\n' + ''.join(f'{key}={value}\n' for key, value in declared_options.items()))
    gate = out / 'clock-gate.so'
    subprocess.run(['cc', '-std=c11', '-Wall', '-Wextra', '-Werror', '-fPIC', '-shared',
                    str(Path(__file__).with_name('foundation-clock-gate.c')), '-o', str(gate), '-ldl'], check=True)
    # The original silently ignores options.ini (falling back to defaults for
    # volumes, VSync and focus pause) when the profile path is long. Hand it a
    # short symlink to the retained profile so the declared options apply.
    short = Path(tempfile.mkdtemp(prefix='isaac-')) / 'p'
    short.symlink_to(out / 'profile', target_is_directory=True)
    env = os.environ.copy()
    env.update({'HOME': str(short), 'XDG_DATA_HOME': str(short / 'data'),
                'XDG_CONFIG_HOME': str(short / 'config'), 'ISAAC_OFFLINE_CLOUD_DIR': str(short / 'cloud'),
                'ALSOFT_DRIVERS': 'null', 'LD_PRELOAD': str(gate) + (' ' + env['LD_PRELOAD'] if env.get('LD_PRELOAD') else ''),
                'LD_LIBRARY_PATH': str(source / 'lib64') + (':' + env['LD_LIBRARY_PATH'] if env.get('LD_LIBRARY_PATH') else ''),
                'ISAAC_CLOCK_GATE_FPS': str(args.fps), 'ISAAC_CLOCK_GATE_STEP_NS': str(args.step_ns),
                'ISAAC_CLOCK_GATE_EPOCH_SEED': str(args.boot_epoch_seed)})
    # Each observer owns its own directory; the graphics one is reported after
    # the run, so it keeps its own name rather than a `trace` the audio and MT
    # observers rebind.
    graphics_trace = out / 'gl-trace'
    if args.gl_trace:
        graphics_trace.mkdir(exist_ok=True)
        env['LD_PRELOAD'] = str(gate) + ' ' + str(args.gl_trace.resolve()) + (' ' + os.environ['LD_PRELOAD'] if os.environ.get('LD_PRELOAD') else '')
        env['ISAAC_GL_TRACE_DIR'] = str(graphics_trace)
        env['ISAAC_GL_TRACE_CONTROL'] = str(graphics_trace / 'request')
        # The graphics observer records submitted draws, so every released
        # frame must still draw them (see foundation-clock-gate.c).
        env['ISAAC_CLOCK_GATE_RENDER_ALL'] = '1'
        env.pop('ISAAC_GL_TRACE_STOP_AFTER_FRAME', None)
        if args.trace_every_frame:
            env['ISAAC_GL_TRACE_EVERY_FRAME'] = '1'
    elif args.trace_every_frame:
        raise SystemExit('--trace-every-frame requires --gl-trace')
    if args.render_all:
        env['ISAAC_CLOCK_GATE_RENDER_ALL'] = '1'
    if args.audio_trace:
        trace = out / 'audio-trace'
        trace.mkdir(exist_ok=True)
        env['LD_PRELOAD'] += ' ' + str(args.audio_trace.resolve())
        env['ISAAC_AUDIO_TRACE_DIR'] = str(trace)
        if args.audio_sfx_owners:
            env['ISAAC_AUDIO_SFX_OWNERS'] = '1'
        else:
            env.pop('ISAAC_AUDIO_SFX_OWNERS', None)
        if args.audio_options_owners:
            if not args.audio_sfx_owners:
                raise SystemExit('--audio-options-owners requires --audio-sfx-owners')
            env['ISAAC_AUDIO_OPTIONS_OWNERS'] = '1'
        else:
            env.pop('ISAAC_AUDIO_OPTIONS_OWNERS', None)
    elif args.audio_sfx_owners or args.audio_options_owners:
        raise SystemExit('Audio owners require --audio-trace')
    if args.mt_trace:
        trace = out / 'mt-trace'
        trace.mkdir(exist_ok=True)
        library = out / 'mt-trace.so'
        subprocess.run(['cc', '-std=c11', '-Wall', '-Wextra', '-Werror', '-fPIC', '-shared',
                        str(Path(__file__).with_name('foundation-mt-trace.c')), '-o', str(library), '-ldl'], check=True)
        env['LD_PRELOAD'] += ' ' + str(library)
        env['ISAAC_MT_TRACE_DIR'] = str(trace)
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
              'seedMode': 'source-challenge-menu' if args.challenge_run else 'ordinary-character-menu' if args.ordinary_run else 'public-manual-entry',
              'bootEpochSeed': args.boot_epoch_seed,
              'requestedViewport': {'width': args.width, 'height': args.height, 'maxRenderScale': args.max_render_scale},
              'profile': {'fresh': True, 'isolated': True}, 'tape': tape, 'frames': frames,
              'audioOptions': {'sfxVolume': args.sfx_volume, 'musicVolume': args.music_volume},
              **({'optionsAudioOwnerObserver': 1} if args.audio_options_owners else {}),
              'inputs': events, 'presentations': acknowledgements,
              'clock': {'scope': 'main-thread wall-clock ABI; worker/CPU clocks unchanged', 'nominalStepsPerSecond': args.fps, 'stepNs': args.step_ns,
                        'mainThreadSleeps': 'recorded then returned without host delay', 'realPerformance': 'controller wall clock'},
              'inputDelivery': {'api': 'public X11 key events', 'serverAutoRepeat': False},
              'hostExecution': {'frameTimeoutSeconds': args.frame_timeout, 'lpNumThreads': env.get('LP_NUM_THREADS'),
                                'releasedPresentations': 'drawn' if env.get('ISAAC_CLOCK_GATE_RENDER_ALL', '0') not in ('', '0') else 'default framebuffer dropped',
                                'windowAttributeReplies': 'forwarded' if env.get('ISAAC_CLOCK_GATE_WINDOW_QUERIES', '0') not in ('', '0') else 'cached per frame',
                                'scope': 'Host-wall wait limit and requested software-renderer pool; public tape/virtual steps unchanged'},
              'simulationTicks': False, 'fullInternalState': False, 'failures': []}
    if observer:
        report['debugObserver'] = {'module': str(args.state_observer.resolve()), 'everyTapeStep': args.observe_every_step, 'scope': 'reference-only parent debug observations at blocked presentation checkpoints; no hidden-state parity claim'}
        report['debugObservations'] = []
    (out / 'tape.json').write_text(json.dumps(tape, indent=2) + '\n')
    progress = evidence.make_progress(out, args.journal_codec) if args.archive_evidence else None
    def record(kind, value):
        if progress:
            progress.append(kind, value)
    record('start', report)
    if progress and observer:
        # The committed journal already owns every complete observation.
        # Keep its final array lazy instead of retaining all decoded snapshots.
        report['debugObservations'] = evidence.JournalValues(progress, 'debugObservation')
    proc = None
    lifecycle = None
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
                record('acknowledgement', status)
                if lifecycle is not None:
                    lifecycle.gate(status)
                return status
            current = acknowledgement()
            if args.composition_lifecycle_events or args.trinket_start_events:
                event_module = 'trinket-start-events.py' if args.trinket_start_events else 'composition-lifecycle.py'
                event_directory = 'trinket-start-events' if args.trinket_start_events else 'lifecycle'
                lifecycle = module('owned_source_events', event_module).Tap(proc.pid, out / event_directory, timeout=args.frame_timeout)
                lifecycle.gate(current)
            connection.sendall(b'step 30\n')
            current = acknowledgement()
            report['bootstrapPresentedFrames'] = 30
            record('field', {'name': 'bootstrapPresentedFrames', 'value': 30})
            # A discarded options.ini is rewritten with defaults at boot; never
            # let the original silently run on options other than the declared ones.
            kept = dict(line.split('=', 1) for line in (local / 'options.ini').read_text().splitlines() if '=' in line)
            differing = {key: kept.get(key) for key, value in declared_options.items()
                         if key in ('VSync', 'PauseOnFocusLost', 'WindowWidth', 'WindowHeight', 'MusicVolume', 'SFXVolume')
                         and (kept.get(key) is None or float(kept[key]) != float(value))}
            if differing:
                raise RuntimeError(f'Original discarded the declared options.ini after boot: {differing}')
            report['declaredOptionsKept'] = True
            record('field', {'name': 'declaredOptionsKept', 'value': True})
            windows = subprocess.check_output(['xdotool', 'search', '--onlyvisible', '--name', '(Isaac|Rebirth)'], text=True)
            window = windows.splitlines()[-1]
            subprocess.run(['xdotool', 'windowfocus', '--sync', window], check=True)
            def observe_current(label):
                current_log = debug.observed_state((local / 'log.txt').read_text()) if (local / 'log.txt').exists() else {}
                context = {'label': label, 'seed': seed, 'seedValue': vector['seedValue'],
                    'presentedFrame': current['presentedFrame'], 'virtualElapsedNs': current['virtualElapsedNs'],
                    'output': str(out), 'publicLogState': current_log, 'gateBlocked': True,
                    'scope': 'reference debug data only; simulation ticks and full internal state not established'}
                observed = checked_observation(observer, proc.pid, context)
                observation = {'context': context, 'observation': observed}
                if not progress:
                    report['debugObservations'].append(observation)
                record('debugObservation', observation)
            if args.observe_every_step:
                observe_current('bootstrap-'+str(current['presentedFrame']))
            for action in tape:
                kind = action['type']
                if kind == 'step':
                    before = current['presentedFrame']
                    cpu_before = debug.process_performance(proc.pid, started)
                    step_started = time.monotonic()
                    # A long step is one acknowledgement; allow the limit per 600 frames.
                    connection.settimeout(args.frame_timeout * -(-action['frames'] // 600))
                    connection.sendall(f"step {action['frames']}\n".encode())
                    current = acknowledgement()
                    connection.settimeout(args.frame_timeout)
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
                    record('advanceComplete', current)
                    if args.observe_every_step:
                        observe_current('step-'+str(current['presentedFrame']))
                elif kind == 'key':
                    keysym = x11.XStringToKeysym(action['key'].encode('ascii'))
                    if not keysym:
                        raise RuntimeError('Unknown public X11 keysym: '+action['key'])
                    subprocess.run(['xdotool', 'keydown' if action['down'] else 'keyup', '--window', window, action['key']], check=True)
                    subprocess.run(['xdotool', 'getwindowfocus'], check=True, stdout=subprocess.DEVNULL)
                    events.append({**action, 'keysym': keysym, 'presentedFrame': current['presentedFrame'], 'virtualElapsedNs': current['virtualElapsedNs']})
                    record('input', events[-1])
                elif kind == 'audioHold':
                    hold_started = time.monotonic()
                    time.sleep(action['seconds'])
                    report.setdefault('hostAudioHolds', []).append({
                        'requestedSeconds': action['seconds'], 'elapsedSeconds': time.monotonic() - hold_started,
                        'presentedFrame': current['presentedFrame'], 'virtualElapsedNs': current['virtualElapsedNs'],
                        'mainThreadGateBlocked': True, 'scope': 'Real worker/audio-device time; no presentations or synthetic completion events',
                    })
                    record('audioHold', report['hostAudioHolds'][-1])
                else:
                    focus_before = x11_focus()
                    if action['label'] in args.focus_handoff:
                        focus_after = x11_focus(window)
                        connection.sendall(b'step 1\n')
                        current = acknowledgement()
                        report.setdefault('focusHandoffs', []).append({'label': action['label'], 'before': focus_before, 'after': focus_after, 'releasedPresentations': 1, 'presentedFrame': current['presentedFrame']})
                        record('focusHandoff', report['focusHandoffs'][-1])
                    if action['label'] == 'gameplay-start':
                        report['initialState'] = debug.observed_state((local / 'log.txt').read_text())
                        record('field', {'name': 'initialState', 'value': report['initialState']})
                    if observer:
                        observe_current(action['label'])
                    path = out / 'frames' / f"{action['label']}.png"
                    width, height = x11_grab(window, path)
                    frames.append({'label': action['label'], 'path': str(path), 'width': width, 'height': height,
                                   'presentedFrame': current['presentedFrame'], 'virtualElapsedNs': current['virtualElapsedNs'],
                                   'simulationTick': None, 'gateBlocked': True, 'x11Focus': x11_focus(), 'targetWindow': int(window)})
                    record('capture', frames[-1])
            report['observedState'] = debug.observed_state((local / 'log.txt').read_text()) if (local / 'log.txt').exists() else {}
            if generated_run:
                observed = report['observedState']
                if codec.decode_seed(observed.get('seed')) != observed.get('seedValue'):
                    raise RuntimeError('Original generated seed code/value did not agree through the public codec')
                report['seed'] = codec.encode_seed(observed['seedValue'])
                report['seedValue'] = observed['seedValue']
            elif report['observedState'].get('seedValue') != vector['seedValue']:
                raise RuntimeError('Original did not read back the requested seed; deterministic time/input may be incomplete')
            report['seedReadbackMatchesCodec'] = True
            if args.gl_trace:
                report['graphicsTrace'] = {'path': str(graphics_trace / 'commands.jsonl'), 'continuous': args.trace_every_frame, 'scope': 'public graphics submitted data; hidden state unknown'}
            report['performance'] = debug.process_performance(proc.pid, started)
            measured = [p for p in acknowledgements if 'advanceHostWallMs' in p]
            report['performance']['controlledAdvances'] = {'presentedFrames': sum(p['requestedPresentationFrames'] for p in measured),
                'hostWallMs': sum(p['advanceHostWallMs'] for p in measured),
                'nativeCpuMs': sum(p['advancePerformance']['cpuUserMs']+p['advancePerformance']['cpuSystemMs'] for p in measured),
                'scope': 'released presentation intervals only; simulation ticks unknown; observer cost included'}
            if lifecycle is not None:
                report[event_report_field] = lifecycle.close()
            report['status'] = 'pass'
            connection.sendall(b'quit\n')
            proc.wait(timeout=5)
            reader.close()
            connection.close()
    except Exception as error:
        report['status'] = 'fail'
        report['failures'].append(str(error))
    finally:
        if lifecycle is not None:
            try:
                report[event_report_field] = lifecycle.close()
            except Exception as error:
                report['status'] = 'fail'
                report['failures'].append(str(error))
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
        manifest = out / ('manifest.json.zst' if args.archive_evidence and args.manifest_codec == 'zstd'
                          else 'manifest.json.gz' if args.archive_evidence else 'manifest.json')
        if progress:
            record('finish', {key: value for key, value in report.items()
                              if key not in ('debugObservations', 'presentations', 'inputs', 'frames', 'tape')})
            progress.close()
            if args.manifest_codec == 'zstd':
                evidence.write_zstd_manifest(manifest, report)
            else:
                evidence.write_manifest(manifest, report)
        else:
            manifest.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({'status': report['status'], 'manifest': str(manifest), 'failures': report['failures']}))
    return 0 if report['status'] == 'pass' else 1

if __name__ == '__main__':
    raise SystemExit(main())
