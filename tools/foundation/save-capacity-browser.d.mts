import type { Page } from "@playwright/test";
export function runSaveCapacityBrowser(page: Page, output: string, role: "under" | "over"): Promise<void>;
