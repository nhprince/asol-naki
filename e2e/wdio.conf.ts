import type { Options } from "@wdio/types";

/**
 * WebdriverIO config driving the REAL compiled Tauri app via the
 * official @wdio/tauri-service (embedded WebDriver server; on Windows it
 * keeps msedgedriver in sync with WebView2 automatically).
 *
 * The service builds/locates the binary via TAURI_APP_BINARY_PATH or its
 * appBinaryPath option — set by CI after `tauri build --debug`.
 */
export const config: Options.Testrunner = {
  runner: "local",
  specs: ["./specs/**/*.spec.ts"],
  maxInstances: 1, // one app window at a time
  capabilities: [
    {
      "tauri:options": {
        appBinaryPath: process.env.TAURI_APP_BINARY_PATH ?? "",
      },
    } as never,
  ],
  services: [["tauri", { driverProvider: "embedded" }]],
  framework: "mocha",
  mochaOpts: {
    ui: "bdd",
    timeout: 60_000,
  },
  reporters: ["spec"],
  outputDir: "./logs",
  logLevel: "info",
};
