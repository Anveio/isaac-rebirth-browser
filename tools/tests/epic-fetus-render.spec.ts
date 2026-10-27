import { test, expect } from "@playwright/test";
import { randomBytes } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { waitForReady } from "../foundation/verify.mjs";
import type { State } from "../../web/engine";

interface PublicCase {
  name: string;
  bootEpochSeed: number;
  initialPhaseOffset: number;
  entropy: string;
}
interface CaseBank { schema: number; cases: PublicCase[]; }

test("Epic Fetus renders its publicly spawned target and falling rockets", async ({ browser }, info) => {
  test.setTimeout(180_000);
  // Declare and retain the whole bank before any game. Replay the same file
  // against baseline/candidate; never redraw a failing OS epoch.
  const replay = process.env.FOUNDATION_EPIC_RENDER_CASES;
  const token = replay ? null : randomBytes(4);
  const bank: CaseBank = replay ? JSON.parse(readFileSync(replay, "utf8")) : {
    schema: 1,
    cases: [
      { name: "epic-fixed-phase0", bootEpochSeed: 1700000000, initialPhaseOffset: 0, entropy: "fixed" },
      { name: "epic-fixed-phase1", bootEpochSeed: 1700000000, initialPhaseOffset: 1, entropy: "fixed" },
      { name: "epic-first-os", bootEpochSeed: token!.readUInt32LE(), initialPhaseOffset: token!.readUInt32LE() >>> 31,
        entropy: token!.toString("hex") },
    ],
  };
  expect(bank.schema).toBe(1);
  expect(bank.cases.map(item => item.name)).toEqual(["epic-fixed-phase0", "epic-fixed-phase1", "epic-first-os"]);
  for (const item of bank.cases) {
    expect(Number.isInteger(item.bootEpochSeed) && item.bootEpochSeed >= 0 && item.bootEpochSeed <= 0xffffffff).toBe(true);
    expect([0, 1]).toContain(item.initialPhaseOffset);
  }
  expect(bank.cases.slice(0, 2)).toEqual([
    { name: "epic-fixed-phase0", bootEpochSeed: 1700000000, initialPhaseOffset: 0, entropy: "fixed" },
    { name: "epic-fixed-phase1", bootEpochSeed: 1700000000, initialPhaseOffset: 1, entropy: "fixed" },
  ]);
  const os = bank.cases[2]!;
  expect(os.entropy).toMatch(/^[a-f0-9]{8}$/);
  const osToken = Buffer.from(os.entropy, "hex");
  expect(os.bootEpochSeed).toBe(osToken.readUInt32LE());
  expect(os.initialPhaseOffset).toBe(osToken.readUInt32LE() >>> 31);
  const inputPath = info.outputPath("public-cases.json");
  writeFileSync(inputPath, JSON.stringify(bank, null, 2) + "\n");
  await info.attach("public-cases", { path: inputPath, contentType: "application/json" });
  const baselineXml = readFileSync("references/isaac/rebirth/resources/challenges.xml", "utf8");
  const failures: unknown[] = [];
  for (const selected of bank.cases) {
    const context = await browser.newContext({ baseURL: info.project.use.baseURL });
    const page = await context.newPage();
    const errors: string[] = [], commands: unknown[] = [], witnesses: unknown[] = [];
    const captured = new Set<string>();
    let failure: { error: string; stack?: string; screenshotError: string | null } | null = null;
    page.on("pageerror", error => errors.push(String(error)));
    const state = async () => page.evaluate(() => (window as any).game.snapshot()) as Promise<State>;
    const step = async (count: number) => {
      commands.push({ type: "tick", count });
      await page.evaluate(count => { (window as any).game.step(count); }, count);
    };
    const key = async (code: string, down: boolean) => {
      commands.push({ type: "key", code, down });
      if (down) await page.keyboard.down(code); else await page.keyboard.up(code);
    };
    const press = async (code: string, settle: number) => {
      await key(code, true); await step(12); await key(code, false); await step(settle);
    };
    try {
      await page.goto("/?manual=1"); await waitForReady(page);
      // The existing composition fixture's XML recipe, derived locally.
      // No original payload, cached stat or actor DTO is checked in.
      const resources = await page.evaluate(xml => {
        const document = new DOMParser().parseFromString(xml, "application/xml");
        const rows = Array.from(document.querySelectorAll('challenge[id="1"]'));
        if (document.querySelector("parsererror") || rows.length !== 1) throw Error("Actual unique Challenge1 required");
        const challenge = rows[0]!;
        challenge.setAttribute("getcurse", "0"); challenge.setAttribute("cursefilter", "127");
        challenge.removeAttribute("roomfilter"); challenge.setAttribute("startingitems", "168,2");
        return { challengesXml: new XMLSerializer().serializeToString(document) };
      }, baselineXml);
      commands.push({ type: "reset", initialSeed: 1, bootEpochSeed: selected.bootEpochSeed,
        resourceRecipe: { challengeId: 1, startingItems: [168, 2], getcurse: 0, cursefilter: 127, remove: ["roomfilter"] } });
      await page.evaluate(({ resources, bootEpochSeed }) => {
        const game = (window as any).game;
        game.pause(); game.reset(1, { bootEpochSeed, resources, audioPreferences: { sfxVolume: 0, musicVolume: 0 },
          fullscreenPreferences: { fullscreen: false } });
      }, { resources, bootEpochSeed: selected.bootEpochSeed });
      await step(63);
      for (const screen of ["profile", "main"]) {
        await press("Enter", 72); expect((await state()).screen).toBe(screen);
      }
      await press("ArrowDown", 72); await press("Enter", 72);
      expect((await state()).screen).toBe("challenge");
      await press("Enter", 300);
      const entered = await state();
      expect(entered.screen).toBe("playing"); expect(entered.challengeId).toBe(1);
      expect(entered.playerItems?.ownedCounts[168]).toBe(1); expect(entered.playerItems?.ownedCounts[2]).toBe(1);
      expect(entered.weaponCache.primary).toBe(6);
      expect(entered.weaponAdmission.pendingPrimaryCleanup).toBeNull();
      if (selected.initialPhaseOffset) await step(1);
      await key("ArrowRight", true);
      for (let presentation = 1; presentation <= 184; presentation++) {
        if (presentation === 25) await key("ArrowRight", false);
        await step(1);
        const actual = await state();
        const effects = actual.spriteEffects.filter(effect => [30, 31].includes(effect.variant) && effect.active && effect.visible);
        for (const effect of effects) {
          const label = effect.variant === 30 && effect.sprite.animation === "Blink" ? "target-blink"
            : effect.variant === 31 && effect.sprite.animation === "Falling" && effect.sprite.frame === 1 ? "rocket-falling" : null;
          if (!label || captured.has(label)) continue;
          // capture() invokes the production renderer on the real State.
          // A snapshot-only/WASM execution cannot satisfy these witnesses.
          const image = await page.evaluate(() => (window as any).game.capture());
          const path = info.outputPath(`${selected.name}-${label}.png`);
          writeFileSync(path, Buffer.from(image.png, "base64"));
          await info.attach(`${selected.name}-${label}`, { path, contentType: "image/png" });
          captured.add(label);
          witnesses.push({ label, presentation, effect, c18: actual.attackRuntime.persistent });
        }
      }
      expect([...captured].sort()).toEqual(["rocket-falling", "target-blink"]);
      expect(errors).toEqual([]);
      expect(await page.evaluate(() => (window as any).game.feedback().runtimeFailed)).toBe(false);
    } catch (error) {
      failure = { error: String(error), stack: error instanceof Error ? error.stack : undefined, screenshotError: null };
      failures.push({ name: selected.name, ...failure, pageErrors: errors });
      // A renderer exception follows the actual engine update. Preserve that
      // reached body as baseline evidence without retrying the failed dispatch.
      const failed = await state().catch(() => null);
      if (failed) witnesses.push({ label: "failed-dispatch", screen: failed.screen,
        effects: failed.spriteEffects.filter(effect => [30, 31].includes(effect.variant)),
        weaponCache: failed.weaponCache, weaponAdmission: failed.weaponAdmission,
        c18: failed.attackRuntime.persistent });
      // Preserve the actual displayed failure without asking the failed game
      // renderer to draw again. Each independent case keeps its own screen.
      try {
        const path = info.outputPath(`${selected.name}-failure.png`);
        await page.screenshot({ path });
        await info.attach(`${selected.name}-failure`, { path, contentType: "image/png" });
      } catch (error) { failure.screenshotError = String(error); }
    } finally {
      const feedback = await page.evaluate(() => (window as any).game.feedback())
        .catch(error => ({ observationError: String(error) }));
      const path = info.outputPath(`${selected.name}-observations.json`);
      writeFileSync(path, JSON.stringify({ selected, commands, witnesses, errors, failure, feedback,
        expectedBaselineError: "Unadmitted effect renderer 30",
        scope: "Actual Browser public-input rendering; no Original pixel, impact, RNG or audio parity claim." }, null, 2) + "\n");
      await info.attach(`${selected.name}-observations`, { path, contentType: "application/json" });
      await context.close();
    }
  }
  expect(failures, "Every declared fixed/OS case must render its actual target and falling rocket").toEqual([]);
});
