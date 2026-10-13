import assert from "node:assert/strict";
import { cp, mkdir, readFile, rename, rm, writeFile } from "node:fs/promises";
import { resolve, join } from "node:path";
import { fileURLToPath } from "node:url";
import { CHECKPOINTS } from "../foundation/verify.mjs";

const root = fileURLToPath(new URL("../..", import.meta.url));
const options = {
  destination: "/workspace/sites/isaac-browser-progress",
  dist: join(root, "dist"),
  report: join(root, ".cache/foundation/verification/immutable-mesa-final/report.json"),
  "mask-report": join(root, "test-results/foundation/stage-masks/report.json"),
};
for (let i = 2; i < process.argv.length; i += 2) {
  const key = process.argv[i].replace(/^--/, "");
  assert.ok(["destination", "dist", "report", "mask-report", "qa-report"].includes(key) && process.argv[i + 1], `Unknown or missing argument: ${process.argv[i]}`);
  options[key] = process.argv[i + 1];
}
assert.ok(options["qa-report"], "Provide --qa-report with actual browser {passed,failed,skipped} counts from the verified build");
const json = async (path) => JSON.parse(await readFile(resolve(path), "utf8"));
const [verification, masks, qa] = await Promise.all([json(options.report), json(options["mask-report"]), json(options["qa-report"])]);
const counts = (value, name) => {
  assert.ok(value && ["passed", "failed", "skipped"].every(key => Number.isInteger(value[key]) && value[key] >= 0), `${name} evidence must contain actual passed/failed/skipped counts`);
  return { passed: value.passed, failed: value.failed, skipped: value.skipped };
};
const frames = verification.checks.filter(check => check.comparison?.regions?.wholeWindow);
assert.deepEqual(frames.map(check => check.id), CHECKPOINTS, "Site status requires every original checkpoint in the executable verification sequence");
assert.ok(Array.isArray(masks.cases) && masks.cases.length > 0 && masks.cases.length === masks.selected && Number.isInteger(masks.skipped), "Exact-mask evidence must record every selected case");
const verifiedAt = qa.browser?.completedAt ?? verification.completedAt ?? verification.capturedAt;
assert.ok(typeof verifiedAt === "string" && Number.isFinite(Date.parse(verifiedAt)), "A retained actual verification UTC time is required");
const status = {
  generatedAt: new Date().toISOString(),
  verifiedAt: new Date(verifiedAt).toISOString(),
  browser: counts(qa.browser, "Browser"),
  ...(qa.node ? { node: counts(qa.node, "Node") } : {}),
  exactMasks: { passed: masks.cases.filter(item => item.status === "pass").length, failed: masks.cases.filter(item => item.status !== "pass").length, selected: masks.selected, skipped: masks.skipped },
  originalFrames: { passed: frames.filter(check => check.status === "pass").length, failed: frames.filter(check => check.status !== "pass").length, selected: frames.length },
  originalStateParity: verification.originalStateParity === true,
  scope: "Playable foundation checks and exact shadow/light mask subset; complete original game state and full-frame parity are separate verification requirements.",
};
const browserPassed = status.browser.failed === 0 && status.browser.skipped === 0;
const masksPassed = status.exactMasks.failed === 0 && status.exactMasks.skipped === 0;
const framePassed = status.originalFrames.failed === 0;
const verifiedLabel = status.verifiedAt.replace("T", " ").replace(/\.\d+Z$/, " UTC");
const wrapper = `<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Isaac browser progress</title>
<style>
*{box-sizing:border-box}body{margin:0;background:#171714;color:#eee8db;font:16px/1.5 system-ui,sans-serif}main{max-width:1000px;margin:auto;padding:24px 20px}h1{font-size:26px;line-height:1.2;margin:0 0 6px}p{margin:6px 0 18px;color:#c6c0b4}.game{background:#000;border:1px solid #504a40;border-radius:8px;overflow:hidden}iframe{display:block;width:100%;aspect-ratio:16/9;border:0}.checks{display:flex;flex-wrap:wrap;gap:8px;margin:18px 0 10px}.check{padding:7px 11px;border-radius:6px;border:1px solid #635e54;font-size:14px}.pass{color:#b9ddbb;border-color:#55725a}.fail{color:#f3c3a1;border-color:#987054}.scope{font-size:14px;max-width:850px}a{color:#ded4b9}
</style></head><body><main>
<h1>Isaac browser progress</h1><p>Playable Rust/WASM foundation. Movement and shooting are implemented.</p>
<div class="game"><iframe id="game-frame" src="game.html" title="Isaac browser foundation" allow="fullscreen"></iframe></div>
<p class="scope">Click the game to focus. Enter selects · WASD moves · arrows shoot · Escape pauses. <a href="game.html">Open game directly</a></p>
<div class="checks" aria-label="Current verification">
<span class="check ${browserPassed ? "pass" : "fail"}">${status.browser.passed} browser checks passed${status.browser.failed ? ` · ${status.browser.failed} failed` : ""}${status.browser.skipped ? ` · ${status.browser.skipped} skipped` : ""}</span>
<span class="check ${masksPassed ? "pass" : "fail"}">${status.exactMasks.passed}/${status.exactMasks.selected} exact shadow/light mask cases passed</span>
<span class="check ${framePassed ? "pass" : "fail"}">${framePassed ? `${status.originalFrames.passed}/${status.originalFrames.selected} original full-frame checks passed` : `${status.originalFrames.failed}/${status.originalFrames.selected} original full-frame checks still failing`}</span>
</div><p class="scope">The mask results cover a graphics subset. ${status.originalStateParity ? "Original state parity is reported for the selected scenarios." : "Full original-game state parity remains unverified."} This is a work in progress.</p>
<p class="scope">Verified <time datetime="${status.verifiedAt}">${verifiedLabel}</time> · <a href="verification-status.json">Verification data</a></p>
</main></body></html>`;

const destination = resolve(options.destination), source = resolve(options.dist);
assert.ok(destination !== root && destination !== source && !root.startsWith(destination + "/"), "The Site checkout must be isolated from the implementation workspace");
assert.notEqual(join(destination, "dist"), source, "The Site output must not overwrite the source build");
assert.ok(!(await readFile(join(source, "index.html"), "utf8")).includes("/main.ts"), "Build the actual production dist before preparing the Site");
await mkdir(destination, { recursive: true });
const output = join(destination, "dist");
await rm(output, { recursive: true, force: true });
await cp(source, output, { recursive: true });
await rename(join(output, "index.html"), join(output, "game.html"));
await writeFile(join(output, "index.html"), wrapper);
await writeFile(join(output, "verification-status.json"), JSON.stringify(status, null, 2) + "\n");
console.log(JSON.stringify({ destination, staticDirectory: output, browser: status.browser, exactMasks: status.exactMasks, originalFrames: status.originalFrames, originalStateParity: status.originalStateParity }, null, 2));
