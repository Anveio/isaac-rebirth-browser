import { defineConfig } from "@playwright/test";
import { browserLaunchOptions } from "./tools/foundation/browser.mjs";

const providedURL = process.env.FOUNDATION_BROWSER_URL;
const baseURL = providedURL || "http://127.0.0.1:5174";

export default defineConfig({
  testDir: "./tools/tests",
  testMatch: "**/*.spec.ts",
  workers: 1,
  use: {
    baseURL,
    launchOptions: browserLaunchOptions(),
  },
  webServer: providedURL ? undefined : {
    command: "node tools/foundation/production-server.mjs --port 5174",
    url: "http://127.0.0.1:5174",
    reuseExistingServer: false,
    timeout: 30_000,
  },
});
