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

  it("toggles to Bengali and back", async () => {
    const scanButton = await $('[data-testid="scan-button"]');
    await scanButton.waitForDisplayed({ timeout: 10_000 });
    const before = await scanButton.getText();

    const bnButton = await $('button[data-lang="bn"]');
    await bnButton.waitForClickable({ timeout: 10_000 });
    await bnButton.click();

    // Poll manually so we can log what we actually see on timeout/failure.
    let after = "";
    for (let i = 0; i < 20; i++) {
      await browser.pause(250);
      after = await scanButton.getText();
      if (after !== before) break;
    }
    await browser.saveScreenshot("./screenshots/toggle-bn.png");

    const enButton = await $('button[data-lang="en"]');
    await enButton.waitForClickable({ timeout: 10_000 });
    await enButton.click();
    await browser.pause(500);
    const reverted = await scanButton.getText();
    await browser.saveScreenshot("./screenshots/toggle-back-en.png");

    console.log(
      `[toggle] before=${JSON.stringify(before)} after=${JSON.stringify(after)} reverted=${JSON.stringify(reverted)}`,
    );

    if (after === before) {
      throw new Error(
        `Language toggle had no effect on UI text (before=${JSON.stringify(before)}, after=${JSON.stringify(after)})`,
      );
    }
    expect(reverted).toBe(before);
  });
});
