// Validate actual paired-game audio evidence; no fabricated events or samples.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile, writeFile } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const sha = bytes => createHash('sha256').update(bytes).digest('hex');

export function compareQueuedMusicOutput(events, observations, check) {
  const starts = events.filter(event => event.type === 'start' && event.channels === 2 && event.when <= event.audioTime + 0.001);
  const queued = events.filter(event => event.type === 'start' && event.channels === 2 && event.when > event.audioTime + 0.001);
  const sources = [['title-entry', 1, 0], ['floor-prepared', 2, 2], ['floor-prepared', 0, 3]];
  check('actual scheduled queued-node count', queued.length, sources.length);
  for (const [index, [label, channel, introIndex]] of sources.entries()) {
    const source = observations.get(label).music.channels[channel], actual = queued[index], intro = starts[introIndex];
    check(`queued${index}: native queued-stream record count`, source.queuedStreams.length, 1);
    check(`queued${index}: actual source and introduction observed`, !!actual && !!intro, true);
    if (!actual || !intro || source.queuedStreams.length !== 1) continue;
    const record = source.queuedStreams[0];
    check(`queued${index}: actual pinned decoder input`, actual.encodedPath, record.path.toLowerCase().replace(/^resources\//, ''));
    // This is the queued decoder's future loop flag, not current decoder loop
    // or AL_LOOPING (native streaming OpenAL sources remain non-looping).
    check(`queued${index}: actual future loop and pitch`, [actual.looping, actual.pitch], [record.loop, source.wrapperPitch]);
    check(`queued${index}: actual destination connection`, actual.reachesDestination, true);
    check(`queued${index}: introduction and queued node share gain`, actual.gainNode, intro.gainNode);
    check(`queued${index}: gapless authored device schedule`, actual.when, intro.when + intro.frames / intro.sampleRate / intro.pitch);
  }
}

export function compareMusicDeviceCheckpoint(capture, source, check) {
  const waiting = [];
  for (let channel = 0; channel < 3; channel++) {
    if (channel === 0 && capture.state.music.currentId === 61) continue; // Disclosed native-only100 tail.
    const actual = capture.audio.music.channels[channel], expected = source.music.channels[channel];
    const field = `${capture.label}: channel${channel} actual output`;
    if (['title-loop', 'run-request-held', 'pause-loop', 'resumed'].includes(capture.label))
      check(`${field} backend node presence after declared hold`, !!actual, expected.backendPresent);
    if (actual) {
      check(`${field} has native backend`, expected.backendPresent, true);
      check(`${field} gain/pitch and generation ownership`,
        [actual.generation === capture.state.music.channels[channel].generation, Math.fround(actual.gain), Math.fround(actual.pitch)],
        [true, Math.fround(expected.wrapperGain), Math.fround(expected.wrapperPitch)]);
    } else if (expected.backendPresent) waiting.push({ label: capture.label, channel,
      contextState: capture.audio.contextState, sourceBackendPresent: true,
      scope: 'Actual browser output has not started at this observation; autoplay/decode startup parity is not established by logical wrapper equality.' });
  }
  return waiting;
}

async function verifyCapture(directory) {
  const verifierSha256 = sha(await readFile(fileURLToPath(import.meta.url)));
  const originalReport = await readFile(join(directory, 'report.json')), report = JSON.parse(originalReport);
  assert.equal(report.status, 'pass', 'Sidecar requires the completed original live gate');
  assert.equal(report.sourceStability, 'pass', 'Captured producer/gate sources must have remained immutable');
  const result = { schema: 1, scope: 'Read-only stronger queued-output assertions on preserved actual original/browser game capture; no recapture or rewritten reference',
    verifierSha256, liveReportSha256: sha(originalReport), capturedSourceHashes: report.sourceHashes, production: report.production, cases: [] };
  for (const scenario of report.cases) {
    const caseDirectory = join(directory, scenario.name), apiBytes = await readFile(join(caseDirectory, 'browser-audio-api.json'));
    const manifestBytes = await readFile(join(caseDirectory, 'original/manifest.json')), statesBytes = await readFile(join(caseDirectory, 'browser-states.json'));
    const api = JSON.parse(apiBytes), manifest = JSON.parse(manifestBytes);
    assert.equal(api.truncated, false); assert.equal(manifest.status, 'pass');
    assert.equal(sha(manifestBytes), scenario.nativeManifestSha256, 'Sidecar must retain the exact native capture');
    const row = { name: scenario.name, browserAudioApiSha256: sha(apiBytes), nativeManifestSha256: sha(manifestBytes), browserStatesSha256: sha(statesBytes), checks: [], waitingOutputObservations: [] };
    const check = (field, actual, expected) => row.checks.push({ field, actual, expected,
      status: JSON.stringify(actual) === JSON.stringify(expected) ? 'pass' : 'fail' });
    const observations = new Map(manifest.debugObservations.map(item => [item.context.label, item.observation]));
    compareQueuedMusicOutput(api.events, observations, check);
    for (const capture of JSON.parse(statesBytes)) row.waitingOutputObservations.push(...compareMusicDeviceCheckpoint(capture, observations.get(capture.label), check));
    row.status = row.checks.every(item => item.status === 'pass') ? 'pass' : 'fail'; result.cases.push(row);
  }
  result.status = result.cases.every(row => row.status === 'pass') ? 'pass' : 'fail';
  const artifact = join(directory, 'music-output-verification.json');
  await writeFile(artifact, JSON.stringify(result, null, 2) + '\n');
  console.log(JSON.stringify({ status: result.status, cases: result.cases.length, checks: result.cases.reduce((n, row) => n + row.checks.length, 0), artifact, verifierSha256 }));
  if (result.status !== 'pass') process.exitCode = 1;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  assert.equal(process.argv.length, 3, 'Pass the completed live music capture directory');
  await verifyCapture(resolve(process.argv[2]));
}
