import { test, expect, type Page, type TestInfo } from "@playwright/test";
import { waitForReady } from "../foundation/verify.mjs";

// Exercise the production host's real own-storage continuation. No profile
// document or dirty flag is installed by the driver; fresh contexts own both
// cases. The older V1 same-origin upgrade remains separate historical evidence.
async function checkLivingRestart(page: Page, info: TestInfo, seedCode: string | null) {
  const inputs: unknown[] = [], checkpoints: unknown[] = [];
  const state = () => page.evaluate(() => (window as any).game.snapshot());
  const own = () => page.evaluate(() => {
    const io = (window as any).game.saveGames();
    return { io, store: localStorage.getItem(io.key) };
  });
  const step = async (count: number) => {
    inputs.push({ type: "tick", count });
    return page.evaluate(count => {
      (window as any).game.step(count); return (window as any).game.snapshot();
    }, count);
  };
  const key = async (code: string, down: boolean) => {
    inputs.push({ type: "DOM-key", code, down });
    await page.keyboard[down ? "down" : "up"](code);
  };
  const press = async (code: string, settle = 72) => {
    await key(code, true); await step(12); await key(code, false); await step(settle);
  };
  const checkpoint = async (label: string) => {
    const captured = { label, state: await state(), own: await own() };
    checkpoints.push(captured); return captured;
  };
  const clockNs = (s: any) => BigInt(s.runAdmission.clock.milliseconds) * 1_000_000n
    + BigInt(s.runAdmission.clock.subMillisecondNanoseconds);
  try {
    await page.goto("/?manual=1"); await waitForReady(page);
    await page.evaluate(() => (window as any).game.pause()); await page.locator("#game").focus();
    expect((await own()).io.events).toContainEqual({ operation: "load", status: "absent" });
    await step(63);
    for (const screen of ["profile", "main", "character"]) {
      await press("Enter"); expect((await state()).screen).toBe(screen);
    }
    if (seedCode !== null) {
      await press("Tab", 108); expect((await state()).screen).toBe("seed");
      // The canonical public seed glyph tape, with real keyup ownership.
      for (const text of seedCode) {
        await key(text.toLowerCase(), true); await step(12);
        await key(text.toLowerCase(), false); await step(6);
      }
      expect((await state()).seedText).toBe(seedCode); await press("Enter");
      expect((await state()).screen).toBe("character");
    }
    await press("Enter", 300);
    const before = await checkpoint("before-R"), s = before.state, seeded = seedCode !== null;
    expect(s.screen).toBe("playing"); expect(s.character).toBe("Isaac"); expect(s.difficulty).toBe("normal");
    expect(s.challengeId).toBe(0); expect(s.seededRun).toBe(seeded); expect(s.achievementEligibility).toBe(!seeded);
    expect(s.persistentRuntime).toEqual({ dirty: false, inhibit: seeded });
    expect(s.livingRestart.context.sourceR9).toBe(seeded); expect(s.livingRestart.context.seedCode).toBe(s.runSeed);
    expect(s.livingRestart.heldPolls).toBe(0); expect(s.livingRestart.pending).toBeNull();
    expect(s.systemClock.interpolationEnabled).toBe(true); expect(s.nightmares.systemActive).toBe(false);
    expect(s.unlockNotifications.state).toBe(0); expect(s.player.dead).toBe(false);
    expect(before.own.io.declared).toBe(true); expect(before.own.io.storageAvailable).toBe(true);
    expect(before.own.io.truncated).toBe(false); expect(before.own.store).not.toBeNull();
    expect(before.own.io.events.some((event: any) => event.operation === "save" && event.status === "pass" && event.text === before.own.store)).toBe(true);
    const retained = JSON.parse(before.own.store!), slot = s.profile - 1, revision = s.saveRevision;
    expect(retained.version).toBe(2); expect(retained.profiles[slot].saved).not.toBeNull();
    expect(s.profiles[slot].counters[21]).toBe(0);
    if (seeded) expect(s.runSeed).toBe(seedCode);
    await key("KeyR", true); const pressed = await state();
    expect(pressed.pressed).toContain("KeyR"); expect(pressed.livingRestart.heldPolls).toBe(0); expect(pressed.tick).toBe(s.tick);
    for (let poll = 1; poll <= 60; poll++) {
      const actual = await step(1);
      expect(actual.tick).toBe(s.tick + poll); expect(actual.systemClock.systemCounter).toBe(s.systemClock.systemCounter + poll);
      expect(actual.livingRestart.heldPolls).toBe(poll); expect(actual.livingRestart.pending).toBeNull();
      expect(actual.runSeed).toBe(s.runSeed); expect(actual.saveRevision).toBe(revision);
      expect(actual.persistentRuntime.dirty).toBe(false); expect(actual.publicActions.held[16]).toBe(true);
      expect(actual.player.dead).toBe(false); expect(actual.nightmares.systemActive).toBe(false); expect(actual.unlockNotifications.state).toBe(0);
    }
    const held60 = await checkpoint("held-60"); expect(held60.own).toEqual(before.own);
    expect(held60.state.gameUpdateCount).toBeGreaterThan(s.gameUpdateCount);
    const polled = await step(1), held61 = await checkpoint("held-61-queued");
    expect(polled.tick).toBe(held60.state.tick + 1); expect(polled.systemClock.systemCounter).toBe(held60.state.systemClock.systemCounter + 1);
    expect(polled.gameUpdateCount).toBe(held60.state.gameUpdateCount + (held60.state.systemClock.systemCounter % 2 === 0 ? 1 : 0));
    expect(polled.livingRestart.heldPolls).toBe(0); expect(polled.livingRestart.pending).not.toBeNull();
    expect(polled.runSeed).toBe(s.runSeed); expect(polled.saveRevision).toBe(revision); expect(polled.runQuit.saveEnabled).toBe(false);
    expect(polled.persistentRuntime).toEqual({ dirty: !seeded, inhibit: seeded });
    expect(polled.profiles[slot].counters).toEqual(s.profiles[slot].counters); expect(held61.own).toEqual(before.own);
    const requested = polled.livingRestart.pending.seedCode;
    if (seeded) expect(requested).toBe(s.runSeed);
    await key("KeyR", false); const released = await state();
    expect(released.tick).toBe(polled.tick); expect(released.systemClock).toEqual(polled.systemClock);
    const restarted = await step(1), after = await checkpoint("next-public-interval");
    expect(restarted.tick).toBe(polled.tick + 1);
    expect(clockNs(restarted) - clockNs(polled)).toBe(BigInt(polled.runAdmission.clock.nanosecondsPerStep));
    expect(restarted.systemClock.systemCounter).toBe(polled.systemClock.systemCounter + polled.systemClock.systemCounter % 2 + 1);
    expect(restarted.runSeed).toBe(requested); expect(restarted.livingRestart.pending).toBeNull(); expect(restarted.livingRestart.heldPolls).toBe(0);
    expect(restarted.runQuit.saveEnabled).toBe(true); expect(restarted.persistentRuntime).toEqual({ dirty: false, inhibit: seeded });
    expect(restarted.seededRun).toBe(seeded); expect(restarted.achievementEligibility).toBe(!seeded);
    expect(restarted.saveWritePending).toBe(false); expect(restarted.saveReadmissionPending).toBe(false); expect(restarted.player.dead).toBe(false);
    expect(after.own.io.truncated).toBe(false); expect(after.own.io.events.slice(0, before.own.io.events.length)).toEqual(before.own.io.events);
    const writes = after.own.io.events.slice(before.own.io.events.length), count = seeded ? 2 : 3;
    expect(writes).toHaveLength(count); expect(restarted.saveRevision).toBe(revision + count);
    for (let i = 0; i < writes.length; i++) {
      expect(writes[i].operation).toBe("save"); expect(writes[i].status).toBe("pass"); expect(writes[i].revision).toBe(revision + i + 1);
    }
    const packets = writes.map((event: any) => JSON.parse(event.text));
    for (const packet of packets) expect(packet.version).toBe(2);
    if (!seeded) expect(packets[0].profiles[slot].saved).toEqual(retained.profiles[slot].saved);
    expect(packets[count - 2].profiles[slot].saved).toBeNull();
    const initial = packets[count - 1].profiles[slot].saved; expect(initial).not.toBeNull();
    // Save owns its literal legacy/current wire spelling. R owns the selected
    // code and numeric seed, and must carry those through the actual initial Save.
    expect(initial.run.runSeed.replaceAll(" ", "")).toBe(requested); expect(initial.context.runSeedValue).toBe(restarted.runSeedValue);
    expect(initial.run.seededRun).toBe(seeded); expect(initial.run.stage).toBe(1);
    expect(initial.run.gameUpdateCount).toBe(0); expect(initial.run.currentRoom).toBe(initial.run.floor.startingDescriptor);
    expect(initial.run.acquisitionHistory.firstRecords).toEqual({}); expect(after.own.store).toBe(writes.at(-1).text);
    expect((await page.evaluate(() => (window as any).game.feedback())).runtimeFailed).toBe(false);
  } finally {
    try { await checkpoint("final-actual"); }
    catch (error) { checkpoints.push({ label: "final-readback-failure", error: String(error) }); }
    await info.attach("public-R61-inputs-and-readback", { body: JSON.stringify({ inputs, checkpoints }), contentType: "application/json" });
  }
}

