import './style.css';
import { Engine, type Command, type State, type ResourceInputs, type AudioPreferences, type FullscreenPreferences } from './engine';
import { Renderer } from './renderer';
import { ReplayTrace } from './trace';
import { AudioRuntime } from './audio';
import { Preferences } from './preferences';
import { Feedback } from './feedback';
import { FullscreenRuntime } from './fullscreen';

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
  const preferences = new Preferences();
  const feedback = new Feedback(canvas);
  const savedPreferences = preferences.load();
  let bootAudioPreferences = savedPreferences?.audio ?? null;
  let bootFullscreenPreferences = savedPreferences?.fullscreen ?? null;
  //Real saved settings precede the first Title Music admission, as5B9930
  //precedes MenuInit. No post-constructor command may discard its Start.
  const [engine, renderer, audio] = await Promise.all([Engine.create(initialSeed, bootEpochSeed, undefined, bootAudioPreferences ?? undefined, bootFullscreenPreferences ?? undefined), Renderer.load(canvas), AudioRuntime.load()]);
  let automaticPaused = false;
  let lifecyclePaused = !document.hasFocus() || document.hidden;
  let accumulator = 0;
  let previous = performance.now();
  let currentState = engine.snapshot();
  const fullscreen = new FullscreenRuntime(canvas.closest('main')!, message => feedback.platformFailure('fullscreen', message));
  let trace = new ReplayTrace(initialSeed, false, currentState);
  // Drawing is a pure function of state, so the frame loop only redraws after
  // the state changes; a paused or idle game does not re-render every frame.
  let drawnState: State | null = null;
  const draw = (state: State) => { renderer.draw(state); drawnState = state; };
  const dispatch = (command: Command, render = true, backendAudioTime?: number) => {
    if (feedback.runtimeFailed) throw new Error('Reload or reset after the recorded runtime failure');
    const previousAmbienceUpdate = currentState.ambience.updateSequence;
    try {
      const state = engine.command(command);
      currentState = state;
      feedback.consume(state.optionsMenu.unavailableRequests, state.screen);
      //Settings5B8DB0 IO precedes SetMenu's audio confirmation. Back remains
      //usable after a real storage error; its actual failure stays observable.
      if (!preferences.consume(state.optionsMenu.saveRequests)) {
        status.textContent = 'Preferences could not be saved.'; status.hidden = false;
        setTimeout(() => { status.hidden = true; }, 2400);
      }
      fullscreen.consume(state.fullscreen.requests);
      audio.consume(state.audio.events, state.audio.volumeCallbacks);
      //Only an actual source Room.Update reissues ambience volume/pitch.
      //Pure key/backend/volume commands must not overwrite5E1DB0's retune.
      if (state.ambience.updateSequence !== previousAmbienceUpdate)
        audio.syncAmbience(state.ambience, state.audio.sfxVolume);
      audio.music.consume(state.music);
      if (backendAudioTime === undefined) trace.record(command, state);
      else if (command.type === 'musicBackend') trace.recordBackend(command, state, backendAudioTime);
      else throw new Error('Backend event metadata requires an actual music callback');
      if (render) draw(state);
      return state;
    } catch (error) {
      // Retain the actual failed dispatch separately. The partial engine
      // state is not rolled back, retried or silently resumed.
      automaticPaused = true; accumulator = 0;
      feedback.runtimeFailure(command, error, currentState);
      throw error;
    }
  };
  fullscreen.consume(currentState.fullscreen.requests);
  audio.music.onBackend((command, audioTime) => {
    if (feedback.runtimeFailed) {
      feedback.blockedMusicBackend(command, audioTime);
      return;
    }
    try { dispatch(command, false, audioTime); }
    catch (error) {
      //dispatch retains its real failure and halts; an asynchronous callback
      //must not create a second uncaught error or resume the failed simulation.
      if (!feedback.runtimeFailed) feedback.runtimeFailure(command, error, currentState);
    }
  });
  audio.music.consume(currentState.music);
  const supported = engine.publicKeys;
  function keyboard(event: KeyboardEvent, down: boolean) {
    // Native UI keys must reach the real notice buttons. A healthy game's
    // already-held key still receives its actual release after focus changes.
    if (feedback.runtimeFailed) return;
    const noticeTarget = feedback.isInteractiveTarget(event.target);
    if ((noticeTarget || (feedback.visible && event.code === 'Tab')) && event.code !== 'Escape') {
      if (!down && supported.has(event.code) && currentState.pressed.includes(event.code)) {
        try { dispatch({ type: 'key', code: event.code, down: false }); }
        catch (error) { status.textContent = String(error); status.hidden = false; }
      }
      return;
    }
    if (down && event.isTrusted) { void audio.unlock().catch(console.error); fullscreen.gesture(event); }
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
    lifecyclePaused = true;
    accumulator = 0;
    if (!feedback.runtimeFailed)
      for (const code of supported) dispatch({ type: 'key', code, down: false });
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
    if (event.isTrusted && !feedback.runtimeFailed) { void audio.unlock().catch(console.error); fullscreen.gesture(event); }
  });
  function frame(now: number) {
    const elapsed = Math.min(now - previous, 250);
    previous = now;
    try {
      if (!automaticPaused && !lifecyclePaused) {
        accumulator += elapsed;
        while (accumulator >= 1000 / 60) {
          dispatch({ type: 'tick', count: 1, timestampMillis: Date.now() }, false);
          accumulator -= 1000 / 60;
        }
      }
      if (!feedback.runtimeFailed && drawnState !== currentState) draw(currentState);
    } catch (error) {
      automaticPaused = true; accumulator = 0;
      if (!feedback.runtimeFailed) feedback.runtimeFailure(null, error, currentState);
    } finally {
      // Unexpected runtime failure remains visible and pauses automatic ticks;
      // the UI and a real Reload action remain alive.
      requestAnimationFrame(frame);
    }
  }
  const game = {
    snapshot: () => engine.snapshot(),
    preferences: () => preferences.snapshot(),
    feedback: () => feedback.snapshot(),
    // The engine's own JSON text: cheap for automation to transfer and parse.
    snapshotJson: () => engine.snapshotJson(),
    rendererTargets: () => renderer.debugTargets(),
    audio: () => audio.snapshot(),
    fullscreen: () => fullscreen.snapshot(),
    command: (command: Command) => structuredClone(dispatch(command)),
    // Automation steps leave drawing to the animation-frame loop (and to
    // capture(), which draws the state it encodes): one software-GL draw per
    // stepped frame dominated live suite time.
    step: (count = 1, timestampMillis?: number) => structuredClone(dispatch(timestampMillis === undefined
      ? { type: 'tick', count } : { type: 'tick', count, timestampMillis }, false)),
    pause: () => { automaticPaused = true; accumulator = 0; },
    resume: () => {
      if (feedback.runtimeFailed) throw new Error('Reload or reset after the recorded runtime failure');
      automaticPaused = false; accumulator = 0; previous = performance.now();
    },
    reset: (seed = randomSeed(), options: { trace?: boolean; bootEpochSeed?: number; resources?: ResourceInputs; audioPreferences?: AudioPreferences; fullscreenPreferences?: FullscreenPreferences } = {}) => {
      fullscreen.assertIdleForReset();
      preferences.resetDispatch();
      const stored = preferences.load();
      const loadedAudioPreferences = options.audioPreferences ?? stored?.audio ?? null;
      // Reused documents receive a real source-default exit request when no
      // saved or independently declared Fullscreen input exists.
      const loadedFullscreenPreferences = options.fullscreenPreferences ?? stored?.fullscreen ?? {fullscreen: false};
      currentState = engine.reset(seed, options.bootEpochSeed ?? 0, options.resources, loadedAudioPreferences ?? undefined,
        loadedFullscreenPreferences);
      feedback.resetDispatch();
      fullscreen.reset(currentState.fullscreen.requests);
      //Retain the validated immutable input, never a caller-owned live object.
      bootAudioPreferences = loadedAudioPreferences === null ? null : {...loadedAudioPreferences};
      bootFullscreenPreferences = {...loadedFullscreenPreferences};
      audio.reset(options.trace === true);
      initialSeed = seed;
      trace = new ReplayTrace(initialSeed, options.trace === true, currentState);
      audio.music.consume(currentState.music);
      accumulator = 0;
      draw(currentState);
      return engine.snapshot();
    },
    trace: () => ({...trace.snapshot(), bootAudioPreferences: bootAudioPreferences === null ? null : {...bootAudioPreferences},
      bootFullscreenPreferences: bootFullscreenPreferences === null ? null : {...bootFullscreenPreferences}}),
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
