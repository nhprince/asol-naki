import { $, expect, browser } from "@wdio/globals";
import { invoke } from "@tauri-apps/api/core";

/**
 * Phase 2 exit gate, end-to-end on real Windows: feed a deliberately
 * spoofed device profile through the PRODUCTION run_integrity_checks
 * command (same IPC the UI uses) and assert it raises Critical flags.
 *
 * The runner's hardware is honest, so we fabricate the classic scam:
 * firmware string claims a big CPU while OS-visible thread count betrays
 * an entry-class chip.
 */
describe("Phase 2 — fraud detection", () => {
  it("flags a spoofed CPU identity as CRITICAL via production IPC", async () => {
    const spoofed = {
      cpu_name: "Intel Core Ultra 5 125H", // claimed
      cpu_threads: 4, // reality of an i3-class chip
      cpu_cores_physical: 2,
      total_memory_mb: 8192,
      os_name: "Windows",
      os_version: "11",
      kernel_version: "10.0.22631",
      hostname: "SCAM-MACHINE",
    };

    const report = await invoke<{
      flags: { severity: string; check_id: string }[];
      has_critical: boolean;
    }>("run_integrity_checks", {
      hardwareJson: JSON.stringify(spoofed),
      storageJson: [],
    });

    await browser.saveScreenshot("./screenshots/spoof-check.png");
    expect(report.has_critical).toBe(true);
    const ids = report.flags.map((f) => f.check_id);
    expect(ids).toContain("cpu_thread_count_mismatch");
    expect(ids).toContain("cpu_core_count_mismatch");
  });

  it("does NOT flag an honest known-CPU profile", async () => {
    // Ultra 5 125H ground truth from known_models.json: 14c/18t.
    const honest = {
      cpu_name: "Intel Core Ultra 5 125H",
      cpu_threads: 18,
      cpu_cores_physical: 14,
      total_memory_mb: 16384,
      os_name: "Windows",
      os_version: "11",
      kernel_version: "10.0.22631",
      hostname: "PROBOOK-G11",
    };

    const report = await invoke<{
      flags: unknown[];
      has_critical: boolean;
    }>("run_integrity_checks", {
      hardwareJson: JSON.stringify(honest),
      storageJson: [],
    });

    expect(report.has_critical).toBe(false);
    expect(report.flags.length).toBe(0);
  });
});
