// Airadoro: Pomodoro timer (work, then rest, on repeat, until you stop).
// Port of AiradoroEngine.swift and AiradoroSettings.swift.
// Pure logic: no clock of its own, zero CPU while idle.

export const AIRADORO_ID = "integration_airadoro";

export type AiradoroPhase = "idle" | "work" | "rest";
export type AiradoroEvent = "workStarts" | "restStarts";

export interface AiradoroConfig {
  /** Work length in seconds. */
  work: number;
  /** Rest length in seconds. */
  rest: number;
}

export interface AiradoroSnapshot {
  phase: AiradoroPhase;
  paused: boolean;
  /** Epoch timestamp in ms for when the current phase ends. */
  deadline: number | null;
  /** Seconds remaining, frozen while paused. */
  pausedRemaining: number | null;
  /** Phase transition event just occurred. */
  notice: AiradoroEvent | null;
}

export const AIRADORO_STORAGE_WORK = "airadoroWorkMinutes";
export const AIRADORO_STORAGE_REST = "airadoroRestMinutes";

export const DEFAULT_WORK_MINUTES = 25;
export const DEFAULT_REST_MINUTES = 5;

export const MIN_MINUTES = 1;
export const MAX_MINUTES = 60;

export function clampMinutes(val: number): number {
  if (!Number.isFinite(val)) return MIN_MINUTES;
  return Math.min(MAX_MINUTES, Math.max(MIN_MINUTES, Math.round(val)));
}

export function loadStoredMinutes(key: string, fallback: number): number {
  try {
    const raw = typeof localStorage !== "undefined" ? localStorage.getItem(key) : null;
    if (raw == null) return fallback;
    const n = parseInt(raw, 10);
    return clampMinutes(Number.isNaN(n) ? fallback : n);
  } catch {
    return fallback;
  }
}

export function saveStoredMinutes(key: string, minutes: number) {
  try {
    if (typeof localStorage !== "undefined") {
      localStorage.setItem(key, String(clampMinutes(minutes)));
    }
  } catch {
    // Ignore storage quota/security issues
  }
}

export function getAiradoroConfig(): AiradoroConfig {
  const workM = loadStoredMinutes(AIRADORO_STORAGE_WORK, DEFAULT_WORK_MINUTES);
  const restM = loadStoredMinutes(AIRADORO_STORAGE_REST, DEFAULT_REST_MINUTES);
  return {
    work: workM * 60,
    rest: restM * 60,
  };
}

export const AiradoroEngine = {
  sanitize(raw: AiradoroConfig): AiradoroConfig {
    return {
      work: Math.max(1, raw.work),
      rest: Math.max(1, raw.rest),
    };
  },

  start(rawConfig: AiradoroConfig, nowMs = Date.now()): { snap: AiradoroSnapshot; events: AiradoroEvent[] } {
    const config = this.sanitize(rawConfig);
    const deadline = nowMs + config.work * 1000;
    const snap: AiradoroSnapshot = {
      phase: "work",
      paused: false,
      deadline,
      pausedRemaining: null,
      notice: "workStarts",
    };
    return { snap, events: ["workStarts"] };
  },

  stop(): AiradoroSnapshot {
    return {
      phase: "idle",
      paused: false,
      deadline: null,
      pausedRemaining: null,
      notice: null,
    };
  },

  pause(s: AiradoroSnapshot, nowMs = Date.now()): AiradoroSnapshot {
    if (s.phase === "idle" || s.paused) return s;
    const remaining = s.deadline ? Math.max(0, (s.deadline - nowMs) / 1000) : 0;
    return {
      ...s,
      paused: true,
      deadline: null,
      pausedRemaining: remaining,
    };
  },

  resume(s: AiradoroSnapshot, nowMs = Date.now()): AiradoroSnapshot {
    if (!s.paused) return s;
    const left = Math.max(0, s.pausedRemaining ?? 0);
    return {
      ...s,
      paused: false,
      pausedRemaining: null,
      deadline: nowMs + left * 1000,
    };
  },

  tick(s: AiradoroSnapshot, rawConfig: AiradoroConfig, nowMs = Date.now()): { snap: AiradoroSnapshot; events: AiradoroEvent[] } {
    if (s.phase === "idle" || s.paused || s.deadline == null) return { snap: s, events: [] };
    if (nowMs < s.deadline) return { snap: s, events: [] };

    const config = this.sanitize(rawConfig);
    const landed = this.land(s.phase, s.deadline, nowMs, config);
    const next: AiradoroSnapshot = {
      ...s,
      phase: landed.phase,
      deadline: landed.deadline,
      notice: landed.event,
    };
    return { snap: next, events: [landed.event] };
  },

  remaining(s: AiradoroSnapshot, nowMs = Date.now()): number | null {
    if (s.paused) return s.pausedRemaining;
    if (s.phase === "idle" || s.deadline == null) return null;
    return Math.max(0, (s.deadline - nowMs) / 1000);
  },

  clock(seconds: number): string {
    const whole = Math.max(0, Math.floor(seconds));
    const m = Math.floor(whole / 60);
    const s = whole % 60;
    const sStr = String(s).padStart(2, "0");
    if (m >= 100) return `${m}:${sStr}`;
    return `${String(m).padStart(2, "0")}:${sStr}`;
  },

  land(
    phase: AiradoroPhase,
    phaseEndMs: number,
    nowMs: number,
    config: AiradoroConfig,
  ): { phase: AiradoroPhase; deadline: number; event: AiradoroEvent } {
    const work = Math.max(1, Math.round(config.work));
    const rest = Math.max(1, Math.round(config.rest));
    const phaseEndSec = phaseEndMs / 1000;
    const nowSec = nowMs / 1000;

    switch (phase) {
      case "work": {
        const restEndSec = phaseEndSec + rest;
        if (nowSec < restEndSec) {
          return { phase: "rest", deadline: restEndSec * 1000, event: "restStarts" };
        }
        const over = Math.max(0, Math.floor(nowSec - restEndSec));
        const cycle = work + rest;
        const cycles = Math.floor(over / cycle);
        const into = over % cycle;
        const originSec = restEndSec + cycles * cycle;
        if (into < work) {
          return { phase: "work", deadline: (originSec + work) * 1000, event: "workStarts" };
        }
        return { phase: "rest", deadline: (originSec + work + rest) * 1000, event: "restStarts" };
      }
      case "rest": {
        const workEndSec = phaseEndSec + work;
        if (nowSec < workEndSec) {
          return { phase: "work", deadline: workEndSec * 1000, event: "workStarts" };
        }
        const over = Math.max(0, Math.floor(nowSec - workEndSec));
        const cycle = work + rest;
        const cycles = Math.floor(over / cycle);
        const into = over % cycle;
        const originSec = workEndSec + cycles * cycle;
        if (into < rest) {
          return { phase: "rest", deadline: (originSec + rest) * 1000, event: "restStarts" };
        }
        return { phase: "work", deadline: (originSec + rest + work) * 1000, event: "workStarts" };
      }
      case "idle":
        return { phase: "work", deadline: (phaseEndSec + work) * 1000, event: "workStarts" };
    }
  },
};
