import { test, expect } from "@playwright/test";
import { waitForReady } from "../foundation/verify.mjs";

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
  expect(opened.minimapPresentation.expandedAlpha).toBe(Math.fround(.3));
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
