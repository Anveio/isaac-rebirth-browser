// Side-by-side playtest footage: the fresh original and the production build
// replay one public-input tape (seed, keys, ticks) and each frame pair is
// stacked into one video, original on the left, port on the right.
//
// Any recorded paired case is a state the video can show: a run seed fixes the
// level generation, the tape's keys walk the route (to a boss room, a treasure
// pedestal, a trapdoor), and a scenario helper finds such routes on demand.
// The tool only adds capture commands to the tape, so neither game sees any
// input the case did not already declare.
//
//   node tools/playtest/side-by-side.mjs --case CASE.json --out DIR
//   node tools/playtest/side-by-side.mjs --helper tools/original/entities-scenarios.py \
//     --helper-arg --only --helper-arg dingle --pick G66GMA3N --out DIR
//   node tools/playtest/side-by-side.mjs --reel DIR/a/side-by-side.mp4,DIR/b/side-by-side.mp4 --out DIR
//
// Footage is a viewing aid, not parity evidence: it records no comparison.
import assert from 'node:assert/strict';
import { execFile, execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { basename, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { promisify } from 'node:util';
import { chromium } from '@playwright/test';
import { browserLaunchOptions } from '../foundation/browser.mjs';
import { startProductionServer } from '../foundation/production-server.mjs';

const ROOT = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const run = promisify(execFile);
const args = process.argv.slice(2);
const flag = (name, fallback) => { const at = args.indexOf(name); return at < 0 ? fallback : args[at + 1]; };
const repeated = name => args.flatMap((arg, index) => arg === name ? [args[index + 1]] : []);
const FONT = '/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf';

const out = resolve(flag('--out', join(ROOT, '.cache/playtest/side-by-side', String(Date.now()))));
const every = Number(flag('--every', '2'));
const extra = Number(flag('--extra', '0'));
const startLabel = flag('--from');
const keepFrames = args.includes('--keep-frames');
assert.ok(Number.isInteger(every) && every >= 1 && every <= 60, '--every must be 1..60 presentations');
mkdirSync(out, { recursive: true });

const sha = (() => { try { return execFileSync('git', ['rev-parse', '--short', 'HEAD'], { cwd: ROOT, encoding: 'utf8' }).trim(); } catch { return 'local'; } })();

// A case file holds one case (pairedTape at top level) or a scenarios file
// with cases[]; `path#id` or --pick selects one by scenario id.
function loadCases() {
  const picks = repeated('--pick');
  const cases = [];
  const select = (list, label) => {
    const chosen = picks.length ? list.filter(item => picks.includes(item.scenario?.id)) : list;
    assert.ok(chosen.length, `No case in ${label} matches ${picks.join(', ') || '(any)'}`);
    cases.push(...chosen);
  };
  for (const spec of repeated('--case')) {
    const [path, id] = spec.split('#');
    const data = JSON.parse(readFileSync(resolve(path), 'utf8'));
    const list = data.pairedTape ? [data] : data.cases;
    select(id ? list.filter(item => item.scenario?.id === id) : list, path);
  }
  const helper = flag('--helper');
  if (helper) {
    const data = JSON.parse(execFileSync('python3', [helper, ...repeated('--helper-arg')],
      { cwd: ROOT, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 }));
    writeFileSync(join(out, 'helper-scenarios.json'), JSON.stringify(data) + '\n');
    for (const failure of data.failures ?? []) console.error(`helper failure: ${JSON.stringify(failure)}`);
    select(data.cases, helper);
  }
  return cases;
}

// Splits the browser tape's steps so a `sbs-NNNNNN` capture follows every
// `every` presentations from the start label on; the original tape is the
// source-only prelude plus the same commands (see paired-game.mjs).
function densify(caseData) {
  const tape = [];
  let recording = !startLabel, presentation = 0, index = 0;
  const capture = () => tape.push({ type: 'capture', label: `sbs-${String(index++).padStart(6, '0')}` });
  const steps = frames => {
    while (frames > 0) {
      const chunk = recording ? Math.min(frames, every - (presentation % every)) : frames;
      tape.push({ type: 'step', frames: chunk });
      presentation += chunk; frames -= chunk;
      if (recording && presentation % every === 0) capture();
    }
  };
  if (recording) capture();
  for (const action of caseData.pairedTape) {
    if (action.type === 'step') steps(action.frames);
    else {
      tape.push(action);
      if (action.type === 'capture' && action.label === startLabel && !recording) { recording = true; capture(); }
    }
  }
  steps(extra);
  assert.ok(index > 1, `Start label ${startLabel} never appears in the tape`);
  const prelude = caseData.bootstrap?.sourceOnlyPrelude ?? [];
  return { browser: tape, native: [...prelude, ...tape], frames: index };
}

async function nativeFrames(caseData, tape, directory) {
  mkdirSync(directory, { recursive: true });
  const tapePath = join(directory, 'tape.json');
  writeFileSync(tapePath, JSON.stringify(tape));
  const command = ['tools/original/with-display.sh', 'python3', 'tools/original/foundation-clock-run.py',
    '--seed', caseData.seed, '--tape', tapePath, '--frame-timeout', flag('--native-frame-timeout', '120'),
    '--boot-epoch-seed', String(caseData.bootEpochSeed ?? 1700000000), '--output', directory];
  try {
    const result = await run('bash', command, { cwd: ROOT, maxBuffer: 64 * 1024 * 1024 });
    writeFileSync(join(directory, 'controller-output.txt'), result.stdout + result.stderr);
    return { status: 'pass' };
  } catch (failure) {
    writeFileSync(join(directory, 'controller-output.txt'), `${failure.stdout || ''}${failure.stderr || ''}${failure.message}`);
    return { status: 'fail', error: String(failure.message).split('\n')[0] };
  }
}

const namedKeys = { Return: 'Enter', Escape: 'Escape', Tab: 'Tab', BackSpace: 'Backspace', space: 'Space',
  Up: 'ArrowUp', Down: 'ArrowDown', Left: 'ArrowLeft', Right: 'ArrowRight' };

async function browserFrames(browser, server, caseData, tape, directory) {
  mkdirSync(directory, { recursive: true });
  const context = await browser.newContext({ viewport: { width: 960, height: 540 } });
  const page = await context.newPage();
  const errors = [];
  page.on('pageerror', error => errors.push(String(error)));
  try {
    await page.goto(server.url + '/?manual=1');
    await page.waitForFunction(() => ['true', 'failed'].includes(document.body.dataset.ready));
    assert.equal(await page.evaluate(() => document.body.dataset.ready), 'true', 'Production browser must boot');
    await page.evaluate(({ initialSeed, bootEpochSeed }) => { window.game.pause(); window.game.reset(initialSeed, { bootEpochSeed }); }, caseData);
    for (const action of tape) {
      if (action.type === 'step') await page.evaluate(count => { window.game.step(count); }, action.frames);
      else if (action.type === 'key') await page.keyboard[action.down ? 'down' : 'up'](namedKeys[action.key] || action.key);
      else if (action.label.startsWith('sbs-')) {
        const png = await page.evaluate(() => window.game.capture().png);
        writeFileSync(join(directory, action.label + '.png'), Buffer.from(png, 'base64'));
      }
    }
    return { status: errors.length ? 'fail' : 'pass', errors };
  } catch (error) {
    return { status: 'fail', errors: [...errors, String(error.message)] };
  } finally { await context.close(); }
}

const text = value => value.replace(/[\\:']/g, ' ');
const banner = (label, x) => `drawtext=fontfile=${FONT}:text='${text(label)}':x=${x}:y=12:fontsize=26:fontcolor=white`;

// Original | port at real-time speed with a caption bar; frames missing on one
// side (a crash or early exit) repeat that side's last frame.
async function compose(caseData, directory, frames, title) {
  const fps = 60 / every;
  const video = join(directory, 'side-by-side.mp4');
  const left = join(directory, 'original', 'frames', 'sbs-%06d.png');
  const right = join(directory, 'port', 'sbs-%06d.png');
  const caption = `${title}  seed ${caseData.seed}`;
  const filter = [
    `[0:v]scale=960:540,tpad=stop_mode=clone:stop=-1[l]`, `[1:v]scale=960:540,tpad=stop_mode=clone:stop=-1[r]`,
    `[l][r]hstack=inputs=2,pad=1920:600:0:60:color=0x111111,${banner('Original Rebirth', 20)},${banner(`Browser port ${sha}`, 980)},`
      + `drawtext=fontfile=${FONT}:text='${text(caption)}':x=(w-tw)/2:y=34:fontsize=18:fontcolor=0xbbbbbb,trim=end_frame=${frames},format=yuv420p[v]`,
  ].join(';');
  await run('ffmpeg', ['-y', '-loglevel', 'error', '-framerate', String(fps), '-i', left, '-framerate', String(fps), '-i', right,
    '-filter_complex', filter, '-map', '[v]', '-r', String(fps), '-c:v', 'libx264', '-crf', '23', '-preset', 'veryfast', '-movflags', '+faststart', video],
  { maxBuffer: 16 * 1024 * 1024 });
  return video;
}

// Concatenates per-case videos behind two-second title cards.
async function reel(videos, title) {
  const parts = [];
  for (const [index, video] of videos.entries()) {
    const card = join(out, `card-${index}.mp4`);
    const name = basename(resolve(video, '..'));
    await run('ffmpeg', ['-y', '-loglevel', 'error', '-f', 'lavfi', '-i', 'color=c=0x111111:s=1920x600:r=30:d=2',
      '-vf', `drawtext=fontfile=${FONT}:text='${text(name)}':x=(w-tw)/2:y=(h-th)/2:fontsize=48:fontcolor=white,format=yuv420p`,
      '-c:v', 'libx264', '-preset', 'veryfast', card]);
    parts.push(card, video);
  }
  const list = join(out, 'reel.txt');
  writeFileSync(list, parts.map(path => `file '${resolve(path)}'`).join('\n') + '\n');
  const target = join(out, `${title}.mp4`);
  // Re-encode so cards and cases at different frame rates join cleanly.
  await run('ffmpeg', ['-y', '-loglevel', 'error', '-f', 'concat', '-safe', '0', '-i', list, '-vf', 'fps=30,format=yuv420p',
    '-c:v', 'libx264', '-crf', '24', '-preset', 'veryfast', '-movflags', '+faststart', target]);
  for (const path of parts.filter(path => basename(path).startsWith('card-'))) rmSync(path);
  rmSync(list);
  return target;
}

const reelInput = flag('--reel');
if (reelInput) {
  const target = await reel(reelInput.split(','), flag('--title', 'reel'));
  console.log(JSON.stringify({ status: 'pass', video: target }));
} else {
  const cases = loadCases();
  const server = await startProductionServer({ directory: flag('--production-directory') ? resolve(flag('--production-directory')) : undefined });
  const browser = await chromium.launch(browserLaunchOptions());
  const results = [];
  try {
    for (const caseData of cases) {
      const id = String(caseData.scenario?.id ?? caseData.seed).replace(/[^A-Za-z0-9_-]/g, '-');
      const directory = join(out, id);
      rmSync(directory, { recursive: true, force: true });
      mkdirSync(directory, { recursive: true });
      writeFileSync(join(directory, 'case.json'), JSON.stringify(caseData) + '\n');
      const tapes = densify(caseData);
      console.log(`${id}: ${tapes.frames} frame pairs, seed ${caseData.seed}`);
      const [native, port] = await Promise.all([
        nativeFrames(caseData, tapes.native, join(directory, 'original')),
        browserFrames(browser, server, caseData, tapes.browser, join(directory, 'port')),
      ]);
      const counted = side => existsSync(side) ? readdirSync(side).filter(name => name.startsWith('sbs-')).length : 0;
      const result = { id, seed: caseData.seed, frames: tapes.frames, native, port,
        nativeFrames: counted(join(directory, 'original', 'frames')), portFrames: counted(join(directory, 'port')) };
      if (result.nativeFrames && result.portFrames) result.video = await compose(caseData, directory, tapes.frames, flag('--title', id));
      if (!keepFrames) {
        rmSync(join(directory, 'port'), { recursive: true, force: true });
        rmSync(join(directory, 'original', 'frames'), { recursive: true, force: true });
      }
      writeFileSync(join(directory, 'result.json'), JSON.stringify(result, null, 2) + '\n');
      console.log(JSON.stringify(result));
      results.push(result);
    }
  } finally {
    await browser.close();
    await server.close?.();
  }
  writeFileSync(join(out, 'report.json'), JSON.stringify({ sha, every, results }, null, 2) + '\n');
  process.exitCode = results.every(result => result.video) ? 0 : 1;
}
