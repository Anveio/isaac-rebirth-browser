// Passive owners around the canonical public capture; no extra game commands.
import {audioObservedBrowser} from './lib/audio-owners.mjs';
import {installPhysicalObserver} from '../foundation/controls-browser-observe.mjs';
import {evaluateJson, readTrace} from './lib/game.mjs';

function installMapReturns() {
  const rows = [], limit = 4096;
  let truncated = false, error = null, presentation = 0, game;
  Object.defineProperty(window, 'game', {configurable: true, enumerable: true,
    get: () => game,
    set(value) {
      game = value;
      const reset = value.reset, step = value.step;
      value.reset = function(...args) {
        const state = reset.apply(this, args);
        rows.length = 0; truncated = false; error = null; presentation = 0;
        return state;
      };
      value.step = function(...args) {
        const state = step.apply(this, args);
        try {
          if ((args[0] ?? 1) !== 1) throw new Error('Map public clock requires individual actual steps');
          presentation++;
          if (rows.length === limit) truncated = true;
          else rows.push({presentation, timestampMillis: args[1], tick: state.tick, screen: state.screen,
            systemCounter: state.systemClock.systemCounter, gameUpdateCount: state.gameUpdateCount,
            opacity: state.mapSettings.opacity, options: structuredClone(state.optionsMenu),
            map: structuredClone(state.minimapPresentation), audioEvents: structuredClone(state.audio.events),
            slot237: structuredClone(state.audio.slots['237'])});
        } catch (failure) { error ??= String(failure.stack ?? failure); }
        return state;
      };
    }});
  window.__mapActualReturns = () => ({rows: structuredClone(rows), truncated, error});
}

export function mapObservedBrowser(real, data, contract, observed, retain) {
  const audio = audioObservedBrowser(real, value => { observed.api = value; });
  return {async newContext(...args) {
    const context = await audio.newContext(...args);
    // Fresh-origin actual absence is a public startup declaration. No Map value
    // is injected, including on reset; the runtime must use its real default.
    await context.addInitScript({content:
      `for (const key of ['isaac-rebirth.options.audio.v1','isaac-rebirth.options.v2','isaac-rebirth.options.v3']) localStorage.removeItem(key);\n` +
      `(${installPhysicalObserver.toString()})();\n(${installMapReturns.toString()})();`});
    const newPage = context.newPage.bind(context), close = context.close.bind(context);
    let page, next = 0;
    context.newPage = async (...pageArgs) => {
      page = await newPage(...pageArgs);
      page.on('pageerror', error => observed.errors.push({type: 'pageerror', message: String(error), stack: error.stack}));
      page.on('console', event => {if (event.type() === 'error') observed.errors.push({type: 'console', message: event.text()});});
      page.on('response', response => {if (response.status() >= 400) observed.errors.push({type: 'http', url: response.url(), status: response.status()});});
      page.on('requestfailed', request => observed.errors.push({type: 'requestfailed', url: request.url(), failure: request.failure()}));
      // Canonical capture may use either Playwright object transport or the
      // exact-JSON evaluateHandle helper. Observe the actual call identically;
      // return the original result/handle without changing commands or timing.
      for (const method of ['evaluate', 'evaluateHandle']) {
        const evaluate = page[method].bind(page);
        page[method] = async (fn, arg) => {
          const result = await evaluate(fn, arg);
          if (typeof fn === 'function' && /window\.game\.capture\(\)/.test(fn.toString())) {
            const actual = await evaluateJson(page, storageKey => ({preferences: window.game.preferences(),
              feedback: window.game.feedback(), storage: {key: storageKey, text: localStorage.getItem(storageKey)},
              audio: window.game.audio()}), contract.storageKey);
            observed.checkpoints.push({label: contract.pairedLabels[next++], ...actual});
          }
          return result;
        };
      }
      return page;
    };
    context.close = async () => {
      try {
        if (!page || page.isClosed()) throw new Error('Missing actual canonical Map page');
        observed.trace = await readTrace(page);
        Object.assign(observed, await evaluateJson(page, () => ({returns: window.__mapActualReturns(),
          physical: window.__controlsPhysicalInputs(), preferences: window.game.preferences(),
          feedback: window.game.feedback()})));
      } catch (error) {observed.failures.push(String(error.stack ?? error));}
      finally {
        try {await close();} catch (error) {observed.failures.push(String(error.stack ?? error));}
        retain(observed);
      }
    };
    return context;
  }};
}
