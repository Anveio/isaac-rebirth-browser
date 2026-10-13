import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdir, mkdtemp, rm, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { startProductionServer } from "./production-server.mjs";

test("production verification freezes build bytes and serves WASM with its actual build identity", async (t) => {
  const scratch = await mkdtemp(join(tmpdir(), "isaac-production-test-"));
  t.after(() => rm(scratch, { recursive: true, force: true }));
  const directory = join(scratch, "build");
  await mkdir(join(directory, "assets"), { recursive: true });
  await writeFile(join(directory, "index.html"), "<h1>Built game</h1>");
  const wasm = Buffer.from([0, 97, 115, 109, 1, 0, 0, 0]);
  await writeFile(join(directory, "assets/engine.wasm"), wasm);
  const first = await startProductionServer({ directory });
  t.after(() => first.close());
  assert.equal(first.build.files, 2);
  const html = await fetch(first.url);
  assert.equal(await html.text(), "<h1>Built game</h1>");
  assert.equal(html.headers.get("x-foundation-build"), first.build.sha256);
  const response = await fetch(`${first.url}/assets/engine.wasm`);
  assert.equal(response.headers.get("content-type"), "application/wasm");
  assert.deepEqual(Buffer.from(await response.arrayBuffer()), wasm);
  await writeFile(join(directory, "index.html"), "<h1>Next build</h1>");
  await rm(join(directory, "assets/engine.wasm"));
  assert.equal(await (await fetch(first.url)).text(), "<h1>Built game</h1>");
  assert.deepEqual(Buffer.from(await (await fetch(`${first.url}/assets/engine.wasm`)).arrayBuffer()), wasm);
  const second = await startProductionServer({ directory });
  t.after(() => second.close());
  assert.notEqual(second.build.sha256, first.build.sha256);
  assert.notEqual(second.url, first.url);
  assert.equal(await (await fetch(second.url)).text(), "<h1>Next build</h1>");
  assert.equal((await fetch(`${second.url}/assets/engine.wasm`)).status, 404);
});

test("production verification rejects missing builds and mutable linked assets without a development fallback", async (t) => {
  const scratch = await mkdtemp(join(tmpdir(), "isaac-production-missing-"));
  t.after(() => rm(scratch, { recursive: true, force: true }));
  await assert.rejects(startProductionServer({ directory: scratch }), /Missing production build/);
  await writeFile(join(scratch, "index.html"), "built");
  await writeFile(join(scratch, "external.png"), "external");
  await symlink(join(scratch, "external.png"), join(scratch, "linked.png"));
  await assert.rejects(startProductionServer({ directory: scratch }), /mutable symlink/);
  await assert.rejects(startProductionServer({ directory: scratch, port: -1 }), /port must be/);
});
