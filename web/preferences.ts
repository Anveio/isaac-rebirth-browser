import type { State, AudioPreferences, FullscreenPreferences, MapPreferences } from './engine';

const KEY = 'isaac-rebirth.options.v3';
const PREVIOUS_KEY = 'isaac-rebirth.options.v2';
const LEGACY_KEY = 'isaac-rebirth.options.audio.v1';
const LIMIT = 256;
type Values = {audio: AudioPreferences; fullscreen: FullscreenPreferences; map: MapPreferences};
type SavedV1 = {schema: 1; sfxVolume: string; musicVolume: string};
type SavedV2 = {schema: 2; sfxVolume: string; musicVolume: string; fullscreen: boolean};
type SavedV3 = {schema: 3; sfxVolume: string; musicVolume: string; fullscreen: boolean; mapOpacity: string};
type Event = {operation: 'load' | 'save'; status: 'pass' | 'absent' | 'fail'; key?: string;
  sequence?: number; value?: SavedV1 | SavedV2 | SavedV3; error?: string};

/** Origin-local IO of actual owners. Map opacity joins the same Back save;
 * older schemas retain the source .3 default, without rewriting old evidence. */
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
      let key = KEY, text = this.storage.getItem(KEY);
      if (text === null) { key = PREVIOUS_KEY; text = this.storage.getItem(PREVIOUS_KEY); }
      if (text === null) { key = LEGACY_KEY; text = this.storage.getItem(LEGACY_KEY); }
      if (text === null) { this.record({operation: 'load', status: 'absent', key: KEY}); return null; }
      if (text.length > 1024) throw new Error('Saved preferences exceed the admitted format');
      const value: unknown = JSON.parse(text);
      if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error('Invalid saved preferences');
      const saved = value as Record<string, unknown>;
      const version = key === KEY ? 3 : key === PREVIOUS_KEY ? 2 : 1;
      const keys = version === 1 ? 'musicVolume,schema,sfxVolume'
        : version === 2 ? 'fullscreen,musicVolume,schema,sfxVolume'
        : 'fullscreen,mapOpacity,musicVolume,schema,sfxVolume';
      if (Object.keys(saved).sort().join(',') !== keys || saved.schema !== version ||
          (version !== 1 && typeof saved.fullscreen !== 'boolean'))
        throw new Error('Unsupported saved preference schema');
      const number = (field: unknown): number => {
        if (typeof field !== 'string' || !/^\d\.\d{4}$/.test(field)) throw new Error('Invalid serialized preference');
        const parsed = Number(field);
        if (!Number.isFinite(parsed) || parsed < 0 || parsed > 1) throw new Error('Saved preference is outside 0..1');
        return Math.fround(parsed);
      };
      const loaded = {audio: {sfxVolume: number(saved.sfxVolume), musicVolume: number(saved.musicVolume)},
        fullscreen: {fullscreen: version === 1 ? false : saved.fullscreen as boolean},
        map: {opacity: version === 3 ? number(saved.mapOpacity) : Math.fround(0.3)}};
      this.record({operation: 'load', status: 'pass', key, value: saved as SavedV1 | SavedV2 | SavedV3});
      return loaded;
    } catch (error) {
      //An invalid newer record never falls back to a different older request.
      //Actual constructor defaults remain active, and failure stays observable.
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
        if (typeof request.mapOpacity !== 'string' || !/^\d\.\d{4}$/.test(request.mapOpacity) ||
            Number(request.mapOpacity)<0 || Number(request.mapOpacity)>1)
          throw new Error('Missing source map-opacity save owner');
        const value: SavedV3 = {schema: 3, sfxVolume: request.sfxVolume, musicVolume: request.musicVolume,
          fullscreen: request.fullscreen, mapOpacity: request.mapOpacity};
        const text = JSON.stringify(value);
        this.storage.setItem(KEY, text);
        if (this.storage.getItem(KEY) !== text) throw new Error('Preference save readback differs from the actual write');
        this.record({operation: 'save', status: 'pass', key: KEY, sequence: request.sequence, value});
      } catch (error) { success = false; this.failure('save', error, request.sequence); }
    }
    return success;
  }

  resetDispatch(): void { this.lastSave = -1; }
  snapshot() { return {key: KEY, previousKey: PREVIOUS_KEY, legacyKey: LEGACY_KEY,
    storageAvailable: this.storage !== null, events: structuredClone(this.events), truncated: this.truncated,
    scope: 'Actual origin-local audio/requested-fullscreen/map-opacity IO; native whole-file or malformed/reload parity requires separate observations'}; }
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
