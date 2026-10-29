// The route recorder's page without a browser: the production snapshot's WASM
// driven through web/engine.ts, web/save-games.ts and web/host.ts, the modules
// the browser page (web/main.ts) runs, on a fresh page's empty storage. It
// implements only what boss-route-record.mjs calls: goto, waitForFunction,
// evaluate over window.game {pause, reset, step, snapshotJson}, keyboard
// down/up, and checkpoint/restore for --rewind. tools/e2e/README.md (Route
// recorder) lists what it leaves out.
import assert from 'node:assert/strict';
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { Engine } from '../../../web/engine.ts';
import { gameKey, resetWithSaves, settleSaveIO } from '../../../web/host.ts';
import { SaveGameIO } from '../../../web/save-games.ts';

// window.localStorage as a fresh page has it: empty, and never failing.
class MemoryStorage {
  #items = new Map();
  getItem(key) { return this.#items.has(key) ? this.#items.get(key) : null; }
  setItem(key, value) { this.#items.set(key, String(value)); }
  removeItem(key) { this.#items.delete(key); }
  get length() { return this.#items.size; }
  key(index) { return [...this.#items.keys()][index] ?? null; }
  entries() { return [...this.#items]; }
  replace(entries) { this.#items = new Map(entries); }
}

// The KeyboardEvent key and code Playwright's US layout sends for a key name.
// The one copy of browser behaviour here; a name outside it is refused.
function keyEvent(name) {
  if (/^Key[A-Z]$/.test(name)) return { key: name.slice(3).toLowerCase(), code: name };
  if (/^Digit[0-9]$/.test(name)) return { key: name.slice(5), code: name };
  if (/^[a-z]$/.test(name)) return { key: name, code: `Key${name.toUpperCase()}` };
  if (/^[0-9]$/.test(name)) return { key: name, code: `Digit${name}` };
  if (name === 'Space' || name === ' ') return { key: ' ', code: 'Space' };
  if (['Enter', 'Escape', 'Tab', 'Backspace', 'ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].includes(name)) return { key: name, code: name };
  throw new Error(`The headless page has no Playwright key mapping for ${JSON.stringify(name)}`);
}

// `server` is a startProductionServer() result: its immutable snapshot holds
// the one WASM module the browser page would fetch.
export async function headlessPage(server) {
  const assets = join(server.directory, 'assets');
  const modules = readdirSync(assets).filter(name => name.endsWith('.wasm'));
  assert.equal(modules.length, 1, 'The production snapshot must hold exactly one WASM module');
  const storage = new MemoryStorage();
  globalThis.window = { localStorage: storage };
  // main.ts boots on a random seed and resets on the recorder's; so does this.
  const engine = await Engine.create(readFileSync(join(assets, modules[0])), 1);
  const saves = new SaveGameIO();
  const failed = () => { throw new Error('Headless save IO failed on in-memory storage'); };
  if (!saves.load(engine)) failed();
  let current = engine.snapshot();
  const dispatch = command => (current = settleSaveIO(engine, saves, command, engine.command(command), failed));
  window.game = {
    // The page's own frame loop is all pause() stops; there is none here.
    pause() {},
    reset(seed, { bootEpochSeed = 0, bootElapsed, trace, audioPreferences, fullscreenPreferences, ...rest } = {}) {
      assert.deepEqual(Object.keys(rest), [], 'The headless page takes only seed, epoch, boot clock, trace, audio and fullscreen reset options');
      // A fresh page stores no preferences or bindings; `trace` records only.
      if (!resetWithSaves(engine, saves, seed, bootEpochSeed, undefined, audioPreferences, fullscreenPreferences ?? { fullscreen: false },
        undefined, undefined, undefined, bootElapsed)) failed();
      current = engine.snapshot();
      return current;
    },
    step: (count = 1) => dispatch({ type: 'tick', count }),
    snapshotJson: () => engine.snapshotJson(),
  };
  // Playwright marks a second keydown of a held key as a repeat.
  const held = new Set();
  const press = (name, down) => {
    const { key, code } = keyEvent(name);
    const repeat = down && held.has(code);
    if (down) held.add(code); else held.delete(code);
    const owned = gameKey({ code, key, down, repeat, ctrlKey: false, altKey: false, metaKey: false }, current, engine.publicKeys);
    if (owned?.command) dispatch(owned.command);
  };
  // The engine's whole state is its WASM linear memory (the module has no
  // imports), including a save write or menu readmission it still awaits.
  // A checkpoint also keeps, and a restore puts back, the page state outside it:
  // - `held`: the key-held set behind Playwright's repeat flag;
  // - `current`: the frame state whose screen and pressed keys decide
  //   seed-screen text and key ownership (web/host.ts gameKey);
  // - SaveGameIO's consumed save revision and IO events, and the in-memory
  //   storage a save write or menu readmission reads and writes;
  // - the Engine's background-stain cache.
  // Memory grown since a checkpoint stays allocated and unused after a restore.
  const memory = engine['wasm'].memory;
  return {
    checkpoint: () => ({ memory: new Uint8Array(memory.buffer).slice(), engine: { ...engine },
      saves: structuredClone({ ...saves, storage: null }), storage: storage.entries(), current, held: [...held] }),
    restore(saved) {
      new Uint8Array(memory.buffer).set(saved.memory);
      Object.assign(engine, saved.engine);
      Object.assign(saves, structuredClone(saved.saves), { storage });
      storage.replace(saved.storage);
      current = saved.current;
      held.clear(); for (const code of saved.held) held.add(code);
    },
    async goto() {},
    async waitForFunction() {},
    async evaluate(fn, arg) { return fn(arg); },
    keyboard: { async down(name) { press(name, true); }, async up(name) { press(name, false); } },
  };
}
