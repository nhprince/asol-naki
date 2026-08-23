import { $, expect, browser } from "@wdio/globals";

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

  it("toggles to Bengali and applies i18n", async () => {
    const heading = await $('[data-testid="app-title"]');
    await expect(heading).toHaveText(expect.stringContaining("Asol Naki"));

    // JS-dispatched click: deterministic under the embedded driver. This test
    // targets the i18n wiring (React onClick → changeLanguage → re-render),
    // not OS-level input synthesis.
    await browser.execute(() => {
      const btn = document.querySelector(
        'button[data-lang="bn"]',
      ) as HTMLElement | null;
      btn?.click();
    });

    let bnText = "";
    for (let i = 0; i < 20; i++) {
      await browser.pause(250);
      bnText = await (await $('[data-testid="app-title"]')).getText();
      if (bnText.includes("আসল")) break;
    }

    await browser.saveScreenshot("./screenshots/toggle-bn.png");
    console.log(`[toggle] heading after BN click: ${JSON.stringify(bnText)}`);
    expect(bnText).toContain("আসল");
  });
});