test("living Restart queues at poll61 and drains three real writes in the next public interval", async ({ page }, info) => {
  test.setTimeout(60_000); await checkLivingRestart(page, info, null);
});

test("living Restart preserves the public seed and drains two inhibited writes in the next public interval", async ({ page }, info) => {
  test.setTimeout(60_000); await checkLivingRestart(page, info, "T1PK6Y0W");
});

test("focus lifecycle clears held controls and resumes without undoing explicit diagnostic pause", async ({ page }) => {
  await page.goto("/"); await waitForReady(page);
  await page.keyboard.down("KeyD");
  await page.evaluate(() => window.dispatchEvent(new Event("blur")));
  const suspended = await page.evaluate(() => (window as any).game.snapshot());
  expect(suspended.pressed).not.toContain("KeyD");
  await page.waitForTimeout(120);
  expect((await page.evaluate(() => (window as any).game.snapshot())).tick).toBe(suspended.tick);
  await page.evaluate(() => window.dispatchEvent(new Event("focus")));
  await expect.poll(async () => (await page.evaluate(() => (window as any).game.snapshot())).tick).toBeGreaterThan(suspended.tick);
  await page.evaluate(() => {
    (window as any).game.pause();
    window.dispatchEvent(new Event("blur")); window.dispatchEvent(new Event("focus"));
  });
  const manual = await page.evaluate(() => (window as any).game.snapshot());
  await page.waitForTimeout(120);
  expect((await page.evaluate(() => (window as any).game.snapshot())).tick).toBe(manual.tick);
});

