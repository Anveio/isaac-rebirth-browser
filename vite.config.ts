import { defineConfig, type Plugin } from "vite";
import { fileURLToPath } from "node:url";
import { createReadStream, readFileSync } from "node:fs";
import { basename, resolve } from "node:path";

// Literal '#' in an authored filename is a module URL fragment to Rolldown.
// Emit the original bytes as assets instead of turning those paths into imports.
function originalMusic(): Plugin {
  const root = fileURLToPath(new URL(".", import.meta.url));
  const catalogue = JSON.parse(readFileSync(resolve(root, "web/audio-data.json"), "utf8")) as {
    assets: Record<string, { sha256: string }>;
  };
  const paths = Object.keys(catalogue.assets).filter(path => path.endsWith(".ogg"));
  const routeFiles = new Map(paths.map(path => [
    `/original-music/${catalogue.assets[path].sha256}.ogg`,
    resolve(root, "references/isaac/rebirth/resources", path),
  ]));
  let building = false;
  return {
    name: "original-music-assets",
    configResolved(config) { building = config.command === "build"; },
    resolveId(id) { if (id === "virtual:original-music") return "\0original-music"; },
    load(id) {
      if (id !== "\0original-music") return;
      const entries = paths.map(path => {
        const file = resolve(root, "references/isaac/rebirth/resources", path);
        this.addWatchFile(file);
        const value = building
          ? `import.meta.ROLLUP_FILE_URL_${this.emitFile({ type: "asset", name: basename(path).replaceAll("#", "_"), source: readFileSync(file) })}`
          : JSON.stringify(`/original-music/${catalogue.assets[path].sha256}.ogg`);
        return `${JSON.stringify(path)}: ${value}`;
      });
      return `export default {${entries.join(",")}};`;
    },
    configureServer(server) {
      server.middlewares.use((request, response, next) => {
        const file = routeFiles.get(request.url?.split("?")[0] ?? "");
        if (!file) return next();
        response.setHeader("Content-Type", "audio/ogg");
        createReadStream(file).on("error", next).pipe(response);
      });
    },
  };
}

export default defineConfig({
  plugins: [originalMusic()],
  // The port-only debug sandbox floor (web/debug-sandbox.ts) exists only in
  // npm run dev:sandbox; every other build folds it away (npm run bundle:check).
  define: { __DEBUG_SANDBOX__: JSON.stringify(process.env.DEBUG_SANDBOX === "1") },
  root: fileURLToPath(new URL("./web", import.meta.url)),
  build: {
    outDir: fileURLToPath(new URL("./dist", import.meta.url)),
    emptyOutDir: true,
    assetsInlineLimit: 0,
  },
});
