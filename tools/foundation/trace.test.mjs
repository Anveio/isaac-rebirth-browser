import assert from "node:assert/strict";
import { test } from "node:test";
import { ReplayTrace } from "../../web/trace.ts";

const state = { tick: 0, label: "initial" };
const command = { type: "tick", count: 1 };

test("ordinary play retains no replay history while opt-in traces preserve immutable commands and states", () => {
  const off = new ReplayTrace(17, false, state);
  for (let tick = 1; tick <= 40_000; tick++) off.record(command, { ...state, tick });
  assert.deepEqual(off.snapshot().states, []);
  assert.deepEqual(off.snapshot().commands, []);
  assert.equal(off.snapshot().encodedBytes, 0);
  assert.equal(off.snapshot().truncated, false);
  const initial = structuredClone(state), next = { ...state, tick: 1 }, input = structuredClone(command);
  const on = new ReplayTrace(17, true, initial);
  on.record(input, next);
  initial.label = next.label = "mutated"; input.count = 2;
  const evidence = on.snapshot();
  assert.deepEqual(evidence.states, [state, { ...state, tick: 1 }]);
  assert.deepEqual(evidence.commands, [command]);
  evidence.states[0].tick = 100;
  assert.equal(on.snapshot().states[0].tick, 0);
});

test("replay evidence budgets keep complete prefixes and report exhaustion instead of silently losing commands", () => {
  const bytes = (value) => Buffer.byteLength(JSON.stringify(value));
  const budget = bytes(state) + bytes(command) + bytes({ ...state, tick: 1 });
  const trace = new ReplayTrace(17, true, state, budget);
  trace.record(command, { ...state, tick: 1 });
  assert.equal(trace.snapshot().encodedBytes, budget);
  assert.equal(trace.snapshot().truncated, false);
  trace.record(command, { ...state, tick: 2 });
  trace.record(command, { ...state, tick: 3 });
  assert.equal(trace.snapshot().truncationReason, "bytes");
  assert.equal(trace.snapshot().commands.length, 1);
  assert.equal(trace.snapshot().states.length, 2);
  const countLimited = new ReplayTrace(17, true, state, 1024, 0);
  countLimited.record(command, state);
  assert.equal(countLimited.snapshot().truncationReason, "commands");
  const initialTooLarge = new ReplayTrace(17, true, state, 0);
  assert.equal(initialTooLarge.snapshot().truncationReason, "bytes");
  assert.deepEqual(initialTooLarge.snapshot().states, []);
});
