import './style.css';
import { Engine, type Command, type State, type ResourceInputs } from './engine';
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
  // Drawing is a pure function of state, so the frame loop only redraws after
  // the state changes; a paused or idle game does not re-render every frame.
  let drawnState: State | null = null;
  const draw = (state: State) => { renderer.draw(state); drawnState = state; };
  const dispatch = (command: Command, render = true, backendAudioTime?: number) => {
    const state = engine.command(command);
    currentState = state;
    audio.consume(state.audio.events);
    audio.syncAmbience(state.ambience, state.audio.sfxVolume);
    audio.music.consume(state.music);
    if (backendAudioTime === undefined) trace.record(command, state);
    else if (command.type === 'musicBackend') trace.recordBackend(command, state, backendAudioTime);
    else throw new Error('Backend event metadata requires an actual music callback');
    if (render) draw(state);
    return state;
  };
  audio.music.onBackend((command, audioTime) => dispatch(command, false, audioTime));
  audio.music.consume(currentState.music);
  const supported = engine.publicKeys;
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
        dispatch({ type: 'tick', count: 1, timestampMillis: Date.now() }, false);
        accumulator -= 1000 / 60;
      }
    }
    if (drawnState !== currentState) draw(currentState);
    requestAnimationFrame(frame);
  }
  const game = {
    snapshot: () => engine.snapshot(),
    // The engine's own JSON text: cheap for automation to transfer and parse.
    snapshotJson: () => engine.snapshotJson(),
    rendererTargets: () => renderer.debugTargets(),
    audio: () => audio.snapshot(),
    command: (command: Command) => structuredClone(dispatch(command)),
    // Automation steps leave drawing to the animation-frame loop (and to
    // capture(), which draws the state it encodes): one software-GL draw per
    // stepped frame dominated live suite time.
    step: (count = 1, timestampMillis?: number) => structuredClone(dispatch(timestampMillis === undefined
      ? { type: 'tick', count } : { type: 'tick', count, timestampMillis }, false)),
    pause: () => { automaticPaused = true; accumulator = 0; },
    resume: () => { automaticPaused = false; accumulator = 0; previous = performance.now(); },
    reset: (seed = randomSeed(), options: { trace?: boolean; bootEpochSeed?: number; resources?: ResourceInputs } = {}) => {
      currentState = engine.reset(seed, options.bootEpochSeed ?? 0, options.resources);
      audio.reset(options.trace === true);
      initialSeed = seed;
      trace = new ReplayTrace(initialSeed, options.trace === true, currentState);
      audio.music.consume(currentState.music);
      accumulator = 0;
      draw(currentState);
      return engine.snapshot();
    },
    trace: () => trace.snapshot(),
    // Diagnostic shared-MT draw log for the e2e --mt-draws comparator.
    mtDraws: { enable: (enabled = true) => engine.setMtDrawLog(enabled), take: () => engine.takeMtDraws() },
    capture: () => {
      const start = performance.now();
      const state = engine.snapshot();
      const serialized = performance.now();
      draw(state);
      const rendered = performance.now();
      const output = document.createElement('canvas');
      output.width = 960; output.height = 540;
      const context = output.getContext('2d')!;
      context.imageSmoothingEnabled = false;
      context.drawImage(canvas, 0, 0, output.width, output.height);
      const png = output.toDataURL('image/png').split(',')[1];
      return { state, png, render: renderer.drawRecord(), timingMs: { snapshot: serialized - start, render: rendered - serialized, captureEncoding: performance.now() - rendered }, timingScope: 'Snapshot; WebGL draw and finish plus Canvas copy submission; PNG encoding. Excludes simulation, deferred Canvas raster work before encoding, and browser presentation.' };
    },
  };
  Object.assign(window, { game });
  draw(currentState);
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
