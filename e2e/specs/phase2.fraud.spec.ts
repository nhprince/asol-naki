import { $, expect, browser } from "@wdio/globals";

/**
 * Phase 2 exit gate, end-to-end on real Windows: feed deliberately spoofed
 * device profiles through the PRODUCTION run_integrity_checks command —
 * executed inside the app's webview (same IPC the UI uses) — and assert
 * Critical flags fire for scams while honest profiles stay clean.
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

    const report = await browser.executeAsync(
      (hwJson, done) => {
        // @ts-expect-error injected Tauri global inside the webview
        window.__TAURI_INTERNALS__
          .invoke("run_integrity_checks", {
            hardwareJson: hwJson,
            storageJson: [],
          })
          .then(done)
          .catch((e: unknown) => done({ error: String(e) }));
      },
      JSON.stringify(spoofed),
    );

    await browser.saveScreenshot("./screenshots/spoof-check.png");
    const r = report as { flags?: { check_id: string }[]; has_critical?: boolean; error?: string };
    expect(r.error).toBeUndefined();
    expect(r.has_critical).toBe(true);
    const ids = (r.flags ?? []).map((f) => f.check_id);
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

    const report = await browser.executeAsync(
      (hwJson, done) => {
        // @ts-expect-error injected Tauri global inside the webview
        window.__TAURI_INTERNALS__
          .invoke("run_integrity_checks", {
            hardwareJson: hwJson,
            storageJson: [],
          })
          .then(done)
          .catch((e: unknown) => done({ error: String(e) }));
      },
      JSON.stringify(honest),
    );

    const r = report as { flags?: unknown[]; has_critical?: boolean; error?: string };
    expect(r.error).toBeUndefined();
    expect(r.has_critical).toBe(false);
    expect(r.flags?.length ?? -1).toBe(0);
  });
});
