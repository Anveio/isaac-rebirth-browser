#!/usr/bin/env python3
"""Verify native OpenGL, window discovery, input, and capture (NOT the game)."""
import json
import os
from pathlib import Path
import subprocess
import time

root = Path(__file__).resolve().parents[2]
output = root / '.cache/original-runtime/probe'
output.mkdir(parents=True, exist_ok=True)
if not os.environ.get('DISPLAY'):
    raise SystemExit('Run with tools/original/with-display.sh')
info = subprocess.check_output(['glxinfo', '-B'], text=True)
(output / 'glxinfo.txt').write_text(info)
with (output / 'glxgears.log').open('w') as log:
    proc = subprocess.Popen(['glxgears', '-geometry', '640x480+0+0'], stdout=log, stderr=log)
    try:
        window = None
        for _ in range(100):
            result = subprocess.run(['xdotool', 'search', '--onlyvisible', '--name', '^glxgears$'], capture_output=True, text=True)
            if result.returncode == 0 and result.stdout.strip():
                window = result.stdout.splitlines()[0]
                break
            if proc.poll() is not None:
                raise RuntimeError('OpenGL probe exited before displaying a window')
            time.sleep(.05)
        if window is None:
            raise RuntimeError('No visible OpenGL window')
        subprocess.run(['xdotool', 'windowfocus', '--sync', window], check=True)
        subprocess.run(['ffmpeg', '-hide_banner', '-loglevel', 'error', '-y', '-f', 'x11grab', '-video_size', '640x480', '-i', os.environ['DISPLAY'], '-frames:v', '1', str(output / 'display.png')], check=True, timeout=15)
        subprocess.run(['xdotool', 'key', 'Escape'], check=True)
        code = proc.wait(timeout=5)
        if code != 0:
            raise RuntimeError(f'Probe exited with {code}')
        report = {'nativeDisplay': True, 'openGL': True, 'capture': True, 'keyboardQuit': True, 'originalGameLaunched': False, 'renderer': next((s.split(': ', 1)[1] for s in info.splitlines() if s.startswith('OpenGL renderer string:')), 'unknown')}
        (output / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
        print(json.dumps(report))
    finally:
        if proc.poll() is None:
            proc.terminate()
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                proc.kill()
                proc.wait()
