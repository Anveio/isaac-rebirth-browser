import type { Command, State } from "../../web/engine";
export function assertGameplayInvariants(initialSeed: number, commands: Command[], states: State[], options?: { manualSeed?: boolean }): { states: number; liveShotStates: number; pauseChecks: number; directionChecks: Array<{ code: string; commandIndex: number; newestTearId: number }>; opposingChecks: number; effectChecks: number; roomChecks: number };
export function invariantTape(encodedSeed: string): Command[];
export function wasmReplay(initialSeed: number, commands?: Command[]): Promise<State[]>;
