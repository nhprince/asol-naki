import { invoke } from "@tauri-apps/api/core";
import { useCallback, useState } from "react";
import { verdictForScore, type Verdict } from "./format";
import type {
  BatteryInfo,
  FullHardwareInfo,
  StorageInfo,
} from "./types";

export interface FraudFlag {
  severity: "info" | "warning" | "critical";
  check_id: string;
  message_key: string;
  detail?: string;
}

export interface ScanState {
  running: boolean;
  hardware: FullHardwareInfo | null;
  battery: BatteryInfo | null;
  storage: StorageInfo[] | null;
  score: number | null;
  verdict: Verdict | null;
  cappedByCritical: boolean;
  flags: FraudFlag[];
}

interface SectionError {
  section: "hardware" | "battery" | "storage" | "integrity";
  message: string;
}

/**
 * Runs all diagnostic commands in parallel. A failing section is recorded
 * and skipped — one broken module must never kill the whole scan.
 */
export function useScan() {
  const [state, setState] = useState<ScanState>({
    running: false,
    hardware: null,
    battery: null,
    storage: null,
    score: null,
    verdict: null,
    cappedByCritical: false,
    flags: [],
  });
  const [errors, setErrors] = useState<SectionError[]>([]);

  const run = useCallback(async () => {
    setState((s) => ({ ...s, running: true }));
    setErrors([]);
    const errs: SectionError[] = [];

    const [hw, bat, stor] = await Promise.allSettled([
      invoke<FullHardwareInfo>("scan_hardware_full"),
      invoke<BatteryInfo>("scan_battery"),
      invoke<StorageInfo[]>("scan_storage"),
    ]);

    let hardware: FullHardwareInfo | null = null;
    let battery: BatteryInfo | null = null;
    let storage: StorageInfo[] | null = null;

    if (hw.status === "fulfilled") {
      hardware = hw.value;
    } else {
      errs.push({ section: "hardware", message: String(hw.reason) });
    }
    if (bat.status === "fulfilled") {
      battery = bat.value;
    } else if (!String(bat.reason).includes("requires Windows")) {
      errs.push({ section: "battery", message: String(bat.reason) });
    }
    if (stor.status === "fulfilled") {
      storage = stor.value;
    } else if (!String(stor.reason).includes("requires Windows")) {
      errs.push({ section: "storage", message: String(stor.reason) });
    }

    // Fraud checks run whenever we have hardware data; integrity failures
    // are surfaced but never block scoring of the remaining data.
    let flags: FraudFlag[] = [];
    let hasCritical = false;
    if (hardware) {
      try {
        const report = await invoke<{
          flags: FraudFlag[];
          has_critical: boolean;
        }>("run_integrity_checks", {
          hardwareJson: JSON.stringify(hardware),
          storageJson: (storage ?? []).map((d) => JSON.stringify(d)),
        });
        flags = report.flags;
        hasCritical = report.has_critical;
      } catch (err) {
        errs.push({ section: "integrity", message: String(err) });
      }
    }

    // Score mirrors src-tauri/src/scoring.rs. Weights per plan.md §7
    // normalized over present categories only.
    let score: number | null = null;
    let verdict: Verdict | null = null;
    if (storage && storage.length > 0) {
      const best = [...storage].sort(
        (a, b) => subscoreStorage(b) - subscoreStorage(a),
      )[0];
      const storageScore = subscoreStorage(best);
      const batteryScore =
        battery?.health_percent != null
          ? subscoreBattery(battery.health_percent)
          : null;

      const present: [number, number][] = [
        [storageScore, 0.25],
        [9.0, 0.25], // cpu/gpu sanity placeholder until deeper Phase 2 signals
        ...(batteryScore != null ? [[batteryScore, 0.2] as [number, number]] : []),
        [9.5, 0.15], // display placeholder until EDID
        [10.0, 0.1], // ports placeholder until guided tests
      ];
      const wSum = present.reduce((acc, [, w]) => acc + w, 0);
      const weighted = present.reduce((acc, [s, w]) => acc + s * w, 0) / wSum;

      // plan.md §6: Critical flag caps the score at 3.0 no matter what.
      const CRITICAL_CAP = 3.0;
      let s = Math.round(weighted * 10) / 10;
      let cappedByCritical = false;
      if (hasCritical && s > CRITICAL_CAP) {
        s = CRITICAL_CAP;
        cappedByCritical = true;
      }
      score = s;
      verdict = verdictForScore(s);
      setState((prev) => ({ ...prev, cappedByCritical }));
    }

    setState((prev) => ({
      ...prev,
      running: false,
      hardware,
      battery,
      storage,
      score,
      verdict,
      flags,
    }));
    setErrors(errs);
  }, []);

  return { ...state, errors, run };
}

function subscoreBattery(healthPercent: number): number {
  if (healthPercent >= 90) return 10;
  if (healthPercent <= 40) return 0;
  return Math.round(((healthPercent - 40) / 50) * 100) / 10;
}

function subscoreStorage(d: StorageInfo): number {
  let s = d.smart_status?.toLowerCase() === "failed" ? 0 : 10;
  if (d.nvme_percentage_used != null) {
    s = Math.min(s, (100 - clamp(d.nvme_percentage_used, 0, 100)) / 10);
  }
  if ((d.realloc_sector_count ?? 0) > 0) {
    s = Math.min(s, 6 - Math.min(d.realloc_sector_count!, 200) / 50);
  }
  if ((d.pending_sector_count ?? 0) > 0) s = Math.min(s, 5);
  if ((d.media_errors ?? 0) > 0) s = Math.min(s, 4);
  return Math.round(clamp(s, 0, 10) * 10) / 10;
}

function clamp(n: number, lo: number, hi: number): number {
  return Math.max(lo, Math.min(hi, n));
}
