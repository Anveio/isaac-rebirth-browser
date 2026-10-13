import type { Page } from "@playwright/test";
import type { Command, State } from "../../web/engine";
export const FOUNDATION_SEED: string;
export const FOUNDATION_REFERENCE_INITIAL_SEED: number;
export const CHECKPOINTS: string[];
export interface BrowserCapture {
  label: string; state: State; png: string;
  timingMs?: Record<string, number>; timingScope?: string;
}
export interface BrowserRun { initialSeed: number; bootEpochSeed: number; commands: Command[]; states: State[]; backendEvents: { afterCommandCount: number; command: Command; audioTime: number; state: State }[]; frames: BrowserCapture[]; timings: Array<{ commandIndex: number; ticks: number; durationMs: number; scope: string }>; tickRate: number; seed: string; inputTransport: string; }
export function waitForReady(page: Page): Promise<void>;
export function manualSeedCommands(seedText: string | null): Command[];
export function assertSeededRun(state: State, seedText: string, seedValue?: number): void;
export function assertInputProvenance(trace: { enabled: boolean; inputTraceTruncated: boolean; initialSeed: number;
  bootEpochSeed: number; commands: Command[] }, commands: Command[], initialSeed: number, bootEpochSeed?: number): void;
export function compareCapturedFrames(frames: BrowserCapture[], original: unknown[], output: string): { checks: Array<{ id: string; status: string; error?: string }>; exactOriginalPixels: boolean };
export function browserScenario(page: Page, options?: { initialSeed?: number; bootEpochSeed?: number; manualSeedText?: string | null; onCheckpoint?: (frame: BrowserCapture) => Promise<void>; bombCheckpoint?: string }): Promise<BrowserRun>;
export function nativeReplay(initialSeed: number, commands: Command[], bootEpochSeed?: number, backendEvents?: { afterCommandCount: number; command: Command; state?: State }[]): State[];
export function investigationForRegion(region: string, result: { passed: boolean; [key: string]: unknown }, evidence: unknown): { taskIds: string[]; tasks: unknown[]; status?: string };
export function readReferences(paths: string[]): unknown[];
export function verify(options?: { references?: string[]; output?: string; baseURL?: string }): Promise<{ status: string; checks: Array<{ id: string; status: string; error?: string }>; exactOriginalPixels: boolean; originalStateParity: boolean }>;
