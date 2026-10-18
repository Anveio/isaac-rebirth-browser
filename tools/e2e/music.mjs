// Fresh original process + production WASM + forwarded actual audio API evidence.
import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { createHash, randomInt } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from '@playwright/test';
import { browserLaunchOptions, inspectBrowserGraphics } from '../foundation/browser.mjs';
import { startProductionServer } from '../foundation/production-server.mjs';
import { waitForReady, nativeReplay, resetBootElapsed } from '../foundation/verify.mjs';
import { exactStateDifference } from '../foundation/compare.mjs';
import { installBrowserAudioObserver } from './audio-api-observer.mjs';
import { compareQueuedMusicOutput, compareMusicDeviceCheckpoint } from './music-output-evidence.mjs';
import { flag, selectCases } from './lib/cli.mjs';
import { step, evaluateJson, readTrace } from './lib/game.mjs';
import { ORIGINAL_SHA, assertOriginalBinary } from './lib/original.mjs';

const ROOT = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const run = promisify(execFile), sha = bytes => createHash('sha256').update(bytes).digest('hex');
const value = flag => { const i = process.argv.indexOf(flag); return i < 0 ? undefined : process.argv[i + 1]; };
const output = resolve(value('--output') || join(ROOT, '.cache/e2e/music', String(Date.now())));
const count = Number(value('--count') || 2);
assert.ok(Number.isInteger(count) && count >= 1 && count <= 8);
await mkdir(output, { recursive: true });
const scenariosPath = flag('--scenarios');
assert.ok(scenariosPath === undefined || scenariosPath.length > 0, '--scenarios requires a nonempty path');
let cases, caseFilter;
if (scenariosPath !== undefined) {
  ({ cases, caseFilter } = JSON.parse(await readFile(resolve(scenariosPath), 'utf8')));
} else {
  const generated = JSON.parse((await run('python3', ['tools/original/foundation-seeds.py', '--count', String(count)], { cwd: ROOT })).stdout);
  cases = [{ name: 'audible-title-floor-pause', code: 'QSYVKMSE', bootEpochSeed: 1700000000, volume: 1 },
    { name: 'muted-real-lifecycle', code: 'ZG800TRJ', bootEpochSeed: 0, volume: 0 },
    ...generated.vectors.map((vector, i) => ({ name: `random-${i}`, code: vector.code.replaceAll(' ', ''), bootEpochSeed: randomInt(0x100000000), volume: i % 2 }))];
}
assert.ok(Array.isArray(cases) && cases.length > 0 && cases.length <= 10 && new Set(cases.map(c => c.name)).size === cases.length);
for (const c of cases) assert.ok(/^[a-zA-Z0-9_-]{1,40}$/.test(c.name) && /^[a-zA-Z0-9]{8}$/.test(c.code) &&
  Number.isInteger(c.bootEpochSeed) && c.bootEpochSeed >= 0 && c.bootEpochSeed <= 0xffffffff && (c.volume === 0 || c.volume === 1));
