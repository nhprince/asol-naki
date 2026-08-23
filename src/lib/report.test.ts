import { describe, it, expect } from "vitest";
import { buildReport, summaryText } from "./report";
import type { FullHardwareInfo, BatteryInfo, StorageInfo } from "./types";

const hw: FullHardwareInfo = {
  cpu_name: "Intel Core Ultra 5 125H",
  cpu_threads: 18,
  cpu_cores_physical: 14,
  total_memory_mb: 16384,
  os_name: "Windows",
  os_version: "11 Pro",
  kernel_version: "10.0.22631",
  hostname: "PROBOOK-G11",
  motherboard: "HP 8A43",
  gpus: [{ name: "Intel Iris Xe Graphics" }],
};

const battery: BatteryInfo = {
  design_capacity_mwh: 68472,
  full_charge_capacity_mwh: 61240,
  health_percent: 89.4,
  cycle_count: 142,
  manufacturer: "SMP",
  chemistry: "LION",
};

const storage: StorageInfo[] = [
  {
    model_name: "Samsung SSD 980 PRO 1TB",
    protocol: "nvme",
    total_capacity_bytes: 1000204886016,
    smart_status: "passed",
    nvme_percentage_used: 3,
    power_on_hours: 2871,
  },
];

describe("buildReport", () => {
  it("maps all diagnostics into the report shape", () => {
    const r = buildReport({
      hardware: hw,
      battery,
      storage,
      score: 7.7,
      verdict: "good-buy",
      flags: [],
    });
    expect(r.cpu).toBe("Intel Core Ultra 5 125H");
    expect(r.battery?.healthPercent).toBe(89.4);
    expect(r.battery?.designWh).toBe(68); // mWh → Wh rounding
    expect(r.storage[0]?.capacityGb).toBe(931);
    expect(r.gpus).toEqual(["Intel Iris Xe Graphics"]);
    expect(r.score).toBe(7.7);
  });

  it("handles null battery/storage gracefully", () => {
    const r = buildReport({
      hardware: hw,
      battery: null,
      storage: null,
      score: 8,
      verdict: "good-buy",
      flags: [],
    });
    expect(r.battery).toBeNull();
    expect(r.storage).toEqual([]);
  });
});

describe("summaryText", () => {
  const t = (k: string) => k.split(".").pop() ?? k;

  it("produces shareable text with score and verdict", () => {
    const r = buildReport({
      hardware: hw,
      battery,
      storage,
      score: 7.7,
      verdict: "good-buy",
      flags: [],
    });
    const text = summaryText(r, t);
    expect(text).toContain("PROBOOK-G11");
    expect(text).toContain("Ultra 5 125H");
    expect(text).toContain("89%");
    expect(text).toContain("7.7/10");
    expect(text).toContain("verdictGoodBuy");
  });

  it("marks critical flag count", () => {
    const r = buildReport({
      hardware: hw,
      battery,
      storage,
      score: 3,
      verdict: "walk-away",
      flags: [
        { severity: "critical", check_id: "t1", message_key: "fraud.cpuThreadMismatch" },
        { severity: "critical", check_id: "t2", message_key: "fraud.cpuCoreMismatch" },
      ],
    });
    expect(summaryText(r, t)).toContain("2 × CRITICAL");
  });
});
