import { $, expect } from "@wdio/globals";

describe("Asol Naki? — Phase 0 smoke", () => {
  it("launches and shows the app heading", async () => {
    const heading = await $('[data-testid="app-title"]');
    await expect(heading).toHaveText(expect.stringContaining("Asol Naki"));
  });

  it("auto-scans and shows a real CPU name from the Rust core", async () => {
    const cpu = await $('[data-testid="cpu-name"]');
    // The Rust command must have resolved by now (auto-scan on launch).
    await cpu.waitForDisplayed({ timeout: 30_000 });
    const text = await cpu.getText();
    // Real CPU strings are longer than 4 chars; empty string = invoke failed.
    expect(text.length).toBeGreaterThan(4);
  });

  it("toggles to Bengali and back", async () => {
    const scanButton = await $('[data-testid="scan-button"]');
    await expect(scanButton).toHaveText(expect.stringContaining("Scan"));

    await $('button[data-lang="bn"]').click();
    await expect(scanButton).toHaveText(expect.stringContaining("স্ক্যান"));

    await $('button[data-lang="en"]').click();
    await expect(scanButton).toHaveText(expect.stringContaining("Scan"));
  });
});
