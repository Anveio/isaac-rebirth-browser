import type { Command, State } from "./engine";

// Traces are diagnostic replay evidence. Ordinary runs retain no history.
// The byte budget counts encoded evidence, not the browser's object overhead.
export class ReplayTrace {
  private readonly commands: Command[] = [];
  private readonly states: State[] = [];
  private encodedBytes = 0;
  private truncationReason: "commands" | "bytes" | null = null;
  private readonly initialSeed: number;
  private readonly enabled: boolean;
  private readonly maxBytes: number;
  private readonly maxCommands: number;

  constructor(initialSeed: number, enabled: boolean, initialState: State,
    maxBytes = 32 * 1024 * 1024, maxCommands = 36_000) {
    this.initialSeed = initialSeed;
    this.enabled = enabled;
    this.maxBytes = maxBytes;
    this.maxCommands = maxCommands;
    if (!enabled) return;
    const bytes = this.bytes(initialState);
    if (bytes > maxBytes) this.truncationReason = "bytes";
    else { this.states.push(structuredClone(initialState)); this.encodedBytes = bytes; }
  }

  record(command: Command, state: State): void {
    if (!this.enabled || this.truncationReason) return;
    if (this.commands.length >= this.maxCommands) { this.truncationReason = "commands"; return; }
    const bytes = this.bytes(command) + this.bytes(state);
    if (this.encodedBytes + bytes > this.maxBytes) { this.truncationReason = "bytes"; return; }
    this.commands.push(structuredClone(command));
    this.states.push(structuredClone(state));
    this.encodedBytes += bytes;
  }

  snapshot() {
    return { schema: 1, tickRate: 60, initialSeed: this.initialSeed, enabled: this.enabled,
      truncated: this.truncationReason !== null, truncationReason: this.truncationReason,
      encodedBytes: this.encodedBytes, maxEncodedBytes: this.maxBytes,
      commands: structuredClone(this.commands), states: structuredClone(this.states) };
  }

  private bytes(value: unknown): number { return new TextEncoder().encode(JSON.stringify(value)).byteLength; }
}
