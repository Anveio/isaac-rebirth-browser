import assert from "node:assert/strict";
import { existsSync } from "node:fs";

export function browserGraphicsProfile(value = process.env.FOUNDATION_BROWSER_GRAPHICS || "mesa") {
  assert.ok(["mesa", "swiftshader"].includes(value), `Unsupported FOUNDATION_BROWSER_GRAPHICS profile: ${value}`);
  return value;
}

// A system Chromium, then the cloud image's preinstalled one, whose revision
// need not match the pinned Playwright's bundled download.
const SYSTEM_CHROMIUM = ["/usr/bin/chromium", "/opt/pw-browsers/chromium"];

export function browserLaunchOptions({ profile = browserGraphicsProfile() } = {}) {
  browserGraphicsProfile(profile);
  return {
    executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE || SYSTEM_CHROMIUM.find(path => existsSync(path)),
    args: profile === "mesa"
      ? ["--use-gl=angle", "--use-angle=gl-egl", "--ignore-gpu-blocklist", "--enable-webgl"]
      : ["--use-angle=swiftshader", "--enable-unsafe-swiftshader"],
  };
}

export function graphicsEvidence(info, requestedProfile = browserGraphicsProfile()) {
  browserGraphicsProfile(requestedProfile);
  const gpu = info.gpu;
  assert.ok(gpu && Array.isArray(gpu.devices), "Browser GPU information must be observed rather than inferred from launch flags");
  const attributes = gpu.auxAttributes || {};
  const renderer = attributes.glRenderer || gpu.devices.map((device) => device.deviceString || "").join(", ");
  const vendor = attributes.glVendor || gpu.devices.map((device) => device.vendorString || "").join(", ");
  const version = attributes.glVersion || "";
  const description = [renderer, vendor, version].join(" ");
  const actualProfile = /SwiftShader/i.test(description) ? "swiftshader" : /Mesa|llvmpipe|softpipe/i.test(description) ? "mesa" : "unidentified";
  assert.equal(actualProfile, requestedProfile, `Requested ${requestedProfile} graphics but observed ${description || "no renderer"}`);
  return { requestedProfile, actualProfile, renderer, vendor, version, displayType: attributes.displayType, devices: gpu.devices, observedFrom: "Chromium SystemInfo.getInfo", scope: "Observed Chromium graphics backend; shader/pixel equality still requires the original reference comparisons" };
}

export async function inspectBrowserGraphics(browser, requestedProfile = browserGraphicsProfile()) {
  const session = await browser.newBrowserCDPSession();
  try { return graphicsEvidence(await session.send("SystemInfo.getInfo"), requestedProfile); }
  finally { await session.detach(); }
}
