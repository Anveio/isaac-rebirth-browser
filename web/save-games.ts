/** Browser-owned profile transport (docs/save-system.md, Storage). The engine
 * builds and validates every stored document; this host keeps them as
 * generations under a slot head and never interprets a payload.
 *
 * Layout: each save writes its schema-3 envelope under a new generation's
 * record key, then swaps the head (the commit point: one atomic setItem) to
 * name it, keeping the generation before it as `previous` for recovery, then
 * removes records the head no longer names. Load tries the head's current
 * record, then `previous`, then the legacy schema-1/2 document. A save that
 * removes a slot's run record (death, run end, Exit while dead) keeps no
 * `previous`, so recovery can never bring back a lost run. */
export const LEGACY_SAVE_KEY = 'isaac-rebirth.savegames.v1';
export const HEAD_KEY = 'isaac-rebirth.save.head.v3';
export const recordKey = (generation: number) => `isaac-rebirth.save.record.v3.${generation}`;
export const SAVE_BYTE_LIMIT = 4 * 1024 * 1024;
//Exact expected refusal from the current string-only save export ABI.
const SAVE_CAPACITY_ERROR = "saved profiles exceed the supported 4 MiB format";
const ENVELOPE = /^\{"magic":"ISAAC-RB-SAVE","schema":3,"checksum":(\d+),"runs":\[(true|false),(true|false),(true|false)\],"payload":/;

interface SaveEngine {
  enableSaveIO(): void;
  completeSaveWrite(revision: number, succeeded: boolean): void;
  importSaveData(text: string): void;
  exportSaveData(): string;
  completeMenuReadmission(revision: number, text: string | null): boolean;
}
type Source = 'current' | 'previous' | 'legacy';
type IOEvent = {
  operation: 'load' | 'save' | 'readmit'; status: 'pass' | 'absent' | 'fail';
  revision?: number; text?: string; error?: string; source?: Source; generation?: number;
  rejected?: {source: Source; generation?: number; error: string}[];
};
type Ref = {generation: number; checksum: number};
type Head = {schema: 3; current: Ref; previous: Ref | null};
type Candidate = {source: Source; text: string; generation?: number};

function bounded(text: string): void {
  if (new TextEncoder().encode(text).length > SAVE_BYTE_LIMIT)
    throw new Error('Saved profiles exceed the supported 4 MiB format');
}

const CRC_TABLE = (() => {
  const table = new Uint32Array(256);
  for (let index = 0; index < 256; index++) {
    let value = index;
    for (let bit = 0; bit < 8; bit++) value = value & 1 ? 0xedb88320 ^ (value >>> 1) : value >>> 1;
    table[index] = value >>> 0;
  }
  return table;
})();

/** CRC-32 (IEEE), the engine's envelope checksum. */
export function crc32(bytes: Uint8Array): number {
  let crc = 0xffffffff;
  for (const byte of bytes) crc = CRC_TABLE[(crc ^ byte) & 0xff] ^ (crc >>> 8);
  return (crc ^ 0xffffffff) >>> 0;
}

/** A schema-3 envelope's header, verified against its payload bytes. */
export function envelopeHeader(text: string): {checksum: number; runs: boolean[]} {
  const match = ENVELOPE.exec(text);
  if (!match || !text.endsWith('}')) throw new Error('Stored record is not a schema-3 save envelope');
  const checksum = Number(match[1]);
  const payload = new TextEncoder().encode(text.slice(match[0].length, -1));
  if (!Number.isSafeInteger(checksum) || crc32(payload) !== checksum)
    throw new Error('Stored record checksum differs from its payload');
  return {checksum, runs: [match[2], match[3], match[4]].map(flag => flag === 'true')};
}

function validRef(value: unknown): value is Ref {
  const ref = value as Ref;
  return !!ref && Number.isSafeInteger(ref.generation) && ref.generation > 0
    && Number.isSafeInteger(ref.checksum) && ref.checksum >= 0 && ref.checksum <= 0xffffffff;
}

export class SaveGameIO {
  private storage: Storage | null = null;
  private declared = false;
  private events: IOEvent[] = [];
  private truncated = false;
  private consumedRevision = 0;
  private retainedBytes = 0;
  private persistRequested = false;
  private persisted: boolean | null = null;
  private writerLock: 'held' | 'unavailable' | 'unsupported' | 'pending' = 'pending';
  constructor() {
    try { this.storage = window.localStorage; }
    catch (error) { this.record({operation: 'load', status: 'fail', error: String(error)}); }
    this.requestWriterLock();
  }

  /** One writing tab: the first tab holds the lock for its lifetime; a tab
   * that cannot take it keeps playing with its writes refused, so two tabs
   * never interleave generations. */
  private requestWriterLock(): void {
    const locks = typeof navigator === 'undefined' ? undefined : (navigator as Navigator).locks;
    if (!locks?.request) { this.writerLock = 'unsupported'; return; }
    const hold = () => { this.writerLock = 'held'; return new Promise<void>(() => {}); };
    locks.request('isaac-rebirth-save', {ifAvailable: true}, lock => {
      if (lock) return hold();
      //Another tab (or this page's unloading predecessor) holds it: writes
      //are refused until it is released and this tab takes it.
      this.writerLock = 'unavailable';
      locks.request('isaac-rebirth-save', hold).catch(() => { this.writerLock = 'unsupported'; });
    }).catch(() => { this.writerLock = 'unsupported'; });
  }

  private head(): Head | null {
    const text = this.storage!.getItem(HEAD_KEY);
    if (text === null) return null;
    const head = JSON.parse(text) as Head;
    if (head?.schema !== 3 || !validRef(head.current) || (head.previous !== null && !validRef(head.previous)))
      throw new Error('Stored save head is malformed');
    return head;
  }

  /** The stored documents a load may use, best first. A record the head
   * names must match the head's checksum and its own payload checksum. */
  private candidates(rejected: NonNullable<IOEvent['rejected']>): Candidate[] {
    const storage = this.storage!;
    let head: Head | null;
    try { head = this.head(); }
    catch (error) { rejected.push({source: 'current', error: String(error)}); head = null; }
    if (head === null) {
      const legacy = storage.getItem(LEGACY_SAVE_KEY);
      return legacy === null ? [] : [{source: 'legacy', text: legacy}];
    }
    const found: Candidate[] = [];
    for (const [source, ref] of [['current', head.current], ['previous', head.previous]] as const) {
      if (ref === null) continue;
      const text = storage.getItem(recordKey(ref.generation));
      try {
        if (text === null) throw new Error('Stored save record is missing');
        bounded(text);
        if (envelopeHeader(text).checksum !== ref.checksum) throw new Error('Stored record is not the one its head names');
        found.push({source, text, generation: ref.generation});
      } catch (error) { rejected.push({source, generation: ref.generation, error: String(error)}); }
    }
    return found;
  }

  /** A load changes only retained profile/save records, never live gameplay.
   * The first stored document the engine accepts loads; a rejected one stays
   * stored. With no acceptable document the engine keeps its rejection, so
   * it refuses to overwrite the stored bytes. False on a storage failure. */
  load(engine: SaveEngine): boolean {
    this.consumedRevision = 0;
    this.declared = false;
    engine.enableSaveIO();
    this.declared = true;
    const rejected: NonNullable<IOEvent['rejected']> = [];
    try {
      if (!this.storage) throw new Error('Saved profile storage is unavailable');
      const candidates = this.candidates(rejected);
      if (candidates.length === 0 && rejected.length === 0) { this.record({operation: 'load', status: 'absent'}); return true; }
      for (const candidate of candidates) {
        try {
          engine.importSaveData(candidate.text);
          this.record({operation: 'load', status: 'pass', text: candidate.text, source: candidate.source,
            generation: candidate.generation, ...(rejected.length ? {rejected} : {})});
          return true;
        } catch (error) { rejected.push({source: candidate.source, generation: candidate.generation, error: String(error)}); }
      }
      throw new Error('No stored save document is loadable');
    } catch (error) {
      this.record({operation: 'load', status: 'fail', error: String(error), ...(rejected.length ? {rejected} : {})});
      return false;
    }
  }

  /** Only a completed canonical Save/Clear requests a write. Actual storage
   * failure is acknowledged to the engine and then reported to the host. */
  consume(engine: SaveEngine, revision: number): boolean {
    if (!Number.isSafeInteger(revision) || revision !== this.consumedRevision + 1)
      throw new Error('Save IO requires the exact next actual callback revision');
    //The runtime exports this immutable request, not the later mutable profile.
    //No retry/coalescing can replace an older request with a newer packet.
    //Only the canonical byte-capacity refusal is a normal failed write.
    //Other request/ABI errors cannot masquerade as storage failure.
    let text: string | undefined;
    try { text = engine.exportSaveData(); }
    catch (error) {
      if (!(error instanceof Error) || error.message !== SAVE_CAPACITY_ERROR) throw error;
    }
    if (text !== undefined) bounded(text);
    let succeeded = false;
    let generation: number | undefined;
    try {
      if (text === undefined) throw new Error(SAVE_CAPACITY_ERROR);
      if (!this.storage) throw new Error('Saved profile storage is unavailable');
      if (this.writerLock === 'unavailable') throw new Error('Another tab owns this origin\'s saves');
      generation = this.commit(this.storage, text);
      this.record({operation: 'save', status: 'pass', revision, text, generation});
      succeeded = true;
      this.requestPersistence();
    } catch (error) {
      this.record({operation: 'save', status: 'fail', revision, text, generation, error: String(error)});
    }
    this.consumedRevision = revision;
    //Runtime/continuation exceptions are outside the storage failure catch.
    engine.completeSaveWrite(revision, succeeded);
    return succeeded;
  }

  /** Record first, then the head swap that commits it, then cleanup. A
   * storage quota failure removes unreferenced records and retries once. */
  private commit(storage: Storage, text: string): number {
    const header = envelopeHeader(text);
    let head: Head | null = null;
    try { head = this.head(); } catch { head = null; }
    const generation = (head?.current.generation ?? 0) + 1;
    const key = recordKey(generation);
    try { storage.setItem(key, text); }
    catch (error) {
      this.collect(storage, head);
      try { storage.setItem(key, text); } catch { throw error; }
    }
    let previous: Ref | null = head?.current ?? null;
    if (previous !== null) {
      //A save that drops a slot's run keeps no recovery generation.
      const before = storage.getItem(recordKey(previous.generation));
      let runs: boolean[] | null = null;
      try { runs = before === null ? null : envelopeHeader(before).runs; } catch { runs = null; }
      if (runs === null || runs.some((had, slot) => had && !header.runs[slot])) previous = null;
    }
    const next: Head = {schema: 3, current: {generation, checksum: header.checksum}, previous};
    const headText = JSON.stringify(next);
    storage.setItem(HEAD_KEY, headText);
    if (storage.getItem(HEAD_KEY) !== headText || storage.getItem(key) !== text)
      throw new Error('Saved profile readback differs from the actual save');
    this.collect(storage, next);
    storage.removeItem(LEGACY_SAVE_KEY);
    return generation;
  }

  /** Removes every record the head does not name. */
  private collect(storage: Storage, head: Head | null): void {
    const keep = new Set([head?.current, head?.previous].filter(ref => ref).map(ref => recordKey(ref!.generation)));
    const prefix = recordKey(0).slice(0, -1);
    const stale: string[] = [];
    for (let index = 0; index < storage.length; index++) {
      const key = storage.key(index);
      if (key !== null && key.startsWith(prefix) && !keep.has(key)) stale.push(key);
    }
    for (const key of stale) storage.removeItem(key);
  }

  private requestPersistence(): void {
    if (this.persistRequested) return;
    this.persistRequested = true;
    const storageManager = typeof navigator === 'undefined' ? undefined : (navigator as Navigator).storage;
    storageManager?.persist?.().then(granted => { this.persisted = granted; }, () => { this.persisted = false; });
  }

  /**Read the actual own store even after a failed write: older valid bytes
   * remain a valid load outcome. The revision matches only this host request. */
  readmit(engine: SaveEngine, revision: number): boolean {
    let candidate: Candidate | undefined;
    const rejected: NonNullable<IOEvent['rejected']> = [];
    try {
      if (!this.storage) throw new Error('Saved profile storage is unavailable');
      candidate = this.candidates(rejected)[0];
    } catch (error) {
      this.record({operation:'readmit', status:'fail', revision, error:String(error)});
      engine.completeMenuReadmission(revision, null);
      return false;
    }
    const text = candidate?.text ?? null;
    //Actual runtime/continuation errors propagate; they are not reclassified
    //as absent storage or retried with a different saved run.
    const accepted = engine.completeMenuReadmission(revision, text);
    this.record({operation:'readmit', status:!accepted ? 'fail' : text === null ? 'absent' : 'pass', revision,
      ...(candidate ? {text: candidate.text, source: candidate.source, generation: candidate.generation} : {}),
      ...(rejected.length ? {rejected} : {}),
      ...(!accepted ? {error:'Actual own saved document was rejected'} : {})});
    return accepted;
  }

  /** `key` names the stored document a load would read first. */
  snapshot() {
    let key = LEGACY_SAVE_KEY, head: Head | null = null;
    try {
      head = this.storage ? this.head() : null;
      if (head) key = recordKey(head.current.generation);
    } catch { head = null; }
    return {key, headKey: HEAD_KEY, head, protocol: 1, declared: this.declared, storageAvailable: this.storage !== null,
      writerLock: this.writerLock, persisted: this.persisted,
      events: structuredClone(this.events), truncated: this.truncated,
      scope: 'Actual origin-local own-save IO; Native binary save format is a separate transport'};
  }
  private record(event: IOEvent): void {
    const bytes = new TextEncoder().encode(JSON.stringify(event)).length;
    if (this.events.length >= 128 || this.retainedBytes + bytes > 16 * 1024 * 1024) this.truncated = true;
    else { this.events.push(event); this.retainedBytes += bytes; }
  }
}