test("releasing a held bomb key in seed entry preserves text-only typing and the first gameplay press", async ({ page }) => {
  await page.goto("/"); await waitForReady(page);
  await page.evaluate(() => {
    const game = (window as any).game;
    game.pause(); game.reset(3350817387, { trace: true });
  });
  const step = async (count: number) => page.evaluate(count => (window as any).game.step(count), count);
  const press = async (code: string, settle: number) => {
    await page.keyboard.down(code); await step(12);
    await page.keyboard.up(code); if (settle) await step(settle);
  };
  // Follow the public menu tape's declared intervals, including New Run's
  // deferred admission, while keeping automatic simulation paused.
  await step(63);
  for (const screen of ["profile", "main", "character"]) {
    await press("Enter", 72);
    expect((await page.evaluate(() => (window as any).game.snapshot())).screen).toBe(screen);
  }
  await page.keyboard.down("KeyE");
  expect((await page.evaluate(() => (window as any).game.snapshot())).pressed).toContain("KeyE");
  await press("Tab", 108); await page.keyboard.up("KeyE");
  const released = await page.evaluate(() => ({
    state: (window as any).game.snapshot(), trace: (window as any).game.trace(),
  }));
  expect(released.state.screen).toBe("seed");
  expect(released.state.pressed).not.toContain("KeyE");
  expect(released.trace.commands.at(-1)).toEqual({ type: "key", code: "KeyE", down: false });
  const seed = "3BMBMFTE";
  await page.keyboard.type(seed);
  const typed = await page.evaluate(() => (window as any).game.trace());
  expect(typed.commands.slice(released.trace.commands.length)).toEqual([...seed].map(text => ({ type: "text", text })));
  expect(typed.truncated).toBe(false);
  await press("Enter", 72);
  const committed = await page.evaluate(() => (window as any).game.snapshot());
  expect(committed.screen).toBe("character");
  expect(committed.seededRun).toBe(true);
  await press("Enter", 300);
  const before = await page.evaluate(() => (window as any).game.snapshot());
  expect(before.screen).toBe("playing"); expect(before.inventory.bombs).toBe(1);
  expect(before.pressed).not.toContain("KeyE");
  expect(before.bombs).toHaveLength(0);
  await press("KeyE", 0);
  const after = await page.evaluate(() => (window as any).game.snapshot());
  expect(after.inventory.bombs).toBe(0); expect(after.bombs).toHaveLength(1);
  expect(after.pressed).not.toContain("KeyE");
});


