import './style.css';
import { Engine, type Command } from './engine';
import { Renderer } from './renderer';

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
  const [engine, renderer] = await Promise.all([Engine.create(initialSeed), Renderer.load(canvas)]);
  let automaticPaused = false;
  let accumulator = 0;
  let previous = performance.now();
  const commands: Command[] = [];
  const states = [engine.snapshot()];
  let traceTruncated = false;
  const dispatch = (command: Command, render = true) => {
    const state = engine.command(command);
    if (commands.length < 36_000) {
      commands.push(structuredClone(command));
      states.push(state);
    } else traceTruncated = true;
    if (render) renderer.draw(state);
    return state;
  };
  const supported = new Set(['Enter', 'Escape', 'Tab', 'Backspace', 'ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'KeyW', 'KeyA', 'KeyS', 'KeyD']);
  function keyboard(event: KeyboardEvent, down: boolean) {
    if (down && (event.ctrlKey || event.altKey || event.metaKey)) return;
    const seedText = engine.snapshot().screen === 'seed' && /^[a-z0-9]$/i.test(event.key);
    if (!supported.has(event.code) && !seedText) return;
    event.preventDefault();
    if (down && event.repeat) return;
    try {
      if (seedText) {
        if (down) dispatch({ type: 'text', text: event.key.toUpperCase() });
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
    automaticPaused = true;
  });
  canvas.addEventListener('pointerdown', () => { canvas.focus(); automaticPaused = false; });
  function frame(now: number) {
    const elapsed = Math.min(now - previous, 250);
    previous = now;
    if (!automaticPaused) {
      accumulator += elapsed;
      while (accumulator >= 1000 / 60) {
        dispatch({ type: 'tick', count: 1 }, false);
        accumulator -= 1000 / 60;
      }
    }
    renderer.draw(engine.snapshot());
    requestAnimationFrame(frame);
  }
  const game = {
    snapshot: () => engine.snapshot(),
    rendererTargets: () => renderer.debugTargets(),
    graphicsContracts: () => renderer.debugGraphicsContracts(),
    command: dispatch,
    step: (count = 1) => dispatch({ type: 'tick', count }),
    pause: () => { automaticPaused = true; accumulator = 0; },
    resume: () => { automaticPaused = false; accumulator = 0; previous = performance.now(); },
    reset: (seed = randomSeed()) => {
      engine.reset(seed);
      initialSeed = seed;
      commands.length = 0;
      traceTruncated = false;
      states.splice(0, states.length, engine.snapshot());
      accumulator = 0;
      renderer.draw(engine.snapshot());
      return engine.snapshot();
    },
    trace: () => ({ schema: 1, tickRate: 60, initialSeed, truncated: traceTruncated, commands: structuredClone(commands), states: structuredClone(states) }),
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
  renderer.draw(engine.snapshot());
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
