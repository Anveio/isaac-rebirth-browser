import './style.css';
import { Engine, type Command, type State, type ResourceInputs, type AudioPreferences, type FullscreenPreferences, type InputConfigs, type BootElapsed } from './engine';
import { Renderer } from './renderer';
import { ReplayTrace } from './trace';
import { AudioRuntime } from './audio';
import { Preferences } from './preferences';
import { InputConfigIO, validateInputConfigs } from './input-configs';
import { Feedback } from './feedback';
import { FullscreenRuntime } from './fullscreen';
import { SaveGameIO } from './save-games';

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
  const inputConfigIO = new InputConfigIO();
  let bootInputConfigs = inputConfigIO.load();
  const feedback = new Feedback(canvas);
  const reportInputConfigLoadFailure = () => {
    const event = inputConfigIO.snapshot().events.at(-1);
    if (event?.operation === 'load' && event.status === 'fail') feedback.inputConfigurationFailure('load',event.error ?? 'Input configuration load failed');
  };
  reportInputConfigLoadFailure();
  const savedPreferences = preferences.load();
  let bootAudioPreferences = savedPreferences?.audio ?? null;
  let bootFullscreenPreferences = savedPreferences?.fullscreen ?? null;
  //Real saved settings precede the first Title Music admission, as5B9930
  //precedes MenuInit. No post-constructor command may discard its Start.
  const [engine, renderer, audio] = await Promise.all([Engine.create(initialSeed, bootEpochSeed, undefined, bootAudioPreferences ?? undefined, bootFullscreenPreferences ?? undefined, bootInputConfigs ?? undefined), Renderer.load(canvas), AudioRuntime.load()]);
  const saveGames = new SaveGameIO();
  let saveFailureVisible = false;
  const reportSaveFailure = () => {
    saveFailureVisible = true;
    status.textContent = 'The saved profile could not be loaded or saved.';
    status.hidden = false;
    setTimeout(() => { status.hidden = true; }, 2400);
  };
  if (!saveGames.load(engine)) reportSaveFailure();
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
      let state = engine.command(command);
      //A batched public Tick can stop before a System iteration for real IO.
      //No rendering, audio callback or next public input sees partial Main state.
      for (let responses = 0; ; responses++) {
        if (!state.saveWritePending && !state.saveReadmissionPending) break;
        //One write and one menu read can belong to one public System interval.
        const maxResponses = command.type === 'tick' ? command.count * 2 : 1;
        if (responses >= maxResponses) throw new Error('Save IO made no bounded tick progress');
        if (state.saveWritePending) {
          if (!saveGames.consume(engine, state.saveRevision)) reportSaveFailure();
        } else if (!saveGames.readmit(engine, state.saveRevision)) reportSaveFailure();
        state = engine.snapshot();
      }
      currentState = state;
      feedback.consume(state.optionsMenu.unavailableRequests, state.screen);
      //Settings5B8DB0 IO precedes SetMenu's audio confirmation. Back remains
      //usable after a real storage error; its actual failure stays observable.
      if (!preferences.consume(state.optionsMenu.saveRequests)) {
        status.textContent = 'Preferences could not be saved.'; status.hidden = false;
        setTimeout(() => { status.hidden = true; }, 2400);
      }
      if (!inputConfigIO.consume(state.controlsMenu.saveRequests)) {
        const event = inputConfigIO.snapshot().events.at(-1);
        feedback.inputConfigurationFailure('save',event?.error ?? 'Input configuration save failed');
        status.textContent = 'Keyboard bindings could not be saved.'; status.hidden = false;
        setTimeout(() => { status.hidden = true; }, 2400);
      }
      feedback.consumeControls(state.controlsMenu.unavailableActions);
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
  // Only the focused game captures raw keys/chords. Native key identities are
  // preserved, including both sides of Control/Alt/Meta; browser/OS shortcuts
  // outside the canvas retain their default behavior.
  function releasePressedKeys() {
    if (feedback.runtimeFailed) return;
    for (const code of [...currentState.pressed])
      if (supported.has(code)) dispatch({ type: 'key', code, down: false });
  }
  function keyboard(event: KeyboardEvent, down: boolean) {
    if (feedback.runtimeFailed) return;
    const held = supported.has(event.code) && currentState.pressed.includes(event.code);
    const noticeTarget = feedback.isInteractiveTarget(event.target);
    if ((noticeTarget || (feedback.visible && event.code === 'Tab')) && event.code !== 'Escape') {
      // Notice buttons keep their native keyboard behavior. An actual game
      // release still reaches its owner after focus moves to a notice.
      if (!down && held) {
        try { dispatch({ type: 'key', code: event.code, down: false }); }
        catch (error) { status.textContent = String(error); status.hidden = false; }
      }
      return;
    }
    // Preserve the existing notice Escape/Back recovery even when its Dismiss
    // button owns focus. Other game keydowns require the actual canvas focus.
    const noticeEscape = feedback.visible && event.code === 'Escape';
    if (down && document.activeElement !== canvas && !noticeEscape) return;
    // Releases belong to the key's real pressed owner, regardless of focus or
    // modifier flags. Unowned browser keyups must not be captured by the game.
    if (!down && !held) return;
    const seedText = currentState.screen === 'seed' && /^[a-z0-9]$/i.test(event.key)
      && !event.ctrlKey && !event.altKey && !event.metaKey;
    if (!supported.has(event.code) && !seedText) return;
    event.preventDefault();
    if (down && event.isTrusted) { void audio.unlock().catch(console.error); fullscreen.gesture(event); }
    if (down && event.repeat) return;
    try {
      if (down && seedText) dispatch({ type: 'text', text: event.key.toUpperCase() });
      else dispatch({ type: 'key', code: event.code, down });
    } catch (error) {
      status.textContent = String(error);
      status.hidden = false;
      setTimeout(() => { status.hidden = true; }, 1600);
    }
  }
  window.addEventListener('keydown', (event) => keyboard(event, true));
  window.addEventListener('keyup', (event) => keyboard(event, false));
  canvas.addEventListener('blur', releasePressedKeys);
  window.addEventListener('blur', () => {
    lifecyclePaused = true;
    accumulator = 0;
    releasePressedKeys();
  });
  window.addEventListener('focus', () => {
    lifecyclePaused = document.hidden;
    accumulator = 0;
    previous = performance.now();
  });
  document.addEventListener('visibilitychange', () => {
    lifecyclePaused = document.hidden || !document.hasFocus();
    if (document.hidden) releasePressedKeys();
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
    inputConfigs: () => inputConfigIO.snapshot(),
    feedback: () => feedback.snapshot(),
    // The engine's own JSON text: cheap for automation to transfer and parse.
    snapshotJson: () => engine.snapshotJson(),
    rendererTargets: () => renderer.debugTargets(),
    audio: () => audio.snapshot(),
    fullscreen: () => fullscreen.snapshot(),
    saveGames: () => saveGames.snapshot(),
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
    reset: (seed = randomSeed(), options: { trace?: boolean; bootEpochSeed?: number; bootElapsed?: BootElapsed; resources?: ResourceInputs; audioPreferences?: AudioPreferences; fullscreenPreferences?: FullscreenPreferences; inputConfigs?: InputConfigs } = {}) => {
      fullscreen.assertIdleForReset();
      preferences.resetDispatch();
      inputConfigIO.resetDispatch();
      const loadedInputConfigs = options.inputConfigs === undefined ? inputConfigIO.load() : validateInputConfigs(options.inputConfigs);
      const stored = preferences.load();
      const loadedAudioPreferences = options.audioPreferences ?? stored?.audio ?? null;
      // Reused documents receive a real source-default exit request when no
      // saved or independently declared Fullscreen input exists.
      const loadedFullscreenPreferences = options.fullscreenPreferences ?? stored?.fullscreen ?? {fullscreen: false};
      currentState = engine.reset(seed, options.bootEpochSeed ?? 0, options.resources, loadedAudioPreferences ?? undefined,
        loadedFullscreenPreferences, loadedInputConfigs ?? undefined, options.bootElapsed);
      if (!saveGames.load(engine)) reportSaveFailure();
      currentState = engine.snapshot();
      feedback.resetDispatch();
      if (options.inputConfigs === undefined) reportInputConfigLoadFailure();
      fullscreen.reset(currentState.fullscreen.requests);
      //Retain the validated immutable input, never a caller-owned live object.
      bootInputConfigs = loadedInputConfigs === null ? null : structuredClone(loadedInputConfigs);
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
    trace: () => ({...trace.snapshot(), bootInputConfigs: bootInputConfigs === null ? null : structuredClone(bootInputConfigs), bootAudioPreferences: bootAudioPreferences === null ? null : {...bootAudioPreferences},
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
  status.hidden = !saveFailureVisible;
  document.body.dataset.ready = 'true';
  canvas.focus();
  requestAnimationFrame(frame);
}
boot().catch((error) => {
  document.body.dataset.ready = 'failed';
  status.textContent = String(error);
  console.error(error);
});