test("the public Keyboard Map binding controls the expanded map after Back", async ({ page }) => {
  test.setTimeout(60_000);
  await page.goto("/"); await waitForReady(page);
  await page.evaluate(() => {
    const game = (window as any).game;
    game.pause(); game.reset(2184569263, { trace: true });
  });
  const state = async () => page.evaluate(() => (window as any).game.snapshot());
  const step = async (count: number) => page.evaluate(count => (window as any).game.step(count), count);
  const press = async (code: string, settle = 24, held = 12) => {
    await page.keyboard.down(code); await step(held);
    await page.keyboard.up(code); if (settle) await step(settle);
  };
  await step(63);
  for (const screen of ["profile", "main"]) {
    await press("Enter", 72); expect((await state()).screen).toBe(screen);
  }
  for (let row = 0; row < 3; row++) await press("ArrowDown");
  for (const screen of ["options", "controls", "keyboard"]) {
    await press("Enter", 72); expect((await state()).screen).toBe(screen);
  }
  const before = (await state()).keyboard.bindings;
  for (let row = 0; row < 12; row++) await press("ArrowDown");
  expect((await state()).controlsMenu.selection).toBe(12);
  await press("Enter"); expect((await state()).controlsMenu.waiting).toBe(true);
  await press("KeyI");
  const configured = await state();
  expect(configured.controlsMenu.waiting).toBe(false);
  expect(configured.keyboard.bindings).toEqual(before.map((pair: number[], action: number) => action === 13 ? [73, -1] : pair));
  for (const screen of ["controls", "options", "main"]) {
    await press("Escape", 72); expect((await state()).screen).toBe(screen);
  }
  for (let row = 0; row < 3; row++) await press("ArrowUp");
  await press("Enter", 72); expect((await state()).screen).toBe("character");
  await press("Tab", 108); expect((await state()).screen).toBe("seed");
  await page.keyboard.type("T1PK6Y0W");
  await press("Enter", 72); await press("Enter", 300);
  expect((await state()).screen).toBe("playing");
  await press("Tab", 18, 8);
  const oldKey = await state();
  expect(oldKey.publicActions.held[13]).toBe(false);
  expect(oldKey.minimapPresentation.mode).toBe(0);
  expect(oldKey.minimapPresentation.expandedAlpha).toBe(0);
  await page.keyboard.down("KeyI"); await step(8);
  const held = await state();
  expect(held.publicActions.held[13]).toBe(true);
  expect(held.controlsMenu.unavailableActions).not.toContain(13);
  await page.keyboard.up("KeyI"); await step(18);
  const opened = await state();
  expect(opened.publicActions.held[13]).toBe(false);
  expect(opened.minimapPresentation.mode).toBe(2);
  expect(opened.minimapPresentation.expandedLatched).toBe(true);
  expect(Math.fround(opened.minimapPresentation.expandedAlpha)).toBe(Math.fround(.3));
  expect(opened.keyboard.bindings[13]).toEqual([73, -1]);
  await press("KeyI", 18, 2);
  const closed = await state();
  expect(closed.minimapPresentation.mode).toBe(0);
  expect(closed.minimapPresentation.expandedLatched).toBe(false);
  expect(closed.minimapPresentation.expandedAlpha).toBe(0);
  const feedback = await page.evaluate(() => (window as any).game.feedback());
  expect(feedback.runtimeFailed).toBe(false);
  expect(feedback.truncated).toBe(false);
  expect(feedback.events.filter((event: any) => event.kind === "unavailableControl" && event.action === 13)).toEqual([]);
});


