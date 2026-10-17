import type { Command, MusicBackendCommand, State } from './engine';

type Event = { kind: 'blockedMusicBackend'; status: 'fail'; command: MusicBackendCommand; audioTime: number }
  | {kind:'inputConfigurationFailure'; status:'fail'; operation:'load' | 'save'; error:string}
  | { kind: 'unavailableControl'; status: 'fail'; action: number; message: string }
  | { kind: 'platformFailure'; status: 'fail'; operation: 'fullscreen'; error: string }
  | { kind: 'unavailableOption'; status: 'fail'; sequence: number; row: number; message: string }
  | { kind: 'runtimeFailure'; status: 'fail'; command: Command | null; error: string; lastBrowserState: { screen: State['screen']; tick: number } };
const LIMIT = 256;
const messages: Record<number, string> = {
  0: 'Keyboard and controller configuration is not available yet. Esc returns to the menu.',
  3: 'Map opacity is not available yet. Esc returns to the menu.',
  4: 'Fullscreen is not available yet. Esc returns to the menu.',
  5: 'The display filter is not available yet. Esc returns to the menu.',
  6: 'Gamma adjustment is not available yet. Esc returns to the menu.',
  7: 'Popup preferences are not available yet. Esc returns to the menu.',
};

/** Port capability failures are explicit; this is not a native settings store.
 * Expected unavailable actions leave the Options page usable. Unexpected
 * runtime failures pause automatic ticks and offer an actual page reload. */
export class Feedback {
  private readonly box = document.createElement('aside');
  private readonly text = document.createElement('p');
  private readonly dismiss = document.createElement('button');
  private readonly reload = document.createElement('button');
  private lastSequence = -1;
  private active: Event['kind'] | null = null;
  private events: Event[] = [];
  private truncated = false;
  runtimeFailed = false;

  constructor(private readonly canvas: HTMLCanvasElement) {
    this.box.id = 'game-feedback'; this.box.setAttribute('role', 'alert'); this.box.hidden = true;
    this.dismiss.textContent = 'Dismiss'; this.dismiss.type = 'button';
    this.dismiss.onclick = () => { this.box.hidden = true; this.canvas.focus(); };
    this.reload.textContent = 'Reload'; this.reload.type = 'button'; this.reload.hidden = true;
    this.reload.onclick = () => window.location.reload();
    this.box.append(this.text, this.dismiss, this.reload); this.canvas.parentElement!.append(this.box);
  }

  consume(requests: State['optionsMenu']['unavailableRequests'], screen: State['screen']): void {
    for (const {sequence, row} of requests) {
      if (sequence <= this.lastSequence) continue;
      this.lastSequence = sequence;
      const message = messages[row] ?? 'This option is not available yet. Esc returns to the menu.';
      this.record({kind: 'unavailableOption', status: 'fail', sequence, row, message});
      this.show('unavailableOption', message);
    }
    // Navigation dismisses only this visible notice; its failure record stays.
    if (screen !== 'options' && this.active === 'unavailableOption') {
      const focused = this.box.contains(document.activeElement);
      this.box.hidden = true;
      if (focused) this.canvas.focus();
    }
  }

  inputConfigurationFailure(operation: 'load' | 'save', error: string): void {
    this.record({kind:'inputConfigurationFailure',status:'fail',operation,error});
    if (!this.runtimeFailed) this.show('inputConfigurationFailure', operation === 'load'
      ? 'Keyboard bindings could not be loaded. Default bindings are active.'
      : 'Keyboard bindings could not be saved. The bindings in this session still work.');
  }

  consumeControls(actions: number[]): void {
    const names: Record<number,string> = {11:'Drop',12:'Pause',13:'Map',16:'Restart',17:'Fullscreen shortcut',18:'Mute'};
    for (const action of actions) {
      const message = `${names[action] ?? 'This control'} is not available yet.`;
      this.record({kind:'unavailableControl',status:'fail',action,message});
      if (!this.runtimeFailed) this.show('unavailableControl',message);
    }
  }

  platformFailure(operation: 'fullscreen', error: string): void {
    this.record({kind: 'platformFailure', status: 'fail', operation, error});
    // A late host rejection must not replace the halted game's Reload UI.
    if (this.runtimeFailed) return;
    this.show('platformFailure', 'Fullscreen could not be changed. You can keep using the menu.');
  }

  runtimeFailure(command: Command | null, error: unknown, state: State): void {
    this.runtimeFailed = true;
    this.record({kind: 'runtimeFailure', status: 'fail', command: command === null ? null : structuredClone(command),
      error: String(error), lastBrowserState: {screen: state.screen, tick: state.tick}});
    this.show('runtimeFailure', 'The game stopped after an error. Reload to restart.');
    console.error('Game runtime failure', error);
  }

  blockedMusicBackend(command: MusicBackendCommand, audioTime: number): void {
    //A real backend completion remains observable after simulation halts.
    //Do not dispatch it, manufacture a success, or replace the fatal Reload UI.
    this.record({kind: 'blockedMusicBackend', status: 'fail', command: structuredClone(command), audioTime});
  }

  resetDispatch(): void {
    const focused = this.box.contains(document.activeElement);
    this.lastSequence = -1; this.runtimeFailed = false; this.active = null; this.box.hidden = true;
    if (focused) this.canvas.focus();
  }

  get visible(): boolean { return !this.box.hidden; }
  isInteractiveTarget(target: EventTarget | null): boolean { return target instanceof Node && this.box.contains(target); }

  snapshot() { return {events: structuredClone(this.events), truncated: this.truncated, runtimeFailed: this.runtimeFailed,
    visible: !this.box.hidden, message: this.text.textContent,
    scope: 'Actual port capability/platform/runtime failure notices; unavailable native callbacks remain unimplemented and recorded as failures'}; }

  private show(kind: Event['kind'], message: string): void {
    this.active = kind; this.text.textContent = message; this.box.hidden = false;
    this.reload.hidden = kind !== 'runtimeFailure'; this.dismiss.hidden = kind === 'runtimeFailure';
    if (kind === 'runtimeFailure') this.reload.focus();
  }
  private record(event: Event): void {
    if (this.events.length === LIMIT) this.truncated = true;
    else this.events.push(event);
  }
}
