/** Browser-owned profile transport. A persisted run is produced and validated
 * by the canonical WASM save owner; native files/captures never enter here. */
export const SAVE_KEY = 'isaac-rebirth.savegames.v1';
export const SAVE_BYTE_LIMIT = 4 * 1024 * 1024;

interface SaveEngine {
  enableSaveIO(): void;
  completeSaveWrite(revision: number, succeeded: boolean): void;
  importSaveData(text: string): void;
  exportSaveData(): string;
  completeMenuReadmission(revision: number, text: string | null): boolean;
}
type IOEvent = {
  operation: 'load' | 'save' | 'readmit'; status: 'pass' | 'absent' | 'fail';
  revision?: number; text?: string; error?: string;
};

function bounded(text: string): void {
  if (new TextEncoder().encode(text).length > SAVE_BYTE_LIMIT)
    throw new Error('Saved profiles exceed the supported 4 MiB format');
}

export class SaveGameIO {
  private storage: Storage | null = null;
  private declared = false;
  private events: IOEvent[] = [];
  private truncated = false;
  private consumedRevision = 0;
  private retainedBytes = 0;
  constructor() {
    try { this.storage = window.localStorage; }
    catch (error) { this.record({operation: 'load', status: 'fail', error: String(error)}); }
  }
  /** A load changes only retained profile/save records, never live gameplay.
   * The public profile/Continue callbacks still own actual run construction. */
  load(engine: SaveEngine): boolean {
    this.consumedRevision = 0;
    this.declared = false;
    engine.enableSaveIO();
    this.declared = true;
    try {
      if (!this.storage) throw new Error('Saved profile storage is unavailable');
      const text = this.storage.getItem(SAVE_KEY);
      if (text === null) { this.record({operation: 'load', status: 'absent'}); return true; }
      bounded(text);
      engine.importSaveData(text);
      this.record({operation: 'load', status: 'pass', text});
      return true;
    } catch (error) {
      this.record({operation: 'load', status: 'fail', error: String(error)});
      return false;
    }
  }
  /** Only a completed canonical Save/Clear requests a write. Actual storage
   * failures remain visible and never manufacture a successful IO event. */
  consume(engine: SaveEngine, revision: number): boolean {
    if (!Number.isSafeInteger(revision) || revision !== this.consumedRevision + 1)
      throw new Error('Save IO requires the exact next actual callback revision');
    //The runtime exports this immutable request, not the later mutable profile.
    //No retry/coalescing can replace an older request with a newer packet.
    //Canonical request/ABI errors cannot masquerade as storage failure.
    const text = engine.exportSaveData();
    bounded(text);
    let succeeded = false;
    try {
      if (!this.storage) throw new Error('Saved profile storage is unavailable');
      this.storage.setItem(SAVE_KEY, text);
      if (this.storage.getItem(SAVE_KEY) !== text)
        throw new Error('Saved profile readback differs from the actual save');
      this.record({operation: 'save', status: 'pass', revision, text});
      succeeded = true;
    } catch (error) {
      this.record({operation: 'save', status: 'fail', revision, text, error: String(error)});
    }
    this.consumedRevision = revision;
    //Runtime/continuation exceptions are outside the storage failure catch.
    engine.completeSaveWrite(revision, succeeded);
    return succeeded;
  }
  /**Read the actual own store even after a failed write: older valid bytes
   * remain a valid load outcome. The revision matches only this host request. */
  readmit(engine: SaveEngine, revision: number): boolean {
    let text: string | null;
    try {
      if (!this.storage) throw new Error('Saved profile storage is unavailable');
      text = this.storage.getItem(SAVE_KEY);
      if (text !== null) bounded(text);
    } catch (error) {
      this.record({operation:'readmit', status:'fail', revision, error:String(error)});
      engine.completeMenuReadmission(revision, null);
      return false;
    }
    //Actual runtime/continuation errors propagate; they are not reclassified
    //as absent storage or retried with a different saved run.
    const accepted = engine.completeMenuReadmission(revision, text);
    this.record({operation:'readmit', status:!accepted ? 'fail' : text === null ? 'absent' : 'pass', revision,
      ...(text === null ? {} : {text}), ...(!accepted ? {error:'Actual own saved document was rejected'} : {})});
    return accepted;
  }
  snapshot() {
    return {key: SAVE_KEY, protocol: 1, declared: this.declared, storageAvailable: this.storage !== null,
      events: structuredClone(this.events), truncated: this.truncated,
      scope: 'Actual origin-local own-save IO; Native binary save format is a separate transport'};
  }
  private record(event: IOEvent): void {
    const bytes = new TextEncoder().encode(JSON.stringify(event)).length;
    if (this.events.length >= 128 || this.retainedBytes + bytes > 16 * 1024 * 1024) this.truncated = true;
    else { this.events.push(event); this.retainedBytes += bytes; }
  }
}