test("Gamma arrows reach the final display owner and Back survives a real document reload", async ({ page }) => {
  test.setTimeout(60_000);
  await page.goto("/"); await waitForReady(page);
  await page.evaluate(() => {
    const game = (window as any).game;
    game.pause(); game.reset(2184569263, { trace: true });
  });
  const state = async () => page.evaluate(() => (window as any).game.snapshot());
  const step = async (count: number) => page.evaluate(count => (window as any).game.step(count), count);
  const tap = async (key: string, held = 2, released = 2) => {
    await page.keyboard.down(key); await step(held);
    await page.keyboard.up(key); if (released) await step(released);
  };
  await step(63);
  for (const screen of ["profile", "main"]) {
    await tap("Enter", 12, 72); expect((await state()).screen).toBe(screen);
  }
  for (let row = 0; row < 3; row++) await tap("ArrowDown");
  await tap("Enter", 2, 60); expect((await state()).screen).toBe("options");
  for (let row = 0; row < 6; row++) await tap("ArrowDown");
  await step(60); expect((await state()).optionsMenu.selection).toBe(6);
  expect((await state()).gammaSettings.gamma).toBe(1);
  await tap("Enter"); expect((await state()).gammaSettings.gamma).toBe(1);
  await page.keyboard.down("ArrowLeft"); await step(53);
  let expected = Math.fround(1);
  for (let prior = 0; prior < 53; prior++)
    if (prior === 0 || prior > 49 && prior % 2 === 0)
      expected = Math.max(0.5, Math.fround(expected - Math.fround(0.01)));
  expect(Math.fround((await state()).gammaSettings.gamma)).toBe(expected);
  await step(98); expect((await state()).gammaSettings.gamma).toBe(0.5);
  await page.keyboard.up("ArrowLeft"); await step(2);
  await tap("ArrowRight", 255); expect((await state()).gammaSettings.gamma).toBe(1.5);
  await tap("Escape", 2, 60); expect((await state()).screen).toBe("main");
  const saved = await page.evaluate(() => localStorage.getItem('isaac-rebirth.options.v4'));
  expect(JSON.parse(saved!).gamma).toBe('1.5000');
  await page.reload(); await waitForReady(page);
  const loaded = await page.evaluate(() => ({state: (window as any).game.snapshot(),
    trace: (window as any).game.trace(), preferences: (window as any).game.preferences()}));
  expect(loaded.state.gammaSettings.gamma).toBe(1.5);
  expect(loaded.trace.bootGammaPreferences).toEqual({gamma: 1.5});
  expect(loaded.preferences.events.some((event: any) => event.operation === 'load' &&
    event.status === 'pass' && event.key === 'isaac-rebirth.options.v4' && event.value.gamma === '1.5000')).toBe(true);
});


