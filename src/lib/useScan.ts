import { invoke } from "@tauri-apps/api/core";
import { useCallback, useRef, useState } from "react";
import { verdictForScore, type Verdict } from "./format";
import type {
  BatteryInfo,
  DisplayInfo,
  FullHardwareInfo,
  StorageInfo,
} from "./types";

export interface FraudFlag {
  severity: "info" | "warning" | "critical";
  check_id: string;
  message_key: string;
  detail?: string;
}

export interface GuidedTestResults {
  display?: { pass: boolean; note?: string } | null;
  keyboard?: { pass: boolean; note?: string } | null;
  ports?: { pass: boolean; note?: string } | null;
}

export interface ScanState {
  running: boolean;
  hardware: FullHardwareInfo | null;
  battery: BatteryInfo | null;
  storage: StorageInfo[] | null;
  display: DisplayInfo[] | null;
  score: number | null;
  verdict: Verdict | null;
  cappedByCritical: boolean;
  flags: FraudFlag[];
  guided: GuidedTestResults;
}

interface SectionError {
  section:
    | "hardware"
    | "battery"
    | "storage"
    | "display"
    | "integrity";
  message: string;
}

export function useScan() {
  const [state, setState] = useState<ScanState>({
    running: false,
    hardware: null,
    battery: null,
    storage: null,
    display: null,
    score: null,
    verdict: null,
    cappedByCritical: false,
    flags: [],
    guided: {
      display: null,
      keyboard: null,
      ports: null,
    },
  });
  const [errors, setErrors] = useState<SectionError[]>([]);

  const guidedRef = useRef<GuidedTestResults>(state.guided);
  guidedRef.current = state.guided;

  const calculateScoreAndVerdict = useCallback(
    (
      storage: StorageInfo[] | null,
      battery: BatteryInfo | null,
      guided: GuidedTestResults,
      hasCritical: boolean,
    ) => {
      if (!storage || storage.length === 0) {
        return { score: null, verdict: null, cappedByCritical: false };
      }

      const best = [...storage].sort(
        (a, b) => subscoreStorage(b) - subscoreStorage(a),
      )[0];
      const storageScore = subscoreStorage(best);
      const batteryScore =
        battery?.health_percent != null
          ? subscoreBattery(battery.health_percent)
          : null;

      // Guided Display Score
      let displayScore = 9.5;
      if (guided.display) {
        displayScore = guided.display.pass ? 10.0 : 4.0;
      }

      // Guided Ports & Keyboard Score
      let portsScore = 10.0;
      if (guided.keyboard && !guided.keyboard.pass) portsScore -= 3.0;
      if (guided.ports && !guided.ports.pass) portsScore -= 3.0;
      portsScore = Math.max(0, portsScore);

      const present: [number, number][] = [
        [storageScore, 0.25],
        [9.0, 0.25],
        ...(batteryScore != null ? [[batteryScore, 0.2] as [number, number]] : []),
        [displayScore, 0.15],
        [portsScore, 0.10],
      ];

      const wSum = present.reduce((acc, [, w]) => acc + w, 0);
      const weighted = present.reduce((acc, [s, w]) => acc + s * w, 0) / wSum;

      const CRITICAL_CAP = 3.0;
      const cappedByCritical = hasCritical && weighted > CRITICAL_CAP;
      const finalScore = cappedByCritical
        ? CRITICAL_CAP
        : Math.min(10, Math.max(0, weighted));
      const score = Math.round(finalScore * 10) / 10;
      const verdict = verdictForScore(score);

      return { score, verdict, cappedByCritical };
    },
    [],
  );

  const run = useCallback(async () => {
    setState((s) => ({ ...s, running: true }));
    setErrors([]);
    const errs: SectionError[] = [];

    const [hw, bat, stor, disp] = await Promise.allSettled([
      invoke<FullHardwareInfo>("scan_hardware_full"),
      invoke<BatteryInfo>("scan_battery"),
      invoke<StorageInfo[]>("scan_storage"),
      invoke<DisplayInfo[]>("scan_display"),
    ]);

    let hardware: FullHardwareInfo | null = null;
    let battery: BatteryInfo | null = null;
    let storage: StorageInfo[] | null = null;
    let display: DisplayInfo[] | null = null;

    if (hw.status === "fulfilled") {
      hardware = hw.value;
    } else {
      errs.push({ section: "hardware", message: String(hw.reason) });
    }
    if (bat.status === "fulfilled") {
      battery = bat.value;
    } else if (
      !String(bat.reason).includes("requires Windows") &&
      !String(bat.reason).includes("No battery present")
    ) {
      errs.push({ section: "battery", message: String(bat.reason) });
    }
    if (stor.status === "fulfilled") {
      storage = stor.value;
    } else if (!String(stor.reason).includes("requires Windows")) {
      errs.push({ section: "storage", message: String(stor.reason) });
    }
    if (disp.status === "fulfilled") {
      display = disp.value;
    } else if (!String(disp.reason).includes("requires Windows")) {
      errs.push({ section: "display", message: String(disp.reason) });
    }

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

    const { score, verdict, cappedByCritical } = calculateScoreAndVerdict(
      storage,
      battery,
      guidedRef.current,
      hasCritical,
    );

    setState((prev) => ({
      ...prev,
      running: false,
      hardware,
      battery,
      storage,
      display,
      score,
      verdict,
      cappedByCritical,
      flags,
    }));
    setErrors(errs);
  }, [calculateScoreAndVerdict]);

  const updateGuidedResult = useCallback(
    (key: keyof GuidedTestResults, result: { pass: boolean; note?: string } | null) => {
      setState((prev) => {
        const newGuided = { ...prev.guided, [key]: result };
        guidedRef.current = newGuided;
        const hasCritical = prev.flags.some((f) => f.severity === "critical");
        const { score, verdict, cappedByCritical } = calculateScoreAndVerdict(
          prev.storage,
          prev.battery,
          newGuided,
          hasCritical,
        );
        return {
          ...prev,
          guided: newGuided,
          score,
          verdict,
          cappedByCritical,
        };
      });
    },
    [calculateScoreAndVerdict],
  );

  return { ...state, errors, run, updateGuidedResult };
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
