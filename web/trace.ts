import type { Command, MusicBackendCommand, State } from "./engine";

// Traces are diagnostic replay evidence. Ordinary runs retain no history.
// Budgets count encoded record bodies, excluding array/header punctuation and
// browser object overhead. Input metadata and the separately bounded callback
// states survive a capped ordinary state history.
export class ReplayTrace {
  private readonly commands: Command[] = [];
  private readonly states: State[] = [];
  private readonly backendEvents: { afterCommandCount: number; command: MusicBackendCommand; audioTime: number; state?: State }[] = [];
  private inputEncodedBytes = 0;
  private stateEncodedBytes = 0;
  private backendStateEncodedBytes = 0;
  private inputTruncationReason: "commands" | "bytes" | null = null;
  private stateHistoryTruncationReason: "bytes" | "input" | null = null;
  private backendStateTruncationReason: "bytes" | "input" | null = null;
  private readonly initialSeed: number;
  private readonly bootEpochSeed: number;
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

  private recordState(state: State): State | null {
    if (this.stateHistoryTruncationReason) return null;
    const bytes = this.bytes(state);
    if (this.stateEncodedBytes + bytes > this.maxBytes) {
      this.stateHistoryTruncationReason = 'bytes'; return null;
    }
    this.stateEncodedBytes += bytes;
    return structuredClone(state);
  }

  recordBackend(command: MusicBackendCommand, state: State, audioTime: number): void {
    if (!this.enabled) return;
    const event = { afterCommandCount: this.commands.length, command, audioTime };
    if (!this.reserveInput(event)) {
      this.backendStateTruncationReason ??= 'input';
      return;
    }
    let recordedState: State | null = null;
    if (!this.backendStateTruncationReason) {
      const bytes = this.bytes(state);
      if (this.backendStateEncodedBytes + bytes > this.maxBackendStateBytes)
        this.backendStateTruncationReason = 'bytes';
      else {
        this.backendStateEncodedBytes += bytes;
        recordedState = structuredClone(state);
      }
    }
    this.backendEvents.push({ ...structuredClone(event), ...(recordedState ? { state: recordedState } : {}) });
  }

  snapshot() {
    return { schema: 1, tickRate: 60, initialSeed: this.initialSeed, bootEpochSeed: this.bootEpochSeed, enabled: this.enabled,
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
      maxCommands: this.maxCommands, commands: structuredClone(this.commands), states: structuredClone(this.states),
      backendEvents: structuredClone(this.backendEvents) };
  }

  private bytes(value: unknown): number { return new TextEncoder().encode(JSON.stringify(value)).byteLength; }
}