test("a rejected stored profile refuses admission while menus remain usable and unrelated errors stay fatal", async ({ page }, info) => {
  test.setTimeout(60_000);
  const inputs: unknown[] = [], checkpoints: unknown[] = [];
  const state = () => page.evaluate(() => (window as any).game.snapshot());
  const checkpoint = async (label: string) => {
    const actual = await page.evaluate(() => {
      const game = (window as any).game, io = game.saveGames();
      return { state: game.snapshot(), io, stored: localStorage.getItem(io.key), feedback: game.feedback() };
    });
    checkpoints.push({ label, ...actual }); return actual;
  };
  const key = async (code: string, down: boolean) => {
    inputs.push({ type: "DOM-key", code, down });
    await page.keyboard[down ? "down" : "up"](code);
  };
  const step = async (count: number) => {
    inputs.push({ type: "tick", count });
    return page.evaluate(count => (window as any).game.step(count), count);
  };
  const press = async (code: string) => {
    await key(code, true); await step(12); await key(code, false); await step(72);
  };
  try {
    inputs.push({ type: "page", url: "/?manual=1" });
    await page.goto("/?manual=1"); await waitForReady(page);
    const fresh = await checkpoint("fresh-own-origin");
    expect(fresh.io.events).toContainEqual({ operation: "load", status: "absent" });
    expect(fresh.stored).toBeNull();
    // An intentional fault in this browser's own transport, never a Native
    // document, fabricated Game state or replacement engine snapshot.
    inputs.push({ type: "own-storage-fault", key: fresh.io.key, text: "null" });
    await page.evaluate(() => {
      const key = (window as any).game.saveGames().key;
      localStorage.setItem(key, "null");
    });
    inputs.push({ type: "real-document-reload" });
    await page.reload(); await waitForReady(page);
    await page.evaluate(() => (window as any).game.pause());
    await page.locator("#game").focus();
    const loaded = await checkpoint("actual-rejected-own-load");
    expect(loaded.stored).toBe("null");
    expect(loaded.io.declared).toBe(true); expect(loaded.io.truncated).toBe(false);
    expect(loaded.io.events.some((event: any) => event.operation === "load" && event.status === "fail")).toBe(true);
    expect(loaded.feedback.runtimeFailed).toBe(false);
    await step(63);
    for (const screen of ["profile", "main", "character"]) {
      await press("Enter"); expect((await state()).screen).toBe(screen);
    }
    const before = await checkpoint("before-refused-confirmation");
    // Resume BEFORE the refusal. No explicit step/resume afterward may hide
    // an accidental automaticPaused halt in the actual browser dispatcher.
    inputs.push({ type: "automatic-ticks", operation: "resume-before-refusal" });
    await page.evaluate(() => (window as any).game.resume());
    await key("Enter", true); await key("Enter", false);
    const refused = await checkpoint("actual-refused-confirmation");
    expect(refused.state.screen).toBe("character");
    expect(refused.state.runStartRequest.inputPending).toBe(false);
    expect(refused.state.runStartRequest.resumeSaved).toBe(false);
    expect(refused.state.runStartRequest.fading).toBe(false);
    expect(refused.state.runStartRequest.systemPending).toBe(false);
    expect(refused.state.saveWritePending).toBe(false); expect(refused.state.saveReadmissionPending).toBe(false);
    expect(refused.state.saveRevision).toBe(before.state.saveRevision);
    expect(refused.state.profiles).toEqual(before.state.profiles);
    expect(refused.io).toEqual(loaded.io); expect(refused.stored).toBe("null");
    expect(refused.feedback.runtimeFailed).toBe(false); expect(refused.feedback.truncated).toBe(false);
    const failures = refused.feedback.events.filter((event: any) => event.kind === "savedProfileAdmissionRefused");
    expect(failures).toHaveLength(1);
    expect(failures[0]).toMatchObject({ status: "fail", command: { type: "key", code: "Enter", down: true },
      error: "unknown rejected own-profile document must be recovered or explicitly reset", lastBrowserState: { screen: "character" } });
    await expect(page.locator("#game-feedback")).toContainText("The saved profile could not be loaded.");
    await expect.poll(async () => (await state()).tick).toBeGreaterThan(refused.state.tick);
    await key("Escape", true); await key("Escape", false);
    await expect.poll(async () => (await state()).screen).toBe("main");
    const navigated = await checkpoint("back-and-automatic-ticks-still-usable");
    expect(navigated.feedback.runtimeFailed).toBe(false); expect(navigated.io).toEqual(loaded.io);
    expect(navigated.stored).toBe("null");
    await expect.poll(async () => (await state()).tick).toBeGreaterThan(navigated.state.tick);
    // The actual quarantine flag alone must not reclassify unrelated errors.
    const invalid = { type: "audioVolume", sfx_volume: 2 };
    inputs.push({ type: "public-command-fatal-control", command: invalid });
    const fatal = await page.evaluate(command => {
      let error: string | null = null;
      try { (window as any).game.command(command); } catch (failure) { error = String(failure); }
      return { error, feedback: (window as any).game.feedback() };
    }, invalid);
    expect(fatal.error).not.toBeNull(); expect(fatal.feedback.runtimeFailed).toBe(true);
    expect(fatal.feedback.events.at(-1)).toMatchObject({ kind: "runtimeFailure", status: "fail", command: invalid });
    await expect(page.getByRole("button", { name: "Reload", exact: true })).toBeVisible();
    expect((await checkpoint("unrelated-error-retains-fatal-path")).stored).toBe("null");
  } finally {
    try { await checkpoint("final-actual"); }
    catch (error) { checkpoints.push({ label: "final-readback-failure", error: String(error) }); }
    await info.attach("own-profile-quarantine-inputs-and-readback", {
      body: JSON.stringify({ scope: "Own browser transport fault and runtime error classification; no Original-game parity claim", inputs, checkpoints }),
      contentType: "application/json",
    });
  }
});
