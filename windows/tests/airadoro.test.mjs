// Airadoro unit tests (core/airadoro.ts).

import { test } from "node:test";
import assert from "node:assert/strict";
import {
  AiradoroEngine,
  clampMinutes,
} from "../src/core/airadoro.ts";

test("clampMinutes bounds to 1..60", () => {
  assert.equal(clampMinutes(0), 1);
  assert.equal(clampMinutes(-10), 1);
  assert.equal(clampMinutes(25), 25);
  assert.equal(clampMinutes(60), 60);
  assert.equal(clampMinutes(100), 60);
  assert.equal(clampMinutes(NaN), 1);
});

test("AiradoroEngine.clock formats seconds into MM:SS", () => {
  assert.equal(AiradoroEngine.clock(1500), "25:00");
  assert.equal(AiradoroEngine.clock(300), "05:00");
  assert.equal(AiradoroEngine.clock(5), "00:05");
  assert.equal(AiradoroEngine.clock(0), "00:00");
  assert.equal(AiradoroEngine.clock(6330), "105:30");
});

test("start begins fresh work phase with deadline", () => {
  const config = { work: 1500, rest: 300 };
  const now = 100000;
  const { snap, events } = AiradoroEngine.start(config, now);

  assert.equal(snap.phase, "work");
  assert.equal(snap.paused, false);
  assert.equal(snap.deadline, 100000 + 1500 * 1000);
  assert.equal(snap.pausedRemaining, null);
  assert.equal(snap.notice, "workStarts");
  assert.deepEqual(events, ["workStarts"]);
});

test("pause and resume preserve remaining time without drift", () => {
  const config = { work: 1500, rest: 300 };
  const now = 100000;
  const { snap } = AiradoroEngine.start(config, now);

  // 500 seconds later, user pauses
  const pauseTime = now + 500 * 1000;
  const paused = AiradoroEngine.pause(snap, pauseTime);
  assert.equal(paused.paused, true);
  assert.equal(paused.deadline, null);
  assert.equal(Math.round(paused.pausedRemaining ?? 0), 1000);

  // 100 seconds later, user resumes
  const resumeTime = pauseTime + 100 * 1000;
  const resumed = AiradoroEngine.resume(paused, resumeTime);
  assert.equal(resumed.paused, false);
  assert.equal(resumed.pausedRemaining, null);
  assert.equal(resumed.deadline, resumeTime + 1000 * 1000);

  // Remaining seconds
  assert.equal(Math.round(AiradoroEngine.remaining(resumed, resumeTime) ?? 0), 1000);
});

test("tick transitions work -> rest when deadline is met", () => {
  const config = { work: 1500, rest: 300 };
  const now = 100000;
  const { snap } = AiradoroEngine.start(config, now);

  const deadline = snap.deadline;
  // Right at deadline
  const { snap: rested, events } = AiradoroEngine.tick(snap, config, deadline + 10);
  assert.equal(rested.phase, "rest");
  assert.equal(rested.deadline, deadline + 300 * 1000);
  assert.equal(rested.notice, "restStarts");
  assert.deepEqual(events, ["restStarts"]);
});

test("tick transitions rest -> work when rest deadline is met", () => {
  const config = { work: 1500, rest: 300 };
  const restSnap = {
    phase: "rest",
    paused: false,
    deadline: 200000,
    pausedRemaining: null,
    notice: "restStarts",
  };

  const { snap: worked, events } = AiradoroEngine.tick(restSnap, config, 200000 + 10);
  assert.equal(worked.phase, "work");
  assert.equal(worked.deadline, 200000 + 1500 * 1000);
  assert.equal(worked.notice, "workStarts");
  assert.deepEqual(events, ["workStarts"]);
});

test("stop resets to idle state", () => {
  const idle = AiradoroEngine.stop();
  assert.equal(idle.phase, "idle");
  assert.equal(idle.paused, false);
  assert.equal(idle.deadline, null);
  assert.equal(idle.notice, null);
});
