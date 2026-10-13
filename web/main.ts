import './style.css';
import { Engine, type Command } from './engine';
import { Renderer } from './renderer';
import { ReplayTrace } from './trace';
import { AudioRuntime } from './audio';

const canvas = document.querySelector<HTMLCanvasElement>('#game')!;
const status = document.querySelector<HTMLElement>('#status')!;

async function boot() {
  const randomSeed = () => {
    // The original reserves zero rather than accepting it as a run seed.
    let value = 0;
    while (value === 0) value = crypto.getRandomValues(new Uint32Array(1))[0]!;
    return value;
  };
  let initialSeed = randomSeed();
  const bootEpochSeed = Math.floor(Date.now() / 1000) >>> 0;
  const [engine, renderer, audio] = await Promise.all([Engine.create(initialSeed, bootEpochSeed), Renderer.load(canvas), AudioRuntime.load()]);
  let automaticPaused = false;
  let lifecyclePaused = !document.hasFocus() || document.hidden;
  let accumulator = 0;
  let previous = performance.now();
  let currentState = engine.snapshot();
  let trace = new ReplayTrace(initialSeed, false, currentState);
  const dispatch = (command: Command, render = true) => {
    const state = engine.command(command);
    currentState = state;
    audio.consume(state.audio.events);
    trace.record(command, state);
    if (render) renderer.draw(state);
    return state;
  };
  const supported = new Set(['Enter', 'Escape', 'Tab', 'Backspace', 'ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'KeyW', 'KeyA', 'KeyS', 'KeyD', 'KeyE']);
  function keyboard(event: KeyboardEvent, down: boolean) {
    if (down && event.isTrusted) void audio.unlock().catch(console.error);
    if (down && (event.ctrlKey || event.altKey || event.metaKey)) return;
    const seedText = currentState.screen === 'seed' && /^[a-z0-9]$/i.test(event.key);
    if (!supported.has(event.code) && !seedText) return;
    event.preventDefault();
    if (down && event.repeat) return;
    try {
      if (seedText) {
        if (down) dispatch({ type: 'text', text: event.key.toUpperCase() });
        else if (supported.has(event.code) && currentState.pressed.includes(event.code))
          dispatch({ type: 'key', code: event.code, down: false });
      } else dispatch({ type: 'key', code: event.code, down });
    } catch (error) {
      status.textContent = String(error);
      status.hidden = false;
      setTimeout(() => { status.hidden = true; }, 1600);
    }
  }
  window.addEventListener('keydown', (event) => keyboard(event, true));
  window.addEventListener('keyup', (event) => keyboard(event, false));
  window.addEventListener('blur', () => {
    for (const code of supported) dispatch({ type: 'key', code, down: false });
    lifecyclePaused = true;
    accumulator = 0;
  });
  window.addEventListener('focus', () => {
    lifecyclePaused = document.hidden;
    accumulator = 0;
    previous = performance.now();
  });
  document.addEventListener('visibilitychange', () => {
    lifecyclePaused = document.hidden || !document.hasFocus();
    accumulator = 0;
    previous = performance.now();
  });
  canvas.addEventListener('pointerdown', (event) => {
    canvas.focus();
    if (event.isTrusted) void audio.unlock().catch(console.error);
  });
  function frame(now: number) {
    const elapsed = Math.min(now - previous, 250);
    previous = now;
    if (!automaticPaused && !lifecyclePaused) {
      accumulator += elapsed;
      while (accumulator >= 1000 / 60) {
        dispatch({ type: 'tick', count: 1 }, false);
        accumulator -= 1000 / 60;
      }
    }
    renderer.draw(currentState);
    requestAnimationFrame(frame);
  }
  const game = {
    snapshot: () => engine.snapshot(),
    rendererTargets: () => renderer.debugTargets(),
    audio: () => audio.snapshot(),
    command: (command: Command) => structuredClone(dispatch(command)),
    step: (count = 1) => structuredClone(dispatch({ type: 'tick', count })),
    pause: () => { automaticPaused = true; accumulator = 0; },
    resume: () => { automaticPaused = false; accumulator = 0; previous = performance.now(); },
    reset: (seed = randomSeed(), options: { trace?: boolean; bootEpochSeed?: number } = {}) => {
      currentState = engine.reset(seed, options.bootEpochSeed ?? 0);
      audio.reset(options.trace === true);
      initialSeed = seed;
      trace = new ReplayTrace(initialSeed, options.trace === true, currentState);
      accumulator = 0;
      renderer.draw(currentState);
      return engine.snapshot();
    },
    trace: () => trace.snapshot(),
    capture: () => {
      const start = performance.now();
      const state = engine.snapshot();
      const serialized = performance.now();
      renderer.draw(state);
      const rendered = performance.now();
      const output = document.createElement('canvas');
      output.width = 960; output.height = 540;
      const context = output.getContext('2d')!;
      context.imageSmoothingEnabled = false;
      context.drawImage(canvas, 0, 0, output.width, output.height);
      const png = output.toDataURL('image/png').split(',')[1];
      return { state, png, timingMs: { snapshot: serialized - start, render: rendered - serialized, captureEncoding: performance.now() - rendered }, timingScope: 'Snapshot; WebGL draw and finish plus Canvas copy submission; PNG encoding. Excludes simulation, deferred Canvas raster work before encoding, and browser presentation.' };
    },
  };
  Object.assign(window, { game });
  renderer.draw(currentState);
  status.hidden = true;
  document.body.dataset.ready = 'true';
  canvas.focus();
  requestAnimationFrame(frame);
}
boot().catch((error) => {
  document.body.dataset.ready = 'failed';
  status.textContent = String(error);
  console.error(error);
});
