import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { basename, dirname, isAbsolute, join, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { PNG } from "pngjs";
import { pixelDifference } from "./compare.mjs";
import { observedOriginalSeed } from "./original-seed-contract.mjs";
import { CONTROLLED_BOMB_CHECKPOINTS } from "./mask-checkpoints.mjs";

const ROOT = resolve(fileURLToPath(new URL("../..", import.meta.url)));
export const STAGE_MASK_MANIFEST = join(ROOT, "references/isaac/foundation/stage-masks/manifest.json");
export const STAGE_MASK_DEPTH_CONTRACT = Object.freeze({
  path: join(dirname(STAGE_MASK_MANIFEST), "3bmb-controlled-depth-contract.json"),
  sha256: "d03248a146ea8b864c78e7bafc6b4e1187b1e6a519569e8a04c5261fdd8da626",
});
const compact = (value) => value.replaceAll(" ", "");
const hash = (data) => createHash("sha256").update(data).digest("hex");
const CLOCK_EPOCH_NS = 1_000_000_000_000;

function portableFile(directory, path, sha256) {
  assert.ok(typeof path === "string" && path.length && !isAbsolute(path), "Controlled mask evidence must use portable relative paths");
  const filename = resolve(directory, path);
  assert.ok(filename.startsWith(resolve(directory) + sep), "Controlled mask evidence must remain beside its fixture");
  const data = readFileSync(filename);
  if (sha256 !== undefined) {
    assert.match(sha256, /^[a-f0-9]{64}$/, "Controlled evidence needs its SHA256");
    assert.equal(hash(data), sha256, `Controlled evidence SHA256 differs: ${path}`);
  }
  return data;
}

function controlledCapture(evidence, directory) {
  assert.equal(evidence.schema, 1);
  const bombPresentation = CONTROLLED_BOMB_CHECKPOINTS[evidence.checkpoint];
  assert.ok(evidence.checkpoint === "movement" || bombPresentation, "Controlled masks require an explicit movement or bomb checkpoint");
  assert.equal(evidence.format, "RGBA8-straight"); assert.equal(evidence.origin, "top-left-logical");
  assert.equal(evidence.width, 512); assert.equal(evidence.height, 512);
  assert.equal(evidence.simulationTicks, false); assert.equal(evidence.fullInternalState, false);
  assert.deepEqual(evidence.profile, { fresh: true, isolated: true });
  assert.ok(typeof evidence.captureId === "string" && evidence.captureId.length);
  const observed = observedOriginalSeed(portableFile(directory, evidence.originalLog).toString("utf8"));
  for (const [name, value] of Object.entries(observed)) assert.equal(evidence.initialState?.[name], value, `Controlled mask ${name} must match its original log`);
  const controller = JSON.parse(portableFile(directory, evidence.controllerEvidence.path, evidence.controllerEvidence.sha256));
  assert.equal(controller.schema, 1); assert.equal(controller.status, "pass");
  assert.equal(controller.captureId, evidence.captureId);
  assert.equal(controller.seedValue, observed.seedValue); assert.equal(compact(controller.seed), compact(observed.seed));
  assert.deepEqual(controller.initialState, evidence.initialState); assert.deepEqual(controller.profile, evidence.profile);
  assert.equal(controller.simulationTicks, false); assert.equal(controller.fullInternalState, false);
  assert.deepEqual(controller.inputDelivery, { api: "public X11 key events", serverAutoRepeat: false });
  assert.deepEqual(evidence.inputTape, controller.tape); assert.deepEqual(evidence.inputEvents, controller.inputs);
  const publicAcknowledgements = controller.presentations.map((entry) => Object.fromEntries(Object.entries(entry)
    .filter(([key]) => ["presentedFrame", "virtualElapsedNs", "consumedX11Events", "consumedX11FocusEvents", "inputOverflow"].includes(key))));
  assert.deepEqual(evidence.presentationAcknowledgements, publicAcknowledgements);
  const interval = controller.clock.stepNs;
  assert.ok(Number.isSafeInteger(interval) && interval > 0);
  assert.deepEqual(evidence.clock, controller.clock);
  const acknowledgements = controller.presentations;
  assert.ok(Array.isArray(acknowledgements) && acknowledgements.length >= 2, "Complete original clock acknowledgements are required");
  let phase = acknowledgements[1].presentedFrame;
  assert.ok(Number.isSafeInteger(phase) && phase > 0, "Original bootstrap presentation phase is required");
  const phases = [0, phase], captures = [], inputs = [];
  for (const command of controller.tape) {
    if (command.type === "step") {
      assert.ok(Number.isSafeInteger(command.frames) && command.frames > 0);
      phase += command.frames; phases.push(phase);
    } else if (command.type === "key") {
      assert.ok(typeof command.key === "string" && typeof command.down === "boolean");
      inputs.push([command.key, command.down, phase, phase * interval]);
    } else {
      assert.equal(command.type, "capture", "Every controlled tape command must execute");
      captures.push([command.label, phase, phase * interval]);
    }
  }
  assert.equal(new Set(captures.map(([label]) => label)).size, captures.length, "Controlled checkpoints must be unique");
  assert.deepEqual(acknowledgements.map((entry) => entry.presentedFrame), phases, "Original acknowledgements must execute every selected step");
  for (const entry of acknowledgements) {
    assert.ok(Number.isSafeInteger(entry.presentedFrame) && Number.isSafeInteger(entry.virtualElapsedNs));
    assert.equal(entry.virtualElapsedNs, entry.presentedFrame * interval);
    assert.equal(entry.inputOverflow, false, "Consumed original X11 inputs cannot overflow or be unavailable");
    assert.ok(Array.isArray(entry.consumedX11Events));
  }
  assert.deepEqual(controller.inputs.map((entry) => [entry.key, entry.down, entry.presentedFrame, entry.virtualElapsedNs]), inputs, "Issued X11 inputs must match the entire original tape");
  const expectedDelivery = controller.inputs.filter((entry) => entry.presentedFrame < phase).map((entry) => {
    assert.ok(Number.isInteger(entry.keysym) && entry.keysym > 0);
    return { down: entry.down, keysym: entry.keysym, presentedFramePhase: entry.presentedFrame + 1, virtualElapsedNs: (entry.presentedFrame + 1) * interval };
  });
  const consumed = acknowledgements.flatMap((entry) => entry.consumedX11Events);
  assert.deepEqual(consumed, expectedDelivery, "Original must consume every selected X11 event at its exact next presentation phase");
  assert.deepEqual(controller.frames.map((entry) => [entry.label, entry.presentedFrame, entry.virtualElapsedNs]), captures, "Every controlled capture must execute at its selected phase");
  for (const frame of controller.frames) {
    assert.equal(frame.width, 960); assert.equal(frame.height, 540);
    assert.equal(frame.gateBlocked, true); assert.equal(frame.simulationTick, null);
  }
  const baseline = controller.frames.find((frame) => frame.label === "gameplay-start");
  const movement = controller.frames.find((frame) => frame.label === "movement");
  assert.ok(baseline && movement, "Controlled masks require both baseline and movement checkpoints");
  assert.equal(movement.presentedFrame, baseline.presentedFrame + 57, "Controlled movement must preserve D27/release30");
  const movementEvents = consumed.filter((entry) => entry.presentedFramePhase > baseline.presentedFrame && entry.presentedFramePhase <= movement.presentedFrame);
  assert.deepEqual(movementEvents, [
    { down: true, keysym: 100, presentedFramePhase: baseline.presentedFrame + 1, virtualElapsedNs: (baseline.presentedFrame + 1) * interval },
    { down: false, keysym: 100, presentedFramePhase: baseline.presentedFrame + 28, virtualElapsedNs: (baseline.presentedFrame + 28) * interval },
  ], "Movement must consume exactly D27/release30 without other keys");
  const checkpoint = controller.frames.find((frame) => frame.label === evidence.checkpoint);
  assert.ok(checkpoint, "Every selected controlled mask checkpoint must execute");
  if (bombPresentation) assert.equal(checkpoint.presentedFrame, movement.presentedFrame + bombPresentation, "Bomb mask capture must execute at its independently observed public phase");
  assert.deepEqual(evidence.publicPhase, {
    baselinePresentedFrame: baseline.presentedFrame, movementHoldPresentations: 27, movementReleasePresentations: 30,
    ...(bombPresentation ? { movementCheckpointPresentedFrame: movement.presentedFrame } : {}),
    checkpointPresentedFrame: checkpoint.presentedFrame, checkpointVirtualElapsedNs: checkpoint.virtualElapsedNs, consumedMovementEvents: movementEvents,
  });
  if (bombPresentation) {
    const bombEvents = consumed.filter((entry) => entry.presentedFramePhase > movement.presentedFrame && entry.presentedFramePhase <= checkpoint.presentedFrame);
    assert.deepEqual(bombEvents, [
      { down: true, keysym: 101, presentedFramePhase: movement.presentedFrame + 1, virtualElapsedNs: (movement.presentedFrame + 1) * interval },
      { down: false, keysym: 101, presentedFramePhase: movement.presentedFrame + 2, virtualElapsedNs: (movement.presentedFrame + 2) * interval },
    ], "Bomb checkpoints require actual E down/one-step/up without extra inputs");
    assert.deepEqual(evidence.bombPhase, {
      key: "e", pressedAfterPresentedFrame: movement.presentedFrame, releasedAfterPresentedFrame: movement.presentedFrame + 1,
      checkpointBombPresentation: bombPresentation, consumedBombEvents: bombEvents,
    });
  }
  assert.deepEqual(evidence.snapshotRequest, {
    label: evidence.checkpoint, armedAfterPresentedFrame: checkpoint.presentedFrame - 2,
    expectedTargetPresentedFrame: checkpoint.presentedFrame, expectedTargetVirtualElapsedNs: checkpoint.virtualElapsedNs,
  }, "The native GL request must be armed two presentations before its paired checkpoint");
  const records = portableFile(directory, evidence.graphicsTrace.path, evidence.graphicsTrace.sha256).toString("utf8").trim().split("\n").map((line) => JSON.parse(line));
  assert.equal(records.filter((record) => record.op === "frameStart").length, 1, "Controlled graphics require one complete nonreused frame");
  assert.equal(records.filter((record) => record.op === "frameEnd").length, 1, "Controlled graphics require a completed frame");
  assert.equal(records[0].op, "frameStart"); assert.equal(records[0].label, evidence.checkpoint);
  assert.equal(records.at(-1).op, "frameEnd");
  assert.equal(records[0].frame, evidence.graphicsFrame); assert.equal(records.at(-1).frame, evidence.graphicsFrame);
  assert.equal(records.at(-1).monotonicNs - CLOCK_EPOCH_NS, checkpoint.virtualElapsedNs, "GL target readback must pair with the exact gated presentation time");
  let graphicsState;
  for (const record of records) {
    if (record.op === "graphicsState") graphicsState = record;
    if (["draw", "textureSnapshot"].includes(record.op)) assert.equal(record.frame, evidence.graphicsFrame, "Every original draw and snapshot must belong to the selected complete frame");
    if (record.op === "draw") {
      assert.ok(graphicsState && graphicsState.texture === record.texture && graphicsState.program === record.program, "Each original draw requires its associated public graphics state");
      for (const name of ["projection", "modelView"]) assert.ok(Array.isArray(graphicsState[name]) && graphicsState[name].length === 16 && graphicsState[name].every(Number.isFinite), "Original graphics transforms must be completely observed");
    }
  }
  const targets = {};
  for (const name of ["shadow", "lighting"]) {
    const target = evidence.targets[name];
    assert.ok(Number.isInteger(target.texture) && target.texture > 0 && Number.isInteger(target.epoch) && target.epoch > 0);
    const snapshots = records.filter((record) => record.op === "textureSnapshot" && record.id === target.texture && record.stage !== "cache-source");
    assert.equal(snapshots.length, 1, `Requires exactly one complete original ${name} target snapshot`);
    const snapshot = snapshots[0];
    assert.equal(snapshot.frame, evidence.graphicsFrame, "Original target snapshot must belong to the selected frame");
    assert.equal(snapshot.epoch, target.epoch, "Original target snapshot must belong to the selected texture epoch");
    assert.equal(snapshot.width, 512); assert.equal(snapshot.height, 512);
    const draws = records.filter((record) => record.op === "draw" && record.renderTargetTexture === target.texture);
    assert.ok(draws.length > 0, `Original ${name} target requires its actual submitted draw primitives`);
    const primitives = [];
    for (const draw of draws) {
      assert.equal(draw.frame, evidence.graphicsFrame); assert.equal(draw.mode, 4);
      assert.ok(draw.vertices.every((vertex) => vertex.ok && vertex.xy?.length === 2 && vertex.xy.every(Number.isFinite)), "Original target geometry cannot be unavailable");
      assert.ok(draw.vertices.length > 0 && draw.vertices.length % 4 === 0);
      assert.equal(draw.count, draw.vertices.length / 4 * 6);
      const indices = [];
      for (let offset = 0; offset < draw.vertices.length; offset += 4) indices.push(...[0, 2, 1, 1, 2, 3].map((index) => index + offset));
      assert.deepEqual(draw.indices, indices, "Target primitives require their complete original triangle indices");
      for (let offset = 0; offset < draw.vertices.length; offset += 4) {
        const vertices = draw.vertices.slice(offset, offset + 4);
        assert.ok(vertices.every((vertex) => vertex.colorOk && vertex.color?.length === 4 && vertex.color.every(Number.isFinite)), "Target primitive colors cannot be unavailable");
        // The lighting clear uses a solid-color draw with disabled UV arrays.
        // It is retained above; it cannot fabricate a textured light primitive.
        if (vertices.every((vertex) => !vertex.uvOk)) continue;
        assert.ok(vertices.every((vertex) => vertex.uvOk && vertex.uv?.length === 2 && vertex.uv.every(Number.isFinite)), "Target primitive UVs cannot be unavailable");
        const xy = vertices.map((vertex) => vertex.xy), uv = vertices.map((vertex) => vertex.uv.map((value, axis) => value * draw.textureSize[axis]));
        const destination = [Math.min(...xy.map((point) => point[0])), Math.min(...xy.map((point) => point[1])), Math.max(...xy.map((point) => point[0])), Math.max(...xy.map((point) => point[1]))];
        assert.deepEqual(xy, [[destination[0], destination[1]], [destination[2], destination[1]],
          [destination[0], destination[3]], [destination[2], destination[3]]], "Target primitive must retain every submitted rectangular corner");
        primitives.push({
          destination,
          source: [Math.min(...uv.map((point) => point[0])), Math.min(...uv.map((point) => point[1])), Math.max(...uv.map((point) => point[0])), Math.max(...uv.map((point) => point[1]))],
          flipX: uv[1][0] < uv[0][0], flipY: uv[2][1] < uv[0][1], color: vertices.map((vertex) => vertex.color),
        });
      }
    }
    assert.ok(primitives.length > 0, `Original ${name} target requires submitted textured primitives`);
    assert.deepEqual(primitives, evidence.primitives[name], "Portable primitive metadata must match the retained original GL frame");
    targets[name] = portableFile(directory, target.path, target.sha256);
    assert.equal(targets[name].length, 512 * 512 * 4, `Original ${name} target must contain all RGBA bytes`);
  }
  assert.notEqual(evidence.targets.shadow.texture, evidence.targets.lighting.texture, "Shadow and lighting require distinct original render targets");
  return { evidence, controller, observed, targets, records };
}

export function loadStageMaskDepthContract(selected = STAGE_MASK_DEPTH_CONTRACT) {
  const directory = dirname(selected.path);
  const evidence = JSON.parse(portableFile(directory, basename(selected.path), selected.sha256));
  const capture = controlledCapture(evidence, directory);
  assert.equal(evidence.checkpoint, "explosion-start", "Native depth evidence must retain its independently captured E121 checkpoint");
  const repeated = evidence.independentlyRepeatedTargets;
  assert.ok(repeated, "Native depth evidence requires its independently repeated target provenance");
  const reference = readStageMaskCases(join(directory, "manifest.json")).find((item) => item.original.controlledEvidence?.path === repeated.evidence);
  assert.ok(reference, "Native depth evidence must identify the already selected independent original case");
  assert.equal(repeated.sha256, reference.original.controlledEvidence.sha256);
  const original = loadControlledStageMaskEvidence(reference);
  for (const name of ["captureId", "originalLog"]) assert.notEqual(evidence[name], original.evidence[name], "Native depth proof requires a separate original capture");
  for (const name of ["controllerEvidence", "graphicsTrace"]) assert.notEqual(evidence[name].path, original.evidence[name].path, "Native depth proof requires separate retained controller and GL evidence");
  assert.deepEqual(evidence.publicPhase, original.evidence.publicPhase);
  assert.deepEqual(evidence.bombPhase, original.evidence.bombPhase);
  assert.deepEqual(evidence.initialState, original.evidence.initialState);
  for (const name of ["compiledGraphicsObserverSha256", "graphicsStateSourceSha256", "graphicsTraceSourceSha256"]) assert.match(evidence.sourceHashes[name], /^[a-f0-9]{64}$/, "Native depth proof requires its actual observer/source identity");
  for (const name of ["shadow", "lighting"]) {
    assert.ok(capture.targets[name].equals(original.targets[name]), "Native depth evidence must match every independently repeated target byte");
    const states = [];
    let state;
    for (const record of capture.records) {
      if (record.op === "graphicsState") state = record;
      if (record.op !== "draw" || record.renderTargetTexture !== evidence.targets[name].texture) continue;
      const fields = {
        depthEnabled: state.depthEnabled, depthFunction: state.depthFunction, depthWriteMask: state.depthWriteMask,
        depthBits: state.depthBits, depthClearValue: state.depthClearValue, depthRange: state.depthRange,
      };
      assert.deepEqual(fields, { depthEnabled: true, depthFunction: [516], depthWriteMask: [1], depthBits: [24], depthClearValue: [0], depthRange: [0, 1] }, "Every native target draw requires its complete GREATER/depth24/write/clear/range state");
      const attachment = state.depthAttachment;
      assert.ok(attachment && Number.isInteger(attachment.object) && attachment.object > 0, "Native depth evidence requires an actual attached object");
      assert.ok(Number.isInteger(record.framebuffer) && record.framebuffer > 0);
      assert.deepEqual(attachment, { framebuffer: record.framebuffer, available: true, objectType: 36161,
        object: attachment.object, renderbufferParametersAvailable: true, width: 1024, height: 1024,
        internalFormat: 33190, depthBits: 24, stencilBits: 0, samples: 0 }, "Native depth evidence requires its actual attached 1024-square DEPTH_COMPONENT24 renderbuffer");
      if (record.vertices.every((vertex) => vertex.uvOk)) states.push({ ...fields, depthAttachment: attachment });
    }
    assert.ok(states.length > 0, "Native depth evidence requires the actual target draw state");
    assert.deepEqual(evidence.depthContract?.[name], states, "Native depth contract metadata must match its retained paired GL frame");
  }
  return capture;
}

export function loadControlledStageMaskEvidence(reference) {
  const directory = dirname(reference.manifestPath), selected = reference.original.controlledEvidence;
  assert.ok(selected, "Movement mask references require controlled original evidence");
  const evidence = JSON.parse(portableFile(directory, selected.path, selected.sha256));
  const primary = controlledCapture(evidence, directory);
  assert.equal(evidence.checkpoint, reference.checkpoint, "Controlled evidence must match the selected executable checkpoint");
  assert.equal(primary.observed.seedValue, reference.initialSeed);
  assert.equal(evidence.captureId, reference.original.captureId);
  assert.equal(evidence.originalLog, reference.original.log);
  assert.deepEqual(evidence.initialState, reference.original.initialState);
  for (const name of ["shadow", "lighting"]) assert.equal(evidence.targets[name].path, reference.targets[name]);
  const counterpart = evidence.repeatability?.counterpartEvidence;
  assert.ok(counterpart && resolve(directory, counterpart) !== resolve(directory, selected.path), "Controlled masks require an independent original repeat");
  const repeat = controlledCapture(JSON.parse(portableFile(directory, counterpart)), directory);
  assert.notEqual(evidence.captureId, repeat.evidence.captureId, "Independent repeats require separate original capture identities");
  assert.notEqual(evidence.controllerEvidence.path, repeat.evidence.controllerEvidence.path, "Independent repeats require separate controller evidence");
  assert.notEqual(evidence.graphicsTrace.path, repeat.evidence.graphicsTrace.path, "Independent repeats require separate original GL frames");
  assert.notEqual(evidence.originalLog, repeat.evidence.originalLog, "Independent repeats require separate original logs");
  for (const name of ["originalControllerManifestSha256", "originalPublicGlTraceSha256"]) {
    assert.match(evidence.sourceHashes?.[name], /^[a-f0-9]{64}$/);
    assert.match(repeat.evidence.sourceHashes?.[name], /^[a-f0-9]{64}$/);
    assert.notEqual(evidence.sourceHashes[name], repeat.evidence.sourceHashes[name], "Independent repeats require separate original source evidence");
  }
  assert.equal(resolve(directory, repeat.evidence.repeatability?.counterpartEvidence || ""), resolve(directory, selected.path));
  assert.deepEqual(evidence.publicPhase, repeat.evidence.publicPhase);
  assert.deepEqual(evidence.bombPhase, repeat.evidence.bombPhase);
  assert.deepEqual(evidence.initialState, repeat.evidence.initialState);
  assert.deepEqual(evidence.primitives, repeat.evidence.primitives);
  assert.deepEqual(evidence.numericPositionCandidates, repeat.evidence.numericPositionCandidates);
  for (const name of ["shadow", "lighting"]) assert.ok(primary.targets[name].equals(repeat.targets[name]), "Independent original mask repeats must match every RGBA byte");
  return primary;
}
export function readStageMaskCases(path = STAGE_MASK_MANIFEST) {
  const manifest = JSON.parse(readFileSync(path, "utf8"));
  assert.equal(manifest.schema, 1); assert.equal(manifest.format, "RGBA8-straight"); assert.equal(manifest.origin, "top-left-logical");
  assert.equal(manifest.width, 512); assert.equal(manifest.height, 512);
  assert.ok(Array.isArray(manifest.cases) && manifest.cases.length > 0, "Original stage-mask scenarios cannot be empty");
  assert.equal(new Set(manifest.cases.map((item) => item.id)).size, manifest.cases.length, "Stage-mask scenario IDs must be unique");
  return manifest.cases.map((item) => ({ ...item, manifestPath: resolve(path), format: manifest.format, origin: manifest.origin, width: manifest.width, height: manifest.height }));
}

export function loadStageMaskReference(reference) {
  assert.ok(typeof reference.id === "string" && reference.id.length > 0);
  assert.ok(reference.original?.profile?.fresh && reference.original?.profile?.isolated, "Original stage masks require an isolated fresh profile");
  assert.ok(typeof reference.original.captureId === "string" && reference.original.captureId.length > 0);
  assert.ok(["gameplay-start", "movement", ...Object.keys(CONTROLLED_BOMB_CHECKPOINTS)].includes(reference.checkpoint), "Stage-mask inputs require an executable checkpoint");
  const directory = dirname(reference.manifestPath);
  const observed = observedOriginalSeed(readFileSync(resolve(directory, reference.original.log), "utf8"));
  assert.equal(observed.seedValue, reference.initialSeed, "Stage-mask fixture seed must match the actual original log");
  assert.equal(compact(observed.seed), compact(reference.seedText));
  for (const [name, value] of Object.entries(observed)) assert.equal(reference.original.initialState?.[name], value, `Stage-mask ${name} must match the actual original log`);
  if (reference.checkpoint === "movement" || CONTROLLED_BOMB_CHECKPOINTS[reference.checkpoint]) {
    assert.equal(reference.browserTape?.movementHoldTicks, 27); assert.equal(reference.browserTape?.movementReleaseTicks, 30);
    if (CONTROLLED_BOMB_CHECKPOINTS[reference.checkpoint]) {
      assert.equal(reference.browserTape.bombKey, "KeyE"); assert.equal(reference.browserTape.bombHoldTicks, 1);
      assert.equal(reference.browserTape.bombPresentation, CONTROLLED_BOMB_CHECKPOINTS[reference.checkpoint], "Selected browser bomb phase must match the original checkpoint");
    }
    loadControlledStageMaskEvidence(reference);
  }
  const targets = {};
  for (const name of ["shadow", "lighting"]) {
    assert.ok(typeof reference.targets?.[name] === "string", `Missing original ${name} target`);
    const path = resolve(directory, reference.targets[name]), data = readFileSync(path);
    assert.equal(data.length, reference.width * reference.height * 4, `Original ${name} must contain every RGBA byte`);
    targets[name] = { path, width: reference.width, height: reference.height, data };
  }
  return { ...reference, targets, observed };
}

export function compareStageMaskTargets(actual, reference, output) {
  assert.equal(actual.format, "RGBA8-straight", "Stage masks must preserve straight RGBA, including transparent RGB");
  assert.equal(actual.origin, "top-left-logical", "Stage-mask coordinate origins must agree");
  if (output) mkdirSync(output, { recursive: true });
  const results = {};
  for (const name of ["shadow", "lighting"]) {
    const target = actual[name], expected = reference.targets[name];
    assert.ok(target && expected, `Missing ${name} target cannot be skipped`);
    assert.equal(target.width, 512); assert.equal(target.height, 512);
    assert.equal(expected.width, 512); assert.equal(expected.height, 512);
    const data = Buffer.from(target.rgbaBase64, "base64");
    assert.equal(data.length, 512 * 512 * 4, `Browser ${name} must expose the complete target`);
    assert.equal(expected.data.length, data.length, `Original ${name} must expose the complete target`);
    const { difference, ...comparison } = pixelDifference(expected, { width: target.width, height: target.height, data });
    if (output) {
      writeFileSync(join(output, `${name}-browser.png`), PNG.sync.write({ width: target.width, height: target.height, data }));
      writeFileSync(join(output, `${name}-difference.png`), PNG.sync.write(difference));
    }
    results[name] = { ...comparison, passed: comparison.exactRGBA, budget: { changedPixels: 0, meanAbsoluteChannelError: 0, source: "Complete native GPU target bytes; exact equality required" } };
  }
  const report = { status: Object.values(results).every((item) => item.passed) ? "pass" : "fail", format: actual.format, origin: actual.origin, results, originalStateParity: false, originalSimulationTickDelivery: false, scope: "Complete original-referenced shadow/light target RGBA equality; whole-frame checks remain separate and mandatory" };
  if (output) writeFileSync(join(output, "comparison.json"), JSON.stringify(report, null, 2) + "\n");
  return report;
}
