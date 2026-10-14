import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { createReadStream, rmSync } from "node:fs";
import { cp, mkdtemp, readdir, readFile, rm, stat } from "node:fs/promises";
import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { extname, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = resolve(fileURLToPath(new URL("../..", import.meta.url)));
const CONTENT_TYPES = {
  ".html": "text/html; charset=utf-8", ".js": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8", ".json": "application/json; charset=utf-8",
  ".wasm": "application/wasm", ".png": "image/png", ".svg": "image/svg+xml",
  ".jpg": "image/jpeg", ".jpeg": "image/jpeg", ".ogg": "audio/ogg",
  ".mp3": "audio/mpeg", ".wav": "audio/wav",
};

async function buildIdentity(directory) {
  const files = [];
  const visit = async (path) => {
    for (const entry of (await readdir(path, { withFileTypes: true })).sort((a, b) => a.name.localeCompare(b.name))) {
      const child = join(path, entry.name);
      assert.ok(!entry.isSymbolicLink(), "Production build snapshots cannot depend on mutable symlink targets");
      if (entry.isDirectory()) await visit(child);
      else if (entry.isFile()) files.push(child);
      else assert.fail(`Unsupported production build entry: ${child}`);
    }
  };
  await visit(directory);
  const hash = createHash("sha256");
  for (const file of files) {
    const bytes = await readFile(file);
    hash.update(relative(directory, file).split(sep).join("/")).update("\0");
    hash.update(String(bytes.length)).update("\0").update(bytes);
  }
  return { sha256: hash.digest("hex"), files: files.length };
}

// Verification serves a private copy of the built artifact. Later source edits
// and rebuilds cannot change bytes midway through an input/replay comparison.
export async function startProductionServer({ directory = join(ROOT, "dist"), port = 0 } = {}) {
  assert.ok(Number.isInteger(port) && port >= 0 && port <= 65535, "Production server port must be 0..65535");
  const source = resolve(directory);
  assert.ok((await stat(join(source, "index.html")).catch(() => null))?.isFile(), `Missing production build ${source}/index.html; run npm run build`);
  const scratch = await mkdtemp(join(tmpdir(), "isaac-production-"));
  // A suite that throws or exits without close() must not leak a dist copy.
  const removeScratch = () => rmSync(scratch, { recursive: true, force: true });
  process.once("exit", removeScratch);
  const snapshot = join(scratch, "dist");
  let server;
  try {
    await cp(source, snapshot, { recursive: true });
    const identity = await buildIdentity(snapshot);
    server = createServer(async (request, response) => {
      if (!["GET", "HEAD"].includes(request.method)) {
        response.writeHead(405, { Allow: "GET, HEAD" }); response.end(); return;
      }
      let filename;
      try {
        const pathname = decodeURIComponent(new URL(request.url, "http://127.0.0.1").pathname);
        filename = resolve(snapshot, `.${pathname === "/" ? "/index.html" : pathname}`);
        if (!filename.startsWith(snapshot + sep)) throw new Error("Outside snapshot");
      } catch { response.writeHead(400); response.end(); return; }
      const info = await stat(filename).catch(() => null);
      if (!info?.isFile()) { response.writeHead(404); response.end(); return; }
      response.writeHead(200, {
        "Content-Type": CONTENT_TYPES[extname(filename)] || "application/octet-stream",
        "Content-Length": info.size,
        "Cache-Control": "no-store",
        "X-Foundation-Build": identity.sha256,
      });
      if (request.method === "HEAD") response.end();
      else createReadStream(filename).on("error", () => response.destroy()).pipe(response);
    });
    await new Promise((done, reject) => {
      server.once("error", reject);
      server.listen(port, "127.0.0.1", done);
    });
    return {
      url: `http://127.0.0.1:${server.address().port}`,
      build: { ...identity, source, scope: "Immutable snapshot of production build bytes; source/native replay equality is checked separately" },
      close: async () => {
        await new Promise((done, reject) => server.close((error) => error ? reject(error) : done()));
        process.off("exit", removeScratch);
        await rm(scratch, { recursive: true, force: true });
      },
    };
  } catch (error) {
    if (server?.listening) await new Promise((done) => server.close(done));
    process.off("exit", removeScratch);
    await rm(scratch, { recursive: true, force: true });
    throw error;
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const value = (flag) => { const index = process.argv.indexOf(flag); return index < 0 ? undefined : process.argv[index + 1]; };
  const production = await startProductionServer({ directory: value("--directory"), port: Number(value("--port") || 5174) });
  console.log(JSON.stringify({ url: production.url, build: production.build }));
  for (const signal of ["SIGINT", "SIGTERM"]) process.once(signal, async () => { await production.close(); process.exit(0); });
}
