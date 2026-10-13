import { defineConfig } from "@playwright/test";
import { browserLaunchOptions } from "./tools/foundation/browser.mjs";

const providedURL = process.env.FOUNDATION_BROWSER_URL;
const baseURL = providedURL || "http://127.0.0.1:5173";

export default defineConfig({
  testDir: "./tools/tests",
  testMatch: "**/*.spec.ts",
  workers: 1,
  use: {
    baseURL,
    launchOptions: browserLaunchOptions(),
  },
  webServer: providedURL ? undefined : {
    command: "npm run dev",
    url: "http://127.0.0.1:5173",
    reuseExistingServer: !process.env.CI,
    timeout: 30_000,
  },
});
