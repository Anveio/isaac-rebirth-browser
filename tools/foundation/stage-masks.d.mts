export interface StageMaskCase { id: string; initialSeed: number; seedText: string; checkpoint: string; manifestPath: string; original: { captureId: string; [key: string]: unknown }; [key: string]: unknown; }
export const STAGE_MASK_MANIFEST: string;
export const STAGE_MASK_DEPTH_CONTRACT: Readonly<{ path: string; sha256: string }>;
export function readStageMaskCases(path?: string): StageMaskCase[];
export function loadStageMaskReference(reference: StageMaskCase): StageMaskCase & { observed: { seedValue: number; stage: number; stageSeed: number } };
export function loadControlledStageMaskEvidence(reference: StageMaskCase): unknown;
export function loadStageMaskDepthContract(selected?: { path: string; sha256: string }): unknown;
export function compareStageMaskTargets(actual: unknown, reference: StageMaskCase, output?: string): { status: string; results: Record<string, { passed: boolean; changedPixels: number; exactRGBA: boolean }>; originalStateParity: boolean; originalSimulationTickDelivery: boolean };