cases = selectCases(cases, c => c.name);
caseFilter = flag('--case') ?? caseFilter;
const catalogue = JSON.parse(await readFile(join(ROOT, 'web/audio-data.json')));
await assertOriginalBinary();
await writeFile(join(output, 'scenarios.json'), JSON.stringify({ schema: 1, entropy: 'Recorded OS-random run seeds and independent Node crypto boot epochs', caseFilter, cases }, null, 2) + '\n');
await run('cc', ['-std=c11', '-Wall', '-Wextra', '-Werror', '-fPIC', '-shared', 'tools/original/audio-trace.c', '-o', join(output, 'audio-trace.so'), '-ldl', '-pthread'], { cwd: ROOT });
const librarySha = sha(await readFile(join(output, 'audio-trace.so')));
const referencePcm = new Map();
async function verifySubmittedMusic(api, directory, check) {
  await mkdir(join(directory, 'browser-pcm'), { recursive: true });
  await mkdir(join(output, 'reference-pcm'), { recursive: true });
  for (const event of api.events.filter(event => event.type === 'start' && event.channels === 2 && event.pcmBase64)) {
    const entry = Object.entries(catalogue.assets).find(([, metadata]) => metadata.sha256 === event.encodedInput?.sha256);
    assert.ok(entry && entry[0].endsWith('.ogg'), 'Every actual music buffer must name its observed pinned decode input');
    const [path, metadata] = entry;
    check(`node${event.node}: original decode input/format`, [event.encodedInput.bytes, event.frames, event.sampleRate, event.channels],
      [metadata.bytes, metadata.frames, metadata.sampleRate, metadata.channels]);
    const actual = Buffer.from(event.pcmBase64, 'base64'); assert.equal(sha(actual), event.pcmSha256);
    const artifact = join(directory, 'browser-pcm', `node-${event.node}.pcm`);
    await writeFile(artifact, actual); event.pcmArtifact = artifact; event.encodedPath = path; delete event.pcmBase64;
    if (!referencePcm.has(path)) {
      const artifact = join(output, 'reference-pcm', metadata.sha256 + '.pcm');
      await run('ffmpeg', ['-hide_banner', '-loglevel', 'error', '-y', '-i', join(ROOT, 'references/isaac/rebirth/resources', path), '-f', 's16le', '-acodec', 'pcm_s16le', artifact], { cwd: ROOT });
      referencePcm.set(path, await readFile(artifact));
    }
    const expected = referencePcm.get(path); assert.equal(actual.length, expected.length, 'Complete submitted music buffer length must match original decoded audio');
    let maximum = 0, differingSamples = 0;
    for (let at = 0; at < actual.length; at += 2) {
      const difference = Math.abs(actual.readInt16LE(at) - expected.readInt16LE(at));
      maximum = Math.max(maximum, difference); if (difference) differingSamples++;
    }
    event.pcmComparison = { sourceDecoder: 'FFmpeg decoding the pinned original OGG', samples: actual.length / 2,
      maxInt16Difference: maximum, differingSamples, referencePcmSha256: sha(expected), quantizationBudget: 1 };
    check(`node${event.node}: complete authored PCM within1 int16 decoder rounding`, maximum <= 1, true);
  }
}

function tapes(code) {
  const native = [{ type: 'step', frames: 180 }, { type: 'key', key: 'Return', down: true }, { type: 'step', frames: 12 },
    { type: 'key', key: 'Return', down: false }, { type: 'step', frames: 9 }, { type: 'capture', label: 'title-entry' }];
  const rest = [{ type: 'step', frames: 160 }, { type: 'capture', label: 'title-settled' }, { type: 'audioHold', seconds: 15 },
    { type: 'step', frames: 1 }, { type: 'capture', label: 'title-loop' }];
  for (const label of ['profile', 'main', 'character']) rest.push({ type: 'key', key: 'Return', down: true }, { type: 'step', frames: 12 },
    { type: 'key', key: 'Return', down: false }, { type: 'step', frames: 72 }, { type: 'capture', label });
  rest.push({ type: 'key', key: 'Tab', down: true }, { type: 'step', frames: 12 }, { type: 'key', key: 'Tab', down: false }, { type: 'step', frames: 108 });
  for (const key of code.toLowerCase()) rest.push({ type: 'key', key, down: true }, { type: 'step', frames: 12 }, { type: 'key', key, down: false }, { type: 'step', frames: 6 });
  rest.push({ type: 'key', key: 'Return', down: true }, { type: 'step', frames: 12 }, { type: 'key', key: 'Return', down: false },
    { type: 'step', frames: 72 }, { type: 'capture', label: 'character-seeded' },
    { type: 'key', key: 'Return', down: true }, { type: 'step', frames: 12 }, { type: 'key', key: 'Return', down: false },
    { type: 'step', frames: 2 }, { type: 'capture', label: 'run-request' }, { type: 'audioHold', seconds: 6 },
    { type: 'step', frames: 1 }, { type: 'capture', label: 'run-request-held' },
    { type: 'step', frames: 27 }, { type: 'capture', label: 'run-request-threshold' },
    { type: 'step', frames: 1 }, { type: 'capture', label: 'floor-prepared' },
    { type: 'step', frames: 300 }, { type: 'capture', label: 'floor-settled' },
    { type: 'key', key: 'Escape', down: true }, { type: 'step', frames: 12 }, { type: 'key', key: 'Escape', down: false },
    { type: 'step', frames: 12 }, { type: 'capture', label: 'pause-before-loop' }, { type: 'audioHold', seconds: 29 },
    { type: 'step', frames: 2 }, { type: 'capture', label: 'pause-loop' },
    { type: 'key', key: 'Escape', down: true }, { type: 'step', frames: 12 }, { type: 'key', key: 'Escape', down: false },
    { type: 'step', frames: 42 }, { type: 'capture', label: 'resumed' });
  return { native: native.concat(rest), browser: rest };
}

