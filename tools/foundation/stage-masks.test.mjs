import { test } from "node:test";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { cpSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { basename, dirname, join } from "node:path";
import { compareStageMaskTargets, loadControlledStageMaskEvidence, loadStageMaskDepthContract, loadStageMaskReference, readStageMaskCases, STAGE_MASK_DEPTH_CONTRACT, STAGE_MASK_MANIFEST } from "./stage-masks.mjs";
import { CONTROLLED_BOMB_CHECKPOINTS } from "./mask-checkpoints.mjs";

function controlledFixture(t, checkpoint = "movement") {
  const directory = mkdtempSync(join(tmpdir(), "isaac-controlled-mask-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  cpSync(dirname(STAGE_MASK_MANIFEST), directory, { recursive: true });
  const manifestPath = join(directory, "manifest.json");
  const depthContract = checkpoint === "depth-contract";
  const reference = depthContract ? { original: { controlledEvidence: { path: basename(STAGE_MASK_DEPTH_CONTRACT.path) } } }
    : readStageMaskCases(manifestPath).find((item) => item.checkpoint === checkpoint);
  const path = join(directory, reference.original.controlledEvidence.path);
  const saved = JSON.parse(readFileSync(path, "utf8"));
  const controllerText = readFileSync(join(directory, saved.controllerEvidence.path), "utf8");
  const traceText = readFileSync(join(directory, saved.graphicsTrace.path), "utf8");
  const targetBytes = readFileSync(join(directory, saved.targets.shadow.path));
  const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");
  return {
    reference,
    reject(mutate, pattern) {
      const evidence = structuredClone(saved), controller = JSON.parse(controllerText);
      const records = traceText.trim().split("\n").map((line) => JSON.parse(line));
      const bytes = Buffer.from(targetBytes);
      mutate({ evidence, controller, records, bytes });
      const controllerBytes = JSON.stringify(controller);
      writeFileSync(join(directory, evidence.controllerEvidence.path), controllerBytes);
      evidence.controllerEvidence.sha256 = sha(controllerBytes);
      evidence.presentationAcknowledgements = controller.presentations.map((entry) => Object.fromEntries(Object.entries(entry)
        .filter(([key]) => ["presentedFrame", "virtualElapsedNs", "consumedX11Events", "consumedX11FocusEvents", "inputOverflow"].includes(key))));
      const traceBytes = records.map((record) => JSON.stringify(record)).join("\n") + "\n";
      writeFileSync(join(directory, evidence.graphicsTrace.path), traceBytes);
      evidence.graphicsTrace.sha256 = sha(traceBytes);
      writeFileSync(join(directory, evidence.targets.shadow.path), bytes);
      const metadata = JSON.stringify(evidence);
      writeFileSync(path, metadata);
      const selected = { ...reference, original: { ...reference.original,
        controlledEvidence: { ...reference.original.controlledEvidence, sha256: sha(metadata) } } };
      assert.throws(() => depthContract ? loadStageMaskDepthContract({ path, sha256: sha(metadata) }) : loadControlledStageMaskEvidence(selected), pattern);
    },
  };
}

test("native stage-mask fixtures require actual seed logs and complete independent GPU target data", () => {
  const cases = readStageMaskCases(), references = cases.map(loadStageMaskReference);
  assert.ok(new Set(references.map((item) => item.original.captureId)).size >= 3, "Neutral masks need independently captured original seeds");
  assert.ok(references.some((item) => item.checkpoint === "movement"), "A movement checkpoint must remain selected");
  assert.throws(() => loadStageMaskReference({ ...cases[0], initialSeed: 1 }), /actual original log/);
  assert.throws(() => loadStageMaskReference({ ...cases[0], original: { ...cases[0].original, profile: { fresh: false, isolated: true } } }), /isolated fresh profile/);
});

test("controlled movement refresh preserves every selected case and retains the legacy original bytes", () => {
  const cases = readStageMaskCases();
  for (const id of ["8m72-neutral", "j4g3-neutral", "3bmb-neutral", "3bmb-moved"]) assert.ok(cases.some((item) => item.id === id), `${id} must remain selected`);
  const movement = cases.find((item) => item.id === "3bmb-moved");
  const controlled = loadControlledStageMaskEvidence(movement);
  assert.equal(controlled.evidence.publicPhase.movementHoldPresentations, 27);
  assert.equal(controlled.evidence.publicPhase.movementReleasePresentations, 30);
  assert.equal(controlled.evidence.simulationTicks, false);
  const legacy = movement.original.legacyTimingEvidence;
  assert.equal(legacy.original.captureId, "gl-full-3bmb-v10");
  for (const name of ["shadow", "lighting"]) {
    assert.notEqual(legacy.targets[name], movement.targets[name]);
    assert.equal(readFileSync(join(dirname(movement.manifestPath), legacy.targets[name])).length, 512 * 512 * 4);
  }
  assert.throws(() => loadStageMaskReference({ ...movement, original: { ...movement.original, controlledEvidence: undefined } }), /require controlled original evidence/);
});

test("controlled mask references reject shifted phases, skipped steps and altered actual X11 delivery", (t) => {
  const fixture = controlledFixture(t);
  fixture.reject(({ evidence }) => { evidence.publicPhase.movementHoldPresentations = 28; }, /Expected values/);
  fixture.reject(({ evidence }) => { evidence.snapshotRequest.armedAfterPresentedFrame++; }, /armed two presentations/);
  fixture.reject(({ controller }) => { controller.presentations.splice(2, 1); }, /execute every selected step/);
  fixture.reject(({ controller }) => {
    const event = controller.presentations.flatMap((phase) => phase.consumedX11Events).find((entry) => entry.keysym === 100 && entry.down);
    event.presentedFramePhase++;
  }, /consume every selected X11 event/);
  fixture.reject(({ controller }) => { controller.presentations.at(-1).inputOverflow = true; }, /cannot overflow/);
});

test("controlled explosion masks require every observed coasting phase and actual bomb-key delivery", (t) => {
  const cases = readStageMaskCases();
  for (const [checkpoint, presentations] of Object.entries(CONTROLLED_BOMB_CHECKPOINTS)) {
    const selected = cases.find((item) => item.checkpoint === checkpoint);
    assert.ok(selected, `${checkpoint} must remain selected`);
    const evidence = loadControlledStageMaskEvidence(selected).evidence;
    assert.equal(evidence.bombPhase.checkpointBombPresentation, presentations);
    assert.equal(evidence.publicPhase.checkpointPresentedFrame - evidence.publicPhase.movementCheckpointPresentedFrame, presentations);
    assert.throws(() => loadStageMaskReference({ ...selected, browserTape: { ...selected.browserTape, bombPresentation: presentations + 1 } }), /browser bomb phase/);
  }
  const fixture = controlledFixture(t, "explosion-start");
  fixture.reject(({ evidence }) => { evidence.bombPhase.checkpointBombPresentation++; }, /Expected values/);
  fixture.reject(({ evidence }) => { evidence.checkpoint = "unobserved-explosion"; }, /explicit movement or bomb checkpoint/);
  fixture.reject(({ controller, evidence }) => {
    const event = controller.presentations.flatMap((phase) => phase.consumedX11Events)
      .find((entry) => entry.keysym === 101 && entry.down && entry.presentedFramePhase > evidence.publicPhase.movementCheckpointPresentedFrame);
    event.presentedFramePhase++;
  }, /consume every selected X11 event/);
});

test("controlled mask references reject incomplete, reused or unpaired GL frames and altered target bytes", (t) => {
  const fixture = controlledFixture(t);
  fixture.reject(({ records }) => { records.pop(); }, /completed frame/);
  fixture.reject(({ records }) => { records.splice(1, 0, structuredClone(records[0])); }, /complete nonreused frame/);
  fixture.reject(({ records }) => { records.at(-1).monotonicNs++; }, /exact gated presentation time/);
  fixture.reject(({ records }) => {
    const index = records.findIndex((record) => record.op === "graphicsState");
    records.splice(index, 1);
  }, /associated public graphics state/);
  fixture.reject(({ evidence }) => { evidence.targets.shadow.epoch++; }, /selected texture epoch/);
  fixture.reject(({ records, evidence }) => {
    const index = records.findIndex((record) => record.op === "textureSnapshot" && record.id === evidence.targets.shadow.texture);
    records.splice(index, 1);
  }, /complete original shadow target snapshot/);
  fixture.reject(({ records, evidence }) => {
    const draw = records.find((record) => record.op === "draw" && record.renderTargetTexture === evidence.targets.shadow.texture);
    draw.vertices[0].xy[0]++;
  }, /every submitted rectangular corner/);
  fixture.reject(({ bytes }) => { bytes[0] ^= 1; }, /SHA256 differs/);
  fixture.reject(({ evidence }) => {
    evidence.repeatability.counterpartEvidence = fixture.reference.original.controlledEvidence.path;
  }, /independent original repeat/);
});

test("independent native depth evidence requires its complete attached GREATER pipeline and target provenance", (t) => {
  const capture = loadStageMaskDepthContract();
  assert.equal(capture.evidence.checkpoint, "explosion-start");
  assert.equal(capture.evidence.simulationTicks, false);
  assert.equal(capture.evidence.fullInternalState, false);
  const fixture = controlledFixture(t, "depth-contract");
  const state = (records) => records.find((record) => record.op === "graphicsState" && record.depthAttachment?.object > 0);
  fixture.reject(({ records }) => { state(records).depthFunction = [519]; }, /GREATER/);
  fixture.reject(({ records }) => { delete state(records).depthAttachment; }, /actual attached object/);
  fixture.reject(({ records }) => { state(records).depthAttachment.internalFormat = 33189; }, /DEPTH_COMPONENT24/);
  fixture.reject(({ records }) => { state(records).depthAttachment.framebuffer++; }, /DEPTH_COMPONENT24/);
  fixture.reject(({ evidence }) => { delete evidence.depthContract; }, /metadata must match/);
  fixture.reject(({ evidence }) => { evidence.independentlyRepeatedTargets.evidence = basename(STAGE_MASK_DEPTH_CONTRACT.path); }, /already selected independent/);
});

test("raw stage-mask equality detects transparent RGB differences and rejects partial or missing targets", () => {
  const bytes = Buffer.alloc(512 * 512 * 4), expected = { targets: { shadow: { width: 512, height: 512, data: bytes }, lighting: { width: 512, height: 512, data: bytes } } };
  const raw = { width: 512, height: 512, rgbaBase64: bytes.toString("base64") };
  const actual = { format: "RGBA8-straight", origin: "top-left-logical", shadow: raw, lighting: raw };
  assert.equal(compareStageMaskTargets(actual, expected).status, "pass");
  const wrongRGB = Buffer.from(bytes); wrongRGB[wrongRGB.length - 4] = 17;
  const mismatch = compareStageMaskTargets({ ...actual, shadow: { ...raw, rgbaBase64: wrongRGB.toString("base64") } }, expected);
  assert.equal(mismatch.status, "fail"); assert.equal(mismatch.results.shadow.changedPixels, 1);
  assert.equal(mismatch.originalStateParity, false);
  assert.throws(() => compareStageMaskTargets({ ...actual, lighting: undefined }, expected), /cannot be skipped/);
  assert.throws(() => compareStageMaskTargets({ ...actual, shadow: { ...raw, rgbaBase64: bytes.subarray(0, bytes.length - 4).toString("base64") } }, expected), /complete target/);
  assert.throws(() => compareStageMaskTargets({ ...actual, format: "premultiplied-RGBA8" }, expected), /straight RGBA/);
});
