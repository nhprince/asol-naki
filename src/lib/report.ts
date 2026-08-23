import type { BatteryInfo, FullHardwareInfo, StorageInfo, Verdict } from "./types";
import type { FraudFlag } from "./useScan";

export interface ScanReport {
  generatedAtIso: string;
  device: {
    hostname: string;
    os: string;
  };
  cpu: string;
  threads: number;
  coresPhysical?: number;
  memoryMb: number;
  motherboard?: string;
  gpus: string[];
  battery: {
    healthPercent?: number;
    designWh?: number;
    fullChargeWh?: number;
    cycles?: number;
  } | null;
  storage: {
    model?: string;
    capacityGb?: number;
    smartStatus?: string;
    enduranceUsedPct?: number;
    powerOnHours?: number;
  }[];
  score: number;
  verdict: Verdict;
  flags: { severity: string; messageKey: string; detail?: string }[];
}

export function buildReport(args: {
  hardware: FullHardwareInfo;
  battery: BatteryInfo | null;
  storage: StorageInfo[] | null;
  score: number;
  verdict: Verdict;
  flags: FraudFlag[];
}): ScanReport {
  const { hardware: h, battery, storage, score, verdict, flags } = args;
  return {
    generatedAtIso: new Date().toISOString(),
    device: { hostname: h.hostname, os: `${h.os_name} ${h.os_version}` },
    cpu: h.cpu_name,
    threads: h.cpu_threads,
    coresPhysical: h.cpu_cores_physical,
    memoryMb: h.total_memory_mb,
    motherboard: h.motherboard,
    gpus: (h.gpus ?? []).map((g) => g.name),
    battery:
      battery && (battery.health_percent != null || battery.cycle_count != null)
        ? {
            healthPercent: battery.health_percent,
            designWh: battery.design_capacity_mwh != null
              ? Math.round(battery.design_capacity_mwh / 1000)
              : undefined,
            fullChargeWh: battery.full_charge_capacity_mwh != null
              ? Math.round(battery.full_charge_capacity_mwh / 1000)
              : undefined,
            cycles: battery.cycle_count,
          }
        : null,
    storage: (storage ?? []).map((d) => ({
      model: d.model_name,
      capacityGb: d.total_capacity_bytes != null
        ? Math.floor(d.total_capacity_bytes / 1024 ** 3) // matches Windows Explorer
        : undefined,
      smartStatus: d.smart_status,
      enduranceUsedPct: d.nvme_percentage_used,
      powerOnHours: d.power_on_hours,
    })),
    score,
    verdict,
    flags: flags.map((f) => ({
      severity: f.severity,
      messageKey: f.message_key,
      detail: f.detail,
    })),
  };
}

/** Plain-text summary for quick sharing (Messenger/WhatsApp). */
export function summaryText(r: ScanReport, t: (k: string) => string): string {
  const lines: string[] = [];
  lines.push(`${t("app.title")} — ${r.device.hostname} (${r.device.os})`);
  lines.push(`CPU: ${r.cpu}`);
  if (r.battery?.healthPercent != null) {
    lines.push(
      `${t("scan.batteryHealth")}: ${Math.round(r.battery.healthPercent)}%` +
        (r.battery.cycles != null ? ` · ${t("scan.batteryCycles")}: ${r.battery.cycles}` : ""),
    );
  }
  for (const s of r.storage) {
    if (s.model) lines.push(`SSD: ${s.model} (${s.capacityGb ?? "?"} GB)`);
  }
  lines.push("");
  lines.push(`${t("score.title")}: ${r.score.toFixed(1)}/10 — ${t(verdictKey(r.verdict))}`);
  const crit = r.flags.filter((f) => f.severity === "critical").length;
  if (crit > 0) lines.push(`⚠ ${crit} × CRITICAL`);
  return lines.join("\n");
}

function verdictKey(v: Verdict): string {
  switch (v) {
    case "good-buy":
      return "score.verdictGoodBuy";
    case "negotiate":
      return "score.verdictNegotiate";
    default:
      return "score.verdictWalkAway";
  }
}
