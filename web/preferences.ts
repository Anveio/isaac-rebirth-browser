import type { State, AudioPreferences, FullscreenPreferences } from './engine';

const KEY = 'isaac-rebirth.options.v2';
const LEGACY_KEY = 'isaac-rebirth.options.audio.v1';
const LIMIT = 256;
type Values = {audio: AudioPreferences; fullscreen: FullscreenPreferences};
type SavedV1 = {schema: 1; sfxVolume: string; musicVolume: string};
type SavedV2 = {schema: 2; sfxVolume: string; musicVolume: string; fullscreen: boolean};
type Event = {operation: 'load' | 'save'; status: 'pass' | 'absent' | 'fail'; key?: string;
  sequence?: number; value?: SavedV1 | SavedV2; error?: string};

/** Origin-local IO for admitted owners only. Fullscreen serializes source
 * requested intent, never an acknowledgement or desktop geometry. */
export class Preferences {
  private storage: Storage | null = null;
  private lastSave = -1;
  private events: Event[] = [];
  private truncated = false;
  constructor() {
    try { this.storage = window.localStorage; }
    catch (error) { this.failure('load', error); }
  }

  load(): Values | null {
    try {
      if (!this.storage) throw new Error('Preference storage is unavailable');
      const current = this.storage.getItem(KEY);
      const key = current === null ? LEGACY_KEY : KEY;
      const text = current === null ? this.storage.getItem(LEGACY_KEY) : current;
      if (text === null) { this.record({operation: 'load', status: 'absent', key: KEY}); return null; }
      if (text.length > 1024) throw new Error('Saved preferences exceed the admitted format');
      const value: unknown = JSON.parse(text);
      if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error('Invalid saved preferences');
      const saved = value as Record<string, unknown>;
      const legacy = key === LEGACY_KEY;
      const keys = legacy ? 'musicVolume,schema,sfxVolume' : 'fullscreen,musicVolume,schema,sfxVolume';
      if (Object.keys(saved).sort().join(',') !== keys || saved.schema !== (legacy ? 1 : 2) ||
          (!legacy && typeof saved.fullscreen !== 'boolean'))
        throw new Error('Unsupported saved preference schema');
      const volume = (field: unknown): number => {
        if (typeof field !== 'string' || !/^\d\.\d{4}$/.test(field)) throw new Error('Invalid serialized audio volume');
        const number = Number(field);
        if (!Number.isFinite(number) || number < 0 || number > 1) throw new Error('Saved volume is outside 0..1');
        return Math.fround(number);
      };
      const loaded = {audio: {sfxVolume: volume(saved.sfxVolume), musicVolume: volume(saved.musicVolume)},
        fullscreen: {fullscreen: legacy ? false : saved.fullscreen as boolean}};
      this.record({operation: 'load', status: 'pass', key, value: saved as SavedV1 | SavedV2});
      return loaded;
    } catch (error) {
      //Invalid v2 does not silently fall back to an older different request.
      //Real source defaults remain active; the IO failure stays observable.
      this.failure('load', error); return null;
    }
  }

  consume(requests: State['optionsMenu']['saveRequests']): boolean {
    let success = true;
    for (const request of requests) {
      if (!Number.isSafeInteger(request.sequence) || request.sequence < 0) {
        success = false; this.failure('save', 'Invalid source save sequence'); continue;
      }
      if (request.sequence <= this.lastSave) continue;
      this.lastSave = request.sequence;
      try {
        if (!this.storage) throw new Error('Preference storage is unavailable');
        if (typeof request.fullscreen !== 'boolean') throw new Error('Missing source fullscreen save owner');
        const value: SavedV2 = {schema: 2, sfxVolume: request.sfxVolume, musicVolume: request.musicVolume,
          fullscreen: request.fullscreen};
        const text = JSON.stringify(value);
        this.storage.setItem(KEY, text);
        if (this.storage.getItem(KEY) !== text) throw new Error('Preference save readback differs from the actual write');
        this.record({operation: 'save', status: 'pass', key: KEY, sequence: request.sequence, value});
      } catch (error) { success = false; this.failure('save', error, request.sequence); }
    }
    return success;
  }

  resetDispatch(): void { this.lastSave = -1; }
  snapshot() { return {key: KEY, legacyKey: LEGACY_KEY, storageAvailable: this.storage !== null,
    events: structuredClone(this.events), truncated: this.truncated,
    scope: 'Actual origin-local audio/requested-fullscreen preference IO; OS window geometry and complete native Options.ini persistence are not claimed'}; }
  private record(event: Event): void {
    if (this.events.length === LIMIT) this.truncated = true;
    else this.events.push(event);
  }
  private failure(operation: Event['operation'], error: unknown, sequence?: number): void {
    const message = String(error);
    this.record({operation, status: 'fail', ...(sequence === undefined ? {} : {sequence}), error: message});
    console.error(`Preference ${operation} failed`, error);
  }
}
