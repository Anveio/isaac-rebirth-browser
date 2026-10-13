import { test, expect } from "@playwright/test";
import { createHash } from "node:crypto";
import { readFileSync, mkdirSync, writeFileSync } from "node:fs";
import { decodePNG } from "../../web/png";
import { waitForReady } from "../foundation/verify.mjs";
import type { State } from "../../web/engine";

const REFERENCES = "references/isaac/foundation/title-animation";
const ARTIFACTS = process.env.FOUNDATION_BROWSER_ARTIFACTS ?? "test-results/title";
interface SourceCase {
  label: string; presentation: number; sha256: string; comparison: "phase-state" | "full-title-frame";
  sourcePhase: { menuScreen: number; titleAnimation: { frame: number }; menuFade: { active: boolean; alpha: number; color: number[] } };
}
type Action = { type: "step"; frames: number } | { type: "key"; key: string; down: boolean } | { type: "capture"; label: string };

test("fresh source title fade, f32 animation, loop and retained reentry clock match seven complete original frames", async ({ page }) => {
  const reference = JSON.parse(readFileSync(`${REFERENCES}/manifest.json`, "utf8")) as { cases: SourceCase[] };
  const tape = JSON.parse(readFileSync(`${REFERENCES}/tape.json`, "utf8")) as Action[];
  const cases = new Map(reference.cases.map(item => [item.label, item]));
  const start = tape.findIndex(action => action.type === "capture" && action.label === "title-first");
  const end = tape.findIndex(action => action.type === "capture" && action.label === "title-return-settled");
  expect(start).toBeGreaterThan(0); expect(end).toBeGreaterThan(start);
  await page.goto("/?manual=1"); await waitForReady(page);
  // The engine boots at the first rendered Title snapshot, after the native
  // introductory cutscene. The reference tape retains that public prelude.
  await page.evaluate(() => {
    const game = (window as any).game;
    game.pause(); game.reset(2184569263, { bootEpochSeed: 1700000000, trace: true });
  });
  mkdirSync(`${ARTIFACTS}/title`, { recursive: true });
  const results = [];
  for (const action of tape.slice(start, end + 1)) {
    if (action.type === "step") { await page.evaluate(frames => (window as any).game.step(frames), action.frames); continue; }
    if (action.type === "key") {
      const key = action.key === "Return" ? "Enter" : action.key;
      if (action.down) await page.keyboard.down(key); else await page.keyboard.up(key);
      continue;
    }
    const expected = cases.get(action.label)!;
    const state = await page.evaluate(() => (window as any).game.snapshot()) as State;
    writeFileSync(`${ARTIFACTS}/title/${action.label}-state.json`, JSON.stringify({ expected: expected.sourcePhase, actual: state }, null, 2));
    expect(state.screen).toBe(expected.sourcePhase.menuScreen === 1 ? "title" : "profile");
    expect(state.titleAnimationFrame, action.label).toBe(expected.sourcePhase.titleAnimation.frame);
    expect(state.menuFade.active, action.label).toBe(expected.sourcePhase.menuFade.active);
    expect(state.menuFade.alpha, action.label).toBe(expected.sourcePhase.menuFade.alpha);
    if (expected.comparison !== "full-title-frame") { results.push({ label: action.label, comparison: "phase-state", status: "pass" }); continue; }
    const bytes = readFileSync(`${REFERENCES}/frames/${action.label}.png`);
    expect(createHash("sha256").update(bytes).digest("hex")).toBe(expected.sha256);
    const capture = await page.evaluate(() => (window as any).game.capture());
    const actualBytes = Buffer.from(capture.png, "base64");
    writeFileSync(`${ARTIFACTS}/title/${action.label}-browser.png`, actualBytes);
    const actual = await decodePNG(actualBytes), original = await decodePNG(bytes);
    expect(actual.width).toBe(original.width); expect(actual.height).toBe(original.height);
    let changedPixels = 0, maximumChannelError = 0;
    for (let at = 0; at < original.pixels.length; at += 4) {
      let changed = false;
      for (let channel = 0; channel < 4; channel++) {
        const error = Math.abs(actual.pixels[at + channel]! - original.pixels[at + channel]!);
        maximumChannelError = Math.max(maximumChannelError, error); changed ||= error !== 0;
      }
      if (changed) changedPixels++;
    }
    results.push({ label: action.label, comparison: "full-title-frame", changedPixels, maximumChannelError });
  }
  writeFileSync(`${ARTIFACTS}/title/comparison.json`, JSON.stringify(results, null, 2));
  expect(results.filter(item => item.comparison === "full-title-frame")).toHaveLength(7);
  expect(results.filter(item => item.changedPixels !== undefined && item.changedPixels !== 0), "Every complete title image must match without a pixel tolerance").toEqual([]);
});
