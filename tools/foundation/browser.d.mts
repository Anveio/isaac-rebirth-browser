import type { Browser, LaunchOptions } from "@playwright/test";
export function browserGraphicsProfile(value?: string): "mesa" | "swiftshader";
export function browserLaunchOptions(options?: { profile?: "mesa" | "swiftshader" }): LaunchOptions;
export function graphicsEvidence(info: unknown, requestedProfile?: "mesa" | "swiftshader"): Record<string, unknown>;
export function inspectBrowserGraphics(browser: Browser, requestedProfile?: "mesa" | "swiftshader"): Promise<Record<string, unknown>>;