function musicStarts(calls) {
  const settings = new Map(), uploads = new Map(), pending = new Map(), starts = [];
  for (const call of calls) {
    if (call.api === 'alBufferData') uploads.set(call.buffer, call);
    if (call.api === 'alSourcei' || call.api === 'alSourcef') {
      if (!settings.has(call.source)) settings.set(call.source, {});
      settings.get(call.source)[call.parameter] = call.value;
    }
    if (call.api === 'alSourcePlay' && call.source > 16 && call.clockDomain === 'gate-main-thread') {
      const start = { call, gain: settings.get(call.source)?.[0x100a], pitch: settings.get(call.source)?.[0x1003], upload: null, actualDevicePlay: false, devicePlay: null };
      starts.push(start); pending.set(call.source, start);
    }
    if (call.api === 'alSourceQueueBuffers' && pending.has(call.source) && !pending.get(call.source).upload)
      pending.get(call.source).upload = uploads.get(call.buffers[0]);
    if (call.api === 'alSourcePlay' && call.clockDomain === 'real-worker-thread' && pending.has(call.source)) {
      const start = pending.get(call.source); start.actualDevicePlay = true; start.devicePlay ??= call;
    }
  }
  return starts;
}

const production = await startProductionServer({ directory: value('--production-directory') }), browser = await chromium.launch(browserLaunchOptions());
const report = { schema: 1, status: 'running', originalBinarySha256: ORIGINAL_SHA, observerLibrarySha256: librarySha,
  production: production.build, graphics: await inspectBrowserGraphics(browser),
  scope: 'Fresh public menu/seed/play/pause inputs; source45-track table, Title61 intro/loop,95 real completion/room queue, fresh normal first-floor main/layer streams, exact f32 fades and actual Web Audio/OpenAL PCM. Capture names never establish native Playing: actual System state/public log are retained. Native pre-Title100 voiceover tail is explicitly skipped at browser entry. Alternate floor generation, later room jingles, count-driven layer admission and pitch special modes remain separate scope.', caseFilter, cases: [] };
