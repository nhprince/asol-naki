/**
 * WebdriverIO config driving the REAL compiled Tauri app via the
 * official @wdio/tauri-service in "embedded" mode: the W3C WebDriver
 * server runs INSIDE the app (via the feature-gated
 * tauri-plugin-wdio-webdriver crate) — no external tauri-driver or
 * msedgedriver install needed.
 *
 * CI builds the app with `--features wdio` and points TAURI_APP_BINARY_PATH
 * at the binary before running this suite.
 */
// WDIO's TS types don't model the tauri service options; plain record keeps
// typecheck happy while matching the documented shape exactly.
/* eslint-disable @typescript-eslint/no-explicit-any */
export const config: Record<string, any> = {
  runner: "local",

  specs: ["./specs/**/*.spec.ts"],

  // one app window at a time
  maxInstances: 1,
  maxInstancesPerCapability: 1,

  capabilities: [
    {
      browserName: "tauri",
      "tauri:options": {
        application:
          process.env.TAURI_APP_BINARY_PATH ??
          "./src-tauri/target/debug/asol-naki.exe",
      },
    },
  ],

  services: [
    [
      "@wdio/tauri-service",
      {
        // WebDriver server runs inside the app binary itself.
        driverProvider: "embedded",
        logLevel: "info",
        startTimeout: 30_000,
        commandTimeout: 30_000,
        captureBackendLogs: true,
        captureFrontendLogs: true,
      },
    ],
  ],

  framework: "mocha",
  mochaOpts: {
    ui: "bdd",
    timeout: 60_000,
  },

  reporters: ["spec"],
  outputDir: "./logs",
  logLevel: "info",
};
