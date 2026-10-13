import { test } from "node:test";
import assert from "node:assert/strict";
import { browserGraphicsProfile, browserLaunchOptions, graphicsEvidence } from "./browser.mjs";

test("browser graphics profiles select one shared backend without a silent profile fallback", () => {
  assert.equal(browserGraphicsProfile("mesa"), "mesa");
  assert.deepEqual(browserLaunchOptions({ profile: "mesa" }).args, ["--use-gl=angle", "--use-angle=gl-egl", "--ignore-gpu-blocklist", "--enable-webgl"]);
  assert.deepEqual(browserLaunchOptions({ profile: "swiftshader" }).args, ["--use-angle=swiftshader", "--enable-unsafe-swiftshader"]);
  assert.throws(() => browserLaunchOptions({ profile: "unknown" }), /Unsupported FOUNDATION_BROWSER_GRAPHICS/);
});

test("graphics reports identify actual GPU data and reject flags-only or mismatched backend claims", () => {
  const mesa = { gpu: { devices: [], auxAttributes: { glRenderer: "ANGLE (Mesa, llvmpipe (LLVM 19.1.7), OpenGL 4.5)", glVendor: "Google Inc. (Mesa)", glVersion: "OpenGL ES 3.0 Mesa 25.0.7" } } };
  const swift = { gpu: { devices: [], auxAttributes: { glRenderer: "ANGLE (Google, Vulkan (SwiftShader Device))" } } };
  assert.equal(graphicsEvidence(mesa, "mesa").actualProfile, "mesa");
  assert.equal(graphicsEvidence(swift, "swiftshader").actualProfile, "swiftshader");
  assert.throws(() => graphicsEvidence(swift, "mesa"), /Requested mesa graphics/);
  assert.throws(() => graphicsEvidence({ requestedProfile: "mesa" }, "mesa"), /must be observed/);
  assert.throws(() => graphicsEvidence({ gpu: { devices: [] } }, "mesa"), /Requested mesa graphics/);
});
