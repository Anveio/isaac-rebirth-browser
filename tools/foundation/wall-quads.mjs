// Backdrop wall assemblies the original submits to its room cache when a room
// loads (Backdrop::pre_render_walls 424440), read from the public GL observer
// (tools/original/foundation-gl-trace.c; a paired suite opts in with
// `--gl-trace`), and compared with the port's room.appearance.wallQuads.
// The original keeps no wall rects after pre-rendering, so its submitted
// cache draws are the observation. Each draw is 44 textured quads (264
// indices); a quad's source rect is its UVs scaled to the texture.
import { existsSync, readFileSync } from 'node:fs';
import { gunzipSync } from 'node:zlib';

// Submitted UVs sit within one f32 step of the authored pixels.
const pixel = value => Math.round(value * 100) / 100;

// The paired gate gzips the log once the original exits (paired-game.mjs). A
// long tape's log outgrows one JS string, so its lines are read from the buffer.
function* lines(path) {
  const buffer = existsSync(path) ? readFileSync(path) : gunzipSync(readFileSync(`${path}.gz`));
  for (let start = 0; start < buffer.length;) {
    const newline = buffer.indexOf(10, start), end = newline < 0 ? buffer.length : newline;
    if (end > start) yield buffer.toString('utf8', start, end);
    start = end + 1;
  }
}

export function originalWallAssemblies(commandsPath) {
  const sizes = new Map(), assemblies = [];
  for (const line of lines(commandsPath)) {
    const item = JSON.parse(line);
    if (item.op === 'texture') sizes.set(item.id, [item.width, item.height]);
    if (item.op !== 'draw' || !item.cacheDraw || item.count !== 264) continue;
    const size = item.textureSize ?? sizes.get(item.texture);
    if (!size || item.vertices.length !== 176) continue;
    const quads = [];
    for (let offset = 0; offset < item.vertices.length; offset += 4) {
      const vertices = item.vertices.slice(offset, offset + 4);
      if (!vertices.every(vertex => vertex.ok && vertex.uvOk)) throw new Error('Wall cache quad lacks submitted geometry');
      const uv = vertices.map(vertex => [vertex.uv[0] * size[0], vertex.uv[1] * size[1]]);
      const xs = uv.map(point => point[0]), ys = uv.map(point => point[1]);
      const dx = vertices.map(vertex => vertex.xy[0]), dy = vertices.map(vertex => vertex.xy[1]);
      quads.push({
        source: [Math.min(...xs), Math.min(...ys), Math.max(...xs), Math.max(...ys)].map(pixel),
        dest: [Math.min(...dx), Math.min(...dy), Math.max(...dx), Math.max(...dy)].map(pixel),
        flipX: uv[1][0] < uv[0][0], flipY: uv[2][1] < uv[0][1],
      });
    }
    assemblies.push(quads);
  }
  return assemblies;
}

// The same quads from a port room appearance: a patch index selects its cell
// of the backdrop sheet, then the quad's rect within it.
export function portWallAssembly(appearance) {
  const { width, height } = appearance.backdropPatchSize, columns = appearance.backdropPatchColumns;
  return appearance.wallQuads.map(quad => {
    const x = quad.sourcePatchIndex % columns * width + quad.sourceRect.x;
    const y = Math.floor(quad.sourcePatchIndex / columns) * height + quad.sourceRect.y;
    const { destRect: dest } = quad;
    return {
      source: [x, y, x + quad.sourceRect.width, y + quad.sourceRect.height].map(pixel),
      dest: [dest.x, dest.y, dest.x + dest.width, dest.y + dest.height].map(pixel),
      flipX: quad.flipX, flipY: quad.flipY,
    };
  });
}

// Every room the port entered, in entry order, must find an equal original
// assembly after the previous one. The original also pre-renders rooms no
// capture saw (and, after a revisit, the same room again), so this is an
// ordered subsequence match. Returns one row per port room.
export function compareWallAssemblies(original, rooms) {
  const rows = [];
  let cursor = 0;
  for (const { label, room, appearance } of rooms) {
    const expected = JSON.stringify(portWallAssembly(appearance));
    const found = original.findIndex((assembly, index) => index >= cursor && JSON.stringify(assembly) === expected);
    rows.push({ label, field: `room${room}.wallQuads`, expected: 'an original wall assembly in entry order',
      actual: found >= 0 ? `original assembly ${found} of ${original.length}` : 'no equal original assembly',
      status: found >= 0 ? 'pass' : 'fail' });
    if (found >= 0) cursor = found + 1;
  }
  return rows;
}