try {
  for (const scenario of cases) {
    const directory = join(output, scenario.name); await mkdir(directory, { recursive: true });
    const tape = tapes(scenario.code); await writeFile(join(directory, 'tape.json'), JSON.stringify(tape.native, null, 2) + '\n');
    await writeFile(join(directory, 'browser-tape.json'), JSON.stringify(tape.browser, null, 2) + '\n');
    const result = { ...scenario, status: 'running', checks: [], diagnostics: [] }; report.cases.push(result); let page;
    const check = (field, actual, expected) => result.checks.push({ field, actual, expected, status: JSON.stringify(actual) === JSON.stringify(expected) ? 'pass' : 'fail' });
    try {
      await run('bash', ['tools/original/with-display.sh', 'python3', 'tools/original/foundation-clock-run.py', '--seed', scenario.code,
        '--boot-epoch-seed', String(scenario.bootEpochSeed), '--tape', join(directory, 'tape.json'), '--state-observer', 'tools/original/music-observer.py',
        '--sfx-volume', '0', '--music-volume', String(scenario.volume), '--audio-trace', join(output, 'audio-trace.so'), '--output', join(directory, 'original')],
      { cwd: ROOT, timeout: 240000, maxBuffer: 1024 * 1024 });
      const manifestBytes = await readFile(join(directory, 'original/manifest.json')), native = JSON.parse(manifestBytes);
      result.nativeManifestSha256 = sha(manifestBytes); assert.equal(native.status, 'pass'); assert.deepEqual(native.failures, []);
      assert.equal(native.seedReadbackMatchesCodec, true); assert.equal(native.bootEpochSeed, scenario.bootEpochSeed);
      const observations = new Map(native.debugObservations.map(row => [row.context.label, row.observation]));
      const callsBytes = await readFile(join(directory, 'original/audio-trace/calls.jsonl'));
      const calls = callsBytes.toString().trim().split('\n').map(JSON.parse), allNativeStarts = musicStarts(calls);
      assert.ok(calls.every(call => call.pid === observations.get('title-entry').originalPid), 'Every native audio API event must belong to the directly owned pinned process');
      assert.ok(calls.filter(call => call.api !== 'trace-start').every(call => Number.isSafeInteger(call.hostMonotonicNs) && call.hostMonotonicNs > 0),
        'Native forwarding observation must retain independent kernel monotonic timestamps');
      result.nativeAudioCallsSha256 = sha(callsBytes);
      const origin = observations.get('title-entry').systemCounter;
      page = await browser.newPage(); const errors = []; page.on('pageerror', error => errors.push(String(error)));
      await page.addInitScript(installBrowserAudioObserver); await page.goto(production.url + '/?manual=1'); await waitForReady(page);
      const bootElapsed = resetBootElapsed(tape.native);
      const apiBegin = await page.evaluate(({ seed, boot, bootElapsed, volume }) => { const begin = window.__audioApi.events.length;
        window.game.pause(); window.game.reset(seed, { bootEpochSeed: boot, bootElapsed, trace: true,
          audioPreferences: {sfxVolume: 0, musicVolume: volume}, fullscreenPreferences: {fullscreen: false} });
        return begin;
      }, { seed: native.seedValue, boot: scenario.bootEpochSeed, bootElapsed, volume: scenario.volume });
      check('initial autoplay suspended', (await page.evaluate(() => window.game.audio())).contextState, 'suspended');
      const commands = [], states = [];
      let intendedSeedEntry = false; const intendedPressed = new Set();
      for (const action of tape.browser) {
        if (action.type === 'step') { await step(page, action.frames); commands.push({ type: 'tick', count: action.frames }); }
        else if (action.type === 'audioHold') {
          if ((await page.evaluate(() => window.game.audio())).contextState !== 'running') { await page.locator('#game').click(); await page.waitForFunction(() => window.game.audio().contextState === 'running'); }
          await page.waitForTimeout(action.seconds * 1000);
        } else if (action.type === 'key') {
          const code = ({ Return: 'Enter', Tab: 'Tab', Escape: 'Escape' })[action.key] ||
            (/^[0-9]$/.test(action.key) ? 'Digit' + action.key : 'Key' + action.key.toUpperCase());
          await page.keyboard[action.down ? 'down' : 'up'](({ Return: 'Enter', Tab: 'Tab', Escape: 'Escape' })[action.key] || action.key);
          if (intendedSeedEntry && /^[a-z0-9]$/i.test(action.key)) {
            if (action.down) commands.push({ type: 'text', text: action.key.toUpperCase() });
            else if (intendedPressed.delete(code)) commands.push({ type: 'key', code, down: false });
          } else {
            commands.push({ type: 'key', code, down: action.down });
            if (action.down) intendedPressed.add(code); else intendedPressed.delete(code);
          }
          if (action.down && action.key === 'Tab') intendedSeedEntry = true;
          else if (action.down && action.key === 'Return' && intendedSeedEntry) intendedSeedEntry = false;
        } else {
          const actual = await evaluateJson(page, () => ({ state: window.game.snapshot(), audio: window.game.audio(), deviceTimes: window.__audioApi.contextTimes() }));
          states.push({ label: action.label, ...actual }); const expected = observations.get(action.label), music = expected.music;
          assert.equal(expected.status, 'pass'); assert.equal(expected.nativeBinarySha256, ORIGINAL_SHA); assert.equal(expected.originalPid > 0, true);
          assert.equal(expected.memoryWritten, false); assert.equal(expected.executableMappingsRead, false);
          check(`${action.label}: system counter`, actual.state.systemClock.systemCounter, expected.systemCounter - origin);
          check(`${action.label}: actual menu/game admission`, actual.state.screen === 'playing', music.systemState === 2);
          if (action.label === 'character-seeded' || action.label === 'floor-prepared') check(`${action.label}: manual public seed committed`,
            [actual.state.seededRun, actual.state.seedValid, actual.state.seedText.replaceAll(' ', '').toUpperCase(),
              actual.state.runSeed.replaceAll(' ', '').toUpperCase(), actual.state.runSeedValue],
            [true, true, scenario.code.toUpperCase(), scenario.code.toUpperCase(), native.seedValue]);
          if (music.systemState === 2) check(`${action.label}: source Level stage/alternate`,
            [actual.state.stage, actual.state.floor.contract.includes('normal')], [expected.level.stage, !expected.level.alternate]);
          const request = expected.runStartRequest;
          check(`${action.label}: source deferred System request`, actual.state.runStartRequest.systemPending, request.systemPending);
          if (request.menuPresent) check(`${action.label}: source menu request/f32 fade`,
            [actual.state.runStartRequest.fading, Math.fround(actual.state.runStartRequest.fadeAlpha)], [request.fading, request.fadeAlpha]);
          check(`${action.label}: controller`, [actual.state.music.activeChannel, actual.state.music.currentId, actual.state.music.queuedId,
            actual.state.music.volume, actual.state.music.musicEnabled, actual.state.music.layerEnabled, actual.state.music.pitch, actual.state.music.targetPitch],
          [music.activeChannel, music.currentId, music.queuedId, music.globalMusicVolume, music.musicEnabled, music.layerEnabled, music.pitch, music.targetPitch]);
          check(`${action.label}: source configured45 tracks`, music.tracks.map(track => ({ id: track.id, name: track.name, loop: track.loop,
            ...Object.fromEntries(['path', 'intro', 'layer', 'layerintro'].filter(key => track[key]).map(key => [key, track[key].toLowerCase().replace(/^resources\//, '')])) })), catalogue.music);
          for (let channel = 0; channel < 3; channel++) {
            // Source-only initial100 voiceover belongs to the skipped intro entry.
            if (channel === 0 && actual.state.music.currentId === 61) continue;
            const a = actual.state.music.channels[channel], e = music.channels[channel];
            check(`${action.label}: channel${channel}`, [a.playing, a.generation !== null, a.updatesSincePrepare,
              Math.fround(a.volume), Math.fround(a.targetVolume), Math.fround(a.fadeRate), a.wrapperLooping, Math.fround(a.gain), Math.fround(a.pitch)],
            [e.playing, e.backendPresent, e.updatesSincePrepare, Math.fround(e.volume), Math.fround(e.targetVolume), Math.fround(e.fadeRate), e.wrapperLoop, Math.fround(e.wrapperGain), Math.fround(e.wrapperPitch)]);
            if (e.backendPresent) {
              const field = `${action.label}: channel${channel} decoder and queued path`;
              const observedQueue = [a.backendLooping, a.queuedPath ? [{ path: 'resources/' + a.queuedPath, loop: a.queuedLooping }] : []];
              const sourceQueue = [e.backendLoop, e.queuedStreams.map(item => ({ ...item, path: item.path.toLowerCase() }))];
              // Decoder queue removal and audible onended are distinct contracts.
              // Retain the actual states and independently measured elapsed
              // playback, then gate exact stable state after declared late holds.
              if (['floor-settled', 'pause-before-loop', 'title-settled'].includes(action.label)) {
                const deviceStart = allNativeStarts.filter(start => start.call.source === e.openalSource &&
                  start.call.hostMonotonicNs <= expected.hostMonotonicNs).at(-1);
                const browserStart = actual.audio.music.events.findLast(event => event.type === 'start' && event.generation === a.generation);
                const path = browserStart?.path, metadata = path && catalogue.assets[path];
                result.diagnostics.push({ field, actual: observedQueue, expected: sourceQueue,
                  status: JSON.stringify(observedQueue) === JSON.stringify(sourceQueue) ? 'pass' : 'fail',
                  boundaryContracts: { native: 'Decoder EOF while refilling up to four 65536-byte OpenAL buffers', browser: 'Actual introduction AudioBufferSourceNode ended callback' },
                  deviceTiming: { nativeSnapshotHostMonotonicNs: expected.hostMonotonicNs,
                    nativeDevicePlayHostMonotonicNs: deviceStart?.devicePlay?.hostMonotonicNs ?? null,
                    nativeWallSecondsSinceObservedDevicePlay: deviceStart?.devicePlay ? (expected.hostMonotonicNs - deviceStart.devicePlay.hostMonotonicNs) / 1e9 : null,
                    nativeTimeScope: 'Kernel monotonic wall time, including observer thread stops; does not establish the audible playback cursor',
                    browserContext: actual.deviceTimes, browserStartAudioTime: browserStart?.audioTime ?? null,
                    browserSecondsSinceStart: browserStart ? actual.deviceTimes[0]?.currentTime - browserStart.audioTime : null,
                    authoredIntroductionSeconds: metadata ? metadata.frames / metadata.sampleRate : null,
                    nativeMaximumQueuedPcmSeconds: metadata ? 4 * 65536 / (metadata.channels * 2 * metadata.sampleRate) : null },
                  scope: 'Early native decoder/prefetch parity is unimplemented. Independent elapsed device scheduling can also differ. Both raw states remain recorded; stable explicit late-hold observations are gated.' });
              }
              else check(field, observedQueue, sourceQueue);
            }
          }
          check(`${action.label}: runtime errors`, actual.audio.music.errors, []);
          result.diagnostics.push(...compareMusicDeviceCheckpoint({ label: action.label, ...actual }, expected, check));
          if (expected.nativePause) check(`${action.label}: actual pause`, actual.state.paused, expected.nativePause.paused);
          result.diagnostics.push({ label: action.label, sourceSystemState: music.systemState, sourcePublicLog: native.debugObservations.find(row => row.context.label === action.label).context.publicLogState,
            browserScreen: actual.state.screen, sourceRoomEntry: expected.roomEntry ?? null,
            skippedSourceIntroTail: music.currentId === 61 ? music.channels[0] : null,
            runtime: actual.audio.music, sourceOddDispatchEnabled: music.oddSystemDispatchEnabled });
          await writeFile(join(directory, 'browser-states.json'), JSON.stringify(states, null, 2) + '\n');
        }
      }
      const trace = await readTrace(page); assert.equal(trace.enabled, true); assert.equal(trace.truncated, false);
      check('actual reset/run and boot inputs recorded', [trace.initialSeed, trace.bootEpochSeed], [native.seedValue, scenario.bootEpochSeed]);
      check('declared audio preferences applied before first admission', trace.bootAudioPreferences, {sfxVolume: 0, musicVolume: scenario.volume});
      check('declared window default applied at reset', trace.bootFullscreenPreferences, {fullscreen: false});
      check('actual Native public audio options', native.audioOptions, {sfxVolume: 0, musicVolume: scenario.volume});
      check('complete actual public DOM command tape', trace.commands, commands);
      assert.ok(trace.backendEvents.length >= 4, 'Actual music completion/intro-loop callbacks must be recorded');
      const rust = nativeReplay(native.seedValue, trace.commands, scenario.bootEpochSeed, trace.backendEvents, trace.bootAudioPreferences, trace.bootFullscreenPreferences, trace.bootInputConfigs, trace.bootMapPreferences, trace.bootGammaPreferences, bootElapsed);
      for (let i = 0; i < rust.length; i++) assert.equal(exactStateDifference(rust[i], trace.states[i]), null, `Actual callback replay divergence after public command${i}`);
      check('full public states replay actual backend callbacks', rust.length, trace.states.length);
      const api = await evaluateJson(page, async () => { await Promise.all(window.__audioApi.pending); return { events: structuredClone(window.__audioApi.events), truncated: window.__audioApi.truncated }; });
      await verifySubmittedMusic(api, directory, check);
      check('independent public API history complete', api.truncated, false); check('production page errors', errors, []);
      const starts = api.events.slice(apiBegin).filter(event => event.type === 'start' && event.channels === 2 && event.when <= event.audioTime + 0.001);
      const scheduled = api.events.slice(apiBegin).filter(event => event.type === 'start' && event.channels === 2 && event.when > event.audioTime + 0.001);
      compareQueuedMusicOutput(api.events.slice(apiBegin), observations, check);
      const titleLoop = scheduled[0], titleGeneration = states.find(row => row.label === 'title-loop').state.music.channels[1].generation;
      check('crossfade disables the actual old Title loop node', !!titleLoop && api.events.slice(api.events.indexOf(titleLoop) + 1).some(event =>
        event.type === 'loop' && event.node === titleLoop.node && event.looping === false), true);
      check('crossfade loop command retains the Title logical generation', states.find(row => row.label === 'run-request').audio.music.events.some(event =>
        event.type === 'loop' && event.channel === 1 && event.generation === titleGeneration && event.looping === false), true);
      const nativeStarts = allNativeStarts.slice(1); // Explicitly skipped native100 intro.
      check('actual controller music start count', starts.length, nativeStarts.length);
      for (let i = 0; i < Math.min(starts.length, nativeStarts.length); i++) {
        const a = starts[i], e = nativeStarts[i]; assert.ok(e.upload && !e.upload.truncated && e.actualDevicePlay, 'Actual native music start needs complete uploaded PCM and device Play');
        check(`start${i}: output graph reaches actual destination`, a.reachesDestination, true);
        check(`start${i}: actual Play parameters`, [a.gain, a.pitch, a.looping], [e.gain, e.pitch, false]);
        const pcm = await readFile(join(directory, 'original/audio-trace', e.upload.pcmArtifact)); assert.equal(pcm.length, e.upload.bytes);
        const differences = a.pcmPrefix.map((sample, index) => Math.abs(sample - pcm.readInt16LE(index * 2)));
        check(`start${i}: native/Web Audio Vorbis prefix within1 int16 rounding`, Math.max(...differences) <= 1, true);
      }
      const before = states.find(row => row.label === 'pause-before-loop'), after = states.find(row => row.label === 'pause-loop');
      for (const channel of [0, 2]) {
        check(`paused loop channel${channel} same actual generation`, after.state.music.channels[channel].generation, before.state.music.channels[channel].generation);
        check(`paused loop channel${channel} consumed queued intro`, after.state.music.channels[channel].queuedPath, null);
        check(`paused loop channel${channel} backend loops`, after.state.music.channels[channel].backendLooping, true);
        check(`paused loop channel${channel} same native backend`, observations.get('pause-loop').music.channels[channel].backendIdentity,
          observations.get('pause-before-loop').music.channels[channel].backendIdentity);
      }
      await writeFile(join(directory, 'browser-states.json'), JSON.stringify(states, null, 2) + '\n');
      await writeFile(join(directory, 'trace.json'), JSON.stringify(trace) + '\n');
      await writeFile(join(directory, 'browser-audio-api.json'), JSON.stringify(api, null, 2) + '\n');
      result.status = result.checks.every(check => check.status === 'pass') ? 'pass' : 'fail';
    } catch (error) { result.status = 'fail'; result.error = String(error.stack || error); }
    finally {
      if (page) {
        const final = await evaluateJson(page, async () => { await Promise.all(window.__audioApi.pending); return {
          state: window.game.snapshot(), audio: window.game.audio(), trace: window.game.trace(), api: { events: structuredClone(window.__audioApi.events), truncated: window.__audioApi.truncated } };
        }).catch(error => ({ captureError: String(error) }));
        if (final.api) for (const event of final.api.events) if (event.pcmBase64) {
          await mkdir(join(directory, 'browser-pcm'), { recursive: true });
          const artifact = join(directory, 'browser-pcm', `node-${event.node}.pcm`);
          await writeFile(artifact, Buffer.from(event.pcmBase64, 'base64')); event.pcmArtifact = artifact; delete event.pcmBase64;
        }
        await writeFile(join(directory, 'final-browser-evidence.json'), JSON.stringify(final) + '\n');
        await page.close();
      }
    }
    await writeFile(join(directory, 'report.json'), JSON.stringify(result, null, 2) + '\n');
    await writeFile(join(output, 'report.json'), JSON.stringify(report, null, 2) + '\n');
    console.log(JSON.stringify({ case: scenario.name, status: result.status, checks: result.checks.length, failed: result.checks.filter(check => check.status === 'fail').map(check => check.field), error: result.error }));
  }
  report.status = report.cases.every(result => result.status === 'pass') ? 'pass' : 'fail';
} finally { await browser.close(); await production.close(); await writeFile(join(output, 'report.json'), JSON.stringify(report, null, 2) + '\n'); }
console.log(JSON.stringify({ status: report.status, cases: report.cases.length, report: join(output, 'report.json') }));
if (report.status !== 'pass') process.exitCode = 1;
