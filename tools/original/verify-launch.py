#!/usr/bin/env python3
"""Attempt authentic native launch; capture and send Return only after a window exists."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time
root = Path(__file__).resolve().parents[2]
parser = argparse.ArgumentParser()
parser.add_argument('--game-dir', type=Path, default=root / 'references/isaac/linux/depot_250903')
parser.add_argument('--enter-run', action='store_true', help='Navigate default menus with Return; this is a visual smoke test, not a tick replay.')
parser.add_argument('--output', type=Path, default=root / '.cache/original-runtime/game-check')
args = parser.parse_args()
game = args.game_dir.resolve()
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=True)
if not os.environ.get('DISPLAY'):
    raise SystemExit('Run through tools/original/with-display.sh')
report = {'originalGameLaunched': False, 'inputSent': False, 'screenshotsCaptured': False,
          'gameSha256': hashlib.sha256((game / 'isaac.x64').read_bytes()).hexdigest(),
          'exactStateParity': 'not-tested'}
provenance = game / 'provenance.json'
report['runtimeMode'] = 'offline-derived' if provenance.exists() else 'steam-original'
if provenance.exists():
    report['provenance'] = json.loads(provenance.read_text())
    if report['gameSha256'] != report['provenance']['derivedSha256']:
        raise SystemExit('Derived executable does not match its provenance.')
    library_hash = hashlib.sha256((game/'lib64/libsteam_api.so').read_bytes()).hexdigest()
    if library_hash != report['provenance']['derivedSteamApiSha256']:
        raise SystemExit('Offline host adapter does not match its provenance.')
    source_hash = hashlib.sha256((root/'references/isaac/linux/depot_250903/isaac.x64').read_bytes()).hexdigest()
    if source_hash != report['provenance']['sourceSha256']:
        raise SystemExit('Golden original no longer matches its provenance.')
code = 1
started = time.monotonic()
with (out / 'launch.log').open('w') as log:
    proc = subprocess.Popen(['bash', './run-x64.sh'], cwd=game, stdout=log, stderr=log, start_new_session=True)
    try:
        window = None
        for _ in range(200):
            if proc.poll() is not None:
                report['exitCode'] = proc.returncode
                report['blocker'] = 'Original exited before a visible window; inspect launch.log.'
                break
            result = subprocess.run(['xdotool', 'search', '--onlyvisible', '--name', '(Isaac|Rebirth)'], capture_output=True, text=True)
            if result.returncode == 0 and result.stdout.strip():
                window = result.stdout.splitlines()[-1]
                break
            time.sleep(.1)
        if window:
            report['originalGameLaunched'] = True
            time.sleep(3)
            def capture(name):
                subprocess.run(['ffmpeg','-hide_banner','-loglevel','error','-y','-f','x11grab','-draw_mouse','0','-video_size','1280x720','-i',os.environ['DISPLAY'],'-frames:v','1',str(out/name)],check=True,timeout=15)
            capture('before.png')
            subprocess.run(['xdotool','windowfocus','--sync',window],check=True)
            subprocess.run(['xdotool','key','--delay','150','Return'],check=True)
            report['inputSent'] = True
            time.sleep(2)
            capture('after.png')
            if args.enter_run:
                for _ in range(7):
                    subprocess.run(['xdotool','key','--delay','150','Return'],check=True)
                    time.sleep(.8)
                time.sleep(2)
                time.sleep(2)
                capture('gameplay.png')
                subprocess.run(['xdotool','keydown','d'],check=True)
                time.sleep(.5)
                subprocess.run(['xdotool','keyup','d'],check=True)
                capture('movement.png')
                subprocess.run(['xdotool','keydown','Right'],check=True)
                time.sleep(.5)
                capture('shooting.png')
                subprocess.run(['xdotool','keyup','Right'],check=True)
                report['movementAndFireInputsSent'] = True
                report['menuNavigationInputsSent'] = 7
                report['gameplayVisualReviewRequired'] = True
            report['screenshotsCaptured'] = True
            report['visualReviewRequired'] = True
            process_stat = Path(f'/proc/{proc.pid}/stat').read_text().split(')',1)[1].split()
            status = Path(f'/proc/{proc.pid}/status').read_text()
            clock_ticks = os.sysconf('SC_CLK_TCK')
            peak = next(int(line.split()[1])*1024 for line in status.splitlines() if line.startswith('VmHWM:'))
            report['performance'] = {'wallSeconds': time.monotonic()-started, 'cpuUserSeconds': int(process_stat[11])/clock_ticks, 'cpuSystemSeconds': int(process_stat[12])/clock_ticks, 'rssBytes': int(process_stat[21])*os.sysconf('SC_PAGE_SIZE'), 'peakRssBytes': peak, 'scope': 'native process lifetime; includes boot and capture waits; excludes helper processes', 'perTickTiming': 'unavailable'}
            code = 0
        elif 'blocker' not in report:
            report['blocker'] = 'Timed out waiting for a visible game window.'
    finally:
        if proc.poll() is None:
            import signal
            os.killpg(proc.pid, signal.SIGTERM)
            try: proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                os.killpg(proc.pid, signal.SIGKILL)
                proc.wait()
        (out / 'result.json').write_text(json.dumps(report,indent=2)+'\n')
        print(json.dumps(report))
raise SystemExit(code)
