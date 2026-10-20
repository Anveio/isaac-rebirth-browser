import type { Command, Engine, State } from './engine';
import type { SaveGameIO } from './save-games';

// The engine host rules the browser page (main.ts) and the headless e2e page
// (tools/e2e/lib/headless-page.mjs) share, so a recorder driving the WASM
// without a browser feeds it exactly what the page would. Type-only imports
// keep this module loadable by Node's type stripping.

/** One keyboard event as main.ts reads it. */
export interface KeyInput {
  code: string; key: string; down: boolean; repeat: boolean;
  ctrlKey: boolean; altKey: boolean; metaKey: boolean;
}

/** Whether the game owns a focused key event, and the command it sends.
 * `null`: not the game's (an unowned release, an unsupported key). Otherwise
 * the host captures the event; `command` is null for an auto-repeat. On the
 * seed screen a plain letter or digit is seed text, not a key. */
export function gameKey(input: KeyInput, state: State, supported: ReadonlySet<string>): { command: Command | null } | null {
  const held = supported.has(input.code) && state.pressed.includes(input.code);
  // Releases belong to the key's real pressed owner, regardless of focus or
  // modifier flags. Unowned browser keyups must not be captured by the game.
  if (!input.down && !held) return null;
  const seedText = state.screen === 'seed' && /^[a-z0-9]$/i.test(input.key)
    && !input.ctrlKey && !input.altKey && !input.metaKey;
  if (!supported.has(input.code) && !seedText) return null;
  if (input.down && input.repeat) return { command: null };
  return { command: input.down && seedText ? { type: 'text', text: input.key.toUpperCase() } : { type: 'key', code: input.code, down: input.down } };
}

/** A batched public Tick can stop before a System iteration for real IO.
 * Complete each save write and menu readmission the engine suspends on, so
 * no rendering, audio callback or next public input sees partial Main state.
 * `failed` hears a storage failure; the engine still continues. */
export function settleSaveIO(engine: Engine, saves: SaveGameIO, command: Command, state: State, failed: () => void): State {
  for (let responses = 0; ; responses++) {
    if (!state.saveWritePending && !state.saveReadmissionPending) return state;
    //One write and one menu read can belong to one public System interval.
    const maxResponses = command.type === 'tick' ? command.count * 2 : 1;
    if (responses >= maxResponses) throw new Error('Save IO made no bounded tick progress');
    if (state.saveWritePending) {
      if (!saves.consume(engine, state.saveRevision)) failed();
    } else if (!saves.readmit(engine, state.saveRevision)) failed();
    state = engine.snapshot();
  }
}

/** A reset, then the save transport: enabling Save IO and importing the
 * stored profiles follows the engine reset. False on a storage failure. */
export function resetWithSaves(engine: Engine, saves: SaveGameIO, ...reset: Parameters<Engine['reset']>): boolean {
  engine.reset(...reset);
  return saves.load(engine);
}
