import { test, type Page, type TestInfo } from "@playwright/test";
import { existsSync } from "node:fs";
import { runSaveCapacityBrowser } from "../foundation/save-capacity-browser.mjs";
import { runSaveRegularWriterBrowser } from "../foundation/save-current-regular-browser.mjs";

async function checkCapacity(page: Page, info: TestInfo, role: "under" | "over") {
  const output = info.outputPath("save-capacity");
  try { await runSaveCapacityBrowser(page, output, role); }
  finally {
    const report = `${output}/report.json`;
    if (existsSync(report)) await info.attach("capacity-report", { path: report, contentType: "application/json" });
  }
}

// Finite public tapes bound these whole-game checks. Both complete the actual
// capacity boundary and Continue; a wall-clock cutoff cannot substitute for it.
test("Save capacity under writes the complete packet and Continues its own store", async ({ page }, info) => {
  test.setTimeout(0); await checkCapacity(page, info, "under");
});

test("Save capacity over fails ACK once and Continues the unchanged older store", async ({ page }, info) => {
  test.setTimeout(0); await checkCapacity(page, info, "over");
});

// The finite fixed public tape ends at real menu readmission, without Continue.
test("Pause Quit saves live regular pickups in the current room", async ({ page }, info) => {
  test.setTimeout(0);
  const output = info.outputPath("save-current-regular");
  try { await runSaveRegularWriterBrowser(page, output); }
  finally {
    const report = `${output}/report.json`;
    if (existsSync(report)) await info.attach("current-regular-save-report", { path: report, contentType: "application/json" });
  }
});
