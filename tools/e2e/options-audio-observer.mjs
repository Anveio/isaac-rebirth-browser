import {installBrowserAudioObserver} from './audio-api-observer.mjs';
import {installOptionsGainObserver} from './options-gain-observer.mjs';

// Observe actual public returns already produced by the canonical capture.
// This retains tiny request/Back witnesses beyond the full-history cap without
// adding engine calls, changing arguments or scheduling another presentation.
function installOptionsReturns() {
  const LIMIT = 4096, rows = [];
  let truncated = false, error = null, presentation = 0, previousScreen;
  const record = (state, command) => {
    try {
      if (rows.length === LIMIT) { truncated = true; return; }
      rows.push({presentation, command, beforeScreen: previousScreen, screen: state.screen,
        page: state.menuPresentation.page, tick: state.tick});
      previousScreen = state.screen;
    } catch (failure) { error ??= String(failure.stack ?? failure); }
  };
  let game;
  Object.defineProperty(window, 'game', {configurable: true, enumerable: true,
    get: () => game,
    set(value) {
      game = value;
      const reset = value.reset, step = value.step;
      value.reset = function (...args) {
        const state = reset.apply(this, args);
        rows.length = 0; presentation = 0; previousScreen = state.screen;
        truncated = false; error = null;
        return state;
      };
      value.step = function (...args) {
        const state = step.apply(this, args);
        const count = args[0] ?? 1;
        presentation += count;
        const command = args[1] === undefined ? {type: 'tick', count}
          : {type: 'tick', count, timestampMillis: args[1]};
        record(state, command);
        return state;
      };
    }});
  window.__optionsReturns = {snapshot: () => ({rows: structuredClone(rows), truncated, error})};
  const events = [];
  let inputTruncated = false;
  for (const type of ['keydown', 'keyup']) window.addEventListener(type, event => {
    if (events.length === LIMIT) inputTruncated = true;
    else events.push({type, code: event.code, repeat: event.repeat, trusted: event.isTrusted});
  }, true);
  window.__optionsTypedInput = {snapshot: () => ({events: structuredClone(events), truncated: inputTruncated})};
}

export function optionsObservedBrowser(real, data, observed, retain) {
  const labels = data.pairedTape.filter(action => action.type === 'capture').map(action => action.label);
  return new Proxy(real, {get(target, key) {
    if (key !== 'newContext') {
      const value = Reflect.get(target, key);
      return typeof value === 'function' ? value.bind(target) : value;
    }
    return async (...args) => {
      const context = await target.newContext(...args);
      // Preferences are independent public startup inputs through the real IO.
      // One init script defines API-before-gain forwarding order explicitly.
      const initial = {schema: 2, sfxVolume: '1.0000', musicVolume: '1.0000', fullscreen: false};
      await context.addInitScript({content:
        `localStorage.removeItem('isaac-rebirth.options.audio.v1');\n` +
        `localStorage.setItem('isaac-rebirth.options.v2',${JSON.stringify(JSON.stringify(initial))});\n` +
        `(${installBrowserAudioObserver.toString()})();\n` +
        `(${installOptionsGainObserver.toString()})();\n` +
        `(${installOptionsReturns.toString()})();\n`});
      const newPage = context.newPage.bind(context), close = context.close.bind(context);
      let page, next = 0;
      context.newPage = async (...pageArgs) => {
        page = await newPage(...pageArgs);
        page.on('pageerror', error => observed.errors.push({type: 'pageerror', message: String(error), stack: error.stack}));
        page.on('console', event => { if (event.type() === 'error') observed.errors.push({type: 'console', message: event.text(), location: event.location()}); });
        page.on('response', response => { if (response.status() >= 400) observed.errors.push({type: 'httpError', url: response.url(), status: response.status()}); });
        page.on('requestfailed', request => observed.errors.push({type: 'requestfailed', url: request.url(), failure: request.failure()}));
        const evaluate = page.evaluate.bind(page);
        page.evaluate = async (fn, arg) => {
          const result = await evaluate(fn, arg);
          if (typeof fn === 'function' && /window\.game\.capture\(\)/.test(fn.toString())) {
            const actual = await evaluate(() => ({preferences: window.game.preferences(), feedback: window.game.feedback(),
              storage: localStorage.getItem('isaac-rebirth.options.v2'), apiEvents: window.__audioApi.events.length,
              gainEvents: window.__optionsGainObservation.snapshot().events.length}));
            observed.checkpoints.push({label: labels[next++], ...actual});
          }
          return result;
        };
        return page;
      };
      context.close = async () => {
        try {
          if (page && !page.isClosed()) Object.assign(observed, await page.evaluate(async () => {
            const settled = await Promise.allSettled(window.__audioApi.pending);
            return {trace: window.game.trace(), returns: window.__optionsReturns.snapshot(),
              api: {events: window.__audioApi.events, truncated: window.__audioApi.truncated, contexts: window.__audioApi.contextTimes()},
              gain: window.__optionsGainObservation.snapshot(), input: window.__optionsTypedInput.snapshot(),
              feedback: window.game.feedback(), preferences: window.game.preferences(),
              failures: settled.filter(row => row.status === 'rejected').map(row => String(row.reason))};
          }));
        } catch (error) { observed.failures.push(String(error.stack ?? error)); }
        finally { retain(observed); await close(); }
      };
      return context;
    };
  }});
}
