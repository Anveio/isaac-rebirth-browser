import {audioObservedBrowser} from './lib/audio-owners.mjs';

// Only returns already produced by the real public reset/step. No extra tick,
// snapshot command, callback, input, timing adjustment or state mutation.
function installTntReturns() {
  const rows = [], inputs = [], LIMIT = 4096, MAX_BYTES = 24 * 1024 ** 2;
  let game, presentation = 0, bytes = 0, truncated = false, inputTruncated = false, error = null;
  const project = (state, command) => {
    const row = {presentation, command, screen: state.screen, systemCounter: state.systemClock.systemCounter,
      gameUpdateCount: state.gameUpdateCount, stage: state.stage, inventory: state.inventory};
    if (state.screen === 'playing') Object.assign(row, {
      room: {descriptorIndex: state.room.descriptor?.index, gridIndex: state.room.descriptor?.gridIndex,
        catalogIndex: state.room.descriptor?.catalogIndex, spawnSeed: state.room.descriptor?.spawnSeed},
      grid: state.room.grid && {width: state.room.grid.width, height: state.room.grid.height,
        tnt: state.room.grid.cells.flatMap((cell, index) => cell.entity?.kind === 12 ? [{index, ...cell.entity}] : [])},
      pathCosts: state.room.pathCosts, player: state.player, damageCooldown: state.damageCooldown,
      pickups: state.room.pickups,
      effects: state.spriteEffects.filter(effect => [1, 18, 27].includes(effect.variant)),
      sequenceIdentities: state.actorScheduler.identities,
      background: state.room.backgroundStains.filter(effect => [18, 27].includes(effect.variant)),
      audioEvents: state.audio.events, audio182: state.audio.slots['182']});
    return row;
  };
  Object.defineProperty(window, 'game', {configurable: true, enumerable: true, get: () => game, set(value) {
    game = value;
    const reset = value.reset, step = value.step;
    value.reset = function (...args) {
      const result = reset.apply(this, args);
      rows.length = 0; inputs.length = 0; presentation = 0; bytes = 0;
      truncated = false; inputTruncated = false; error = null;
      return result;
    };
    value.step = function (...args) {
      const result = step.apply(this, args), count = args[0] ?? 1;
      presentation += count;
      try {
        const command = args[1] === undefined ? {type: 'tick', count} : {type: 'tick', count, timestampMillis: args[1]};
        const row = project(result, command), size = new TextEncoder().encode(JSON.stringify(row)).length;
        if (rows.length === LIMIT || bytes + size > MAX_BYTES) truncated = true;
        else { rows.push(row); bytes += size; }
      } catch (failure) { error ??= String(failure.stack ?? failure); }
      return result;
    };
  }});
  for (const type of ['keydown', 'keyup']) window.addEventListener(type, event => {
    if (inputs.length === LIMIT) inputTruncated = true;
    else inputs.push({type, code: event.code, repeat: event.repeat, trusted: event.isTrusted});
  }, true);
  window.__tntReturns = {snapshot: () => ({rows: structuredClone(rows), inputs: structuredClone(inputs),
    bytes, maxBytes: MAX_BYTES, truncated, inputTruncated, error})};
}

export function tntObservedBrowser(real, observed, retain) {
  const wrapped = audioObservedBrowser(real, api => { observed.api = api; });
  return {async newContext(options) {
    const context = await wrapped.newContext(options);
    // Independent actual startup preferences, not Native readback. The two
    // init scripts have no dependency: both complete before application boot.
    await context.addInitScript(`localStorage.removeItem('isaac-rebirth.options.audio.v1');\n` +
      `localStorage.setItem('isaac-rebirth.options.v2',${JSON.stringify(JSON.stringify({schema: 2, sfxVolume: '1.0000', musicVolume: '1.0000', fullscreen: false}))});\n` +
      `(${installTntReturns.toString()})();`);
    const newPage = context.newPage.bind(context), close = context.close.bind(context);
    let page;
    context.newPage = async (...args) => {
      page = await newPage(...args);
      page.on('pageerror', error => observed.errors.push({type: 'pageerror', message: String(error), stack: error.stack}));
      page.on('console', message => { if (message.type() === 'error') observed.errors.push({type: 'console', message: message.text(), location: message.location()}); });
      page.on('response', response => { if (response.status() >= 400) observed.errors.push({type: 'http', url: response.url(), status: response.status()}); });
      page.on('requestfailed', request => observed.errors.push({type: 'requestfailed', url: request.url(), failure: request.failure()}));
      return page;
    };
    context.close = async () => {
      try {
        if (!page || page.isClosed()) throw new Error('Missing actual TNT page on close');
        Object.assign(observed, await page.evaluate(() => ({returns: window.__tntReturns.snapshot(),
          trace: window.game.trace(), runtimeAudio: window.game.audio(), feedback: window.game.feedback()})));
      } catch (error) { observed.errors.push({type: 'observerClose', message: String(error.stack ?? error)}); }
      finally {
        try { await close(); }
        catch (error) { observed.errors.push({type: 'audioObserverClose', message: String(error.stack ?? error)}); }
        finally { retain(observed); }
      }
    };
    return context;
  }};
}
