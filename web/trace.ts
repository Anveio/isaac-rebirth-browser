import type { Command, MusicBackendCommand, State } from "./engine";

// Traces are diagnostic replay evidence. Ordinary runs retain no history.
// Budgets count encoded record bodies, excluding array/header punctuation and
// browser object overhead. Input metadata and the separately bounded callback
// states survive a capped ordinary state history.
export class ReplayTrace {
  private readonly commands: Command[] = [];
  private readonly states: string[] = [];
  private readonly backendEvents: { afterCommandCount: number; command: MusicBackendCommand; audioTime: number; state?: string }[] = [];
  private inputEncodedBytes = 0;
  private stateEncodedBytes = 0;
  private backendStateEncodedBytes = 0;
  private inputTruncationReason: "commands" | "bytes" | null = null;
  private stateHistoryTruncationReason: "bytes" | "input" | null = null;
  private backendStateTruncationReason: "bytes" | "input" | null = null;
  private readonly initialSeed: number;
  private readonly bootEpochSeed: number;
  private readonly clockInput: { initialTimestampMillis: number; initialSubMillisecondNanoseconds: number; presentationIntervalNanoseconds: number };
  private readonly enabled: boolean;
  private readonly maxBytes: number;
  private readonly maxCommands: number;
  private readonly maxInputBytes: number;
  private readonly maxBackendStateBytes: number;

  constructor(initialSeed: number, enabled: boolean, initialState: State,
    maxBytes = 32 * 1024 * 1024, maxCommands = 36_000, maxInputBytes = 4 * 1024 * 1024,
    maxBackendStateBytes = 4 * 1024 * 1024) {
    if (![maxBytes, maxCommands, maxInputBytes, maxBackendStateBytes].every(value => Number.isSafeInteger(value) && value >= 0))
      throw new Error('Trace budgets must be nonnegative safe integers');
    this.initialSeed = initialSeed;
    this.bootEpochSeed = initialState.bootEpochSeed ?? 0;
    this.clockInput = { initialTimestampMillis: initialState.runAdmission.clock.milliseconds,
      initialSubMillisecondNanoseconds: initialState.runAdmission.clock.subMillisecondNanoseconds,
      presentationIntervalNanoseconds: initialState.runAdmission.clock.nanosecondsPerStep };
    this.enabled = enabled;
    this.maxBytes = maxBytes;
    this.maxCommands = maxCommands;
    this.maxInputBytes = maxInputBytes;
    this.maxBackendStateBytes = maxBackendStateBytes;
    if (!enabled) return;
    const state = this.recordState(initialState);
    if (state) this.states.push(state);
  }

  record(command: Command, state: State): void {
    if (!this.reserveInput(command)) return;
    this.commands.push(structuredClone(command));
    const recordedState = this.recordState(state);
    if (recordedState) this.states.push(recordedState);
  }

  private reserveInput(metadata: unknown): boolean {
    if (!this.enabled || this.inputTruncationReason) return false;
    if (this.commands.length + this.backendEvents.length >= this.maxCommands) this.inputTruncationReason = 'commands';
    else {
      const bytes = this.bytes(metadata);
      if (this.inputEncodedBytes + bytes > this.maxInputBytes) this.inputTruncationReason = 'bytes';
      else { this.inputEncodedBytes += bytes; return true; }
    }
    this.stateHistoryTruncationReason ??= 'input';
    return false;
  }

  private recordState(state: State): string | null {
    if (this.stateHistoryTruncationReason) return null;
    const { text, bytes } = this.serializeState(state);
    if (this.stateEncodedBytes + bytes > this.maxBytes) {
      this.stateHistoryTruncationReason = 'bytes'; return null;
    }
    this.stateEncodedBytes += bytes;
    return text;
  }

  recordBackend(command: MusicBackendCommand, state: State, audioTime: number): void {
    if (!this.enabled) return;
    const event = { afterCommandCount: this.commands.length, command, audioTime };
    if (!this.reserveInput(event)) {
      this.backendStateTruncationReason ??= 'input';
      return;
    }
    let recordedState: string | null = null;
    if (!this.backendStateTruncationReason) {
      const { text, bytes } = this.serializeState(state);
      if (this.backendStateEncodedBytes + bytes > this.maxBackendStateBytes)
        this.backendStateTruncationReason = 'bytes';
      else {
        this.backendStateEncodedBytes += bytes;
        recordedState = text;
      }
    }
    this.backendEvents.push({ ...structuredClone(event), ...(recordedState ? { state: recordedState } : {}) });
  }

  snapshot() {
    return { schema: 1, tickRate: 60, initialSeed: this.initialSeed, bootEpochSeed: this.bootEpochSeed,
      clockInput: { ...this.clockInput }, enabled: this.enabled,
      truncated: this.inputTruncationReason !== null || this.stateHistoryTruncationReason !== null || this.backendStateTruncationReason !== null,
      truncationReason: this.inputTruncationReason ?? (this.stateHistoryTruncationReason || this.backendStateTruncationReason ? 'bytes' : null),
      inputTraceTruncated: this.inputTruncationReason !== null, inputTruncationReason: this.inputTruncationReason,
      stateHistoryTruncated: this.stateHistoryTruncationReason !== null, stateHistoryTruncationReason: this.stateHistoryTruncationReason,
      backendStateTruncated: this.backendStateTruncationReason !== null, backendStateTruncationReason: this.backendStateTruncationReason,
      inputEncodedBytes: this.inputEncodedBytes, maxInputEncodedBytes: this.maxInputBytes,
      stateEncodedBytes: this.stateEncodedBytes, maxStateEncodedBytes: this.maxBytes,
      backendStateEncodedBytes: this.backendStateEncodedBytes, maxBackendStateEncodedBytes: this.maxBackendStateBytes,
      encodedBytes: this.inputEncodedBytes + this.stateEncodedBytes + this.backendStateEncodedBytes,
      maxEncodedBytes: this.maxInputBytes + this.maxBytes + this.maxBackendStateBytes,
      maxCommands: this.maxCommands, commands: structuredClone(this.commands), states: this.states.map(text => JSON.parse(text) as State),
      backendEvents: this.backendEvents.map(({ state, ...event }) => ({
        ...structuredClone(event), ...(state ? { state: JSON.parse(state) as State } : {}),
      })) };
  }

  // Engine snapshots and their stain records are JSON values. Retain their
  // serialized form instead of a second expanded graph until trace export.
  private serializeState(state: State): { text: string; bytes: number } {
    const canonical = JSON.stringify(state);
    const bytes = new TextEncoder().encode(canonical).byteLength;
    // JSON's ordinary spelling collapses -0. Keep that number in snapshot()
    // while charging the same canonical JSON bytes as the existing budget.
    let marker = "\u0000trace-negative-zero";
    while (canonical.includes(JSON.stringify(marker))) marker += "0";
    let negativeZero = false;
    const encoded = JSON.stringify(state, (_key, value: unknown) => {
      if (Object.is(value, -0)) { negativeZero = true; return marker; }
      return value;
    });
    return { text: negativeZero ? encoded.replaceAll(JSON.stringify(marker), "-0") : canonical, bytes };
  }

  private bytes(value: unknown): number { return new TextEncoder().encode(JSON.stringify(value)).byteLength; }
}
